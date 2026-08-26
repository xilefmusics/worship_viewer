//! Uploaded audio/video and slide-deck processing jobs, lifecycle, and reconciliation.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use shared::MediaAssetKind;
use shared::media::{
    CommitDeck, DeclaredMediaKind, Media, MediaContent, MediaDeckPage, MediaPendingRevision,
    MediaStagedDeckPage, MediaStatus,
};
use surrealdb::types::RecordId;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::auth::AuthorizationContext;
use crate::error::AppError;
use crate::resources::media::av_processor::{
    AvProcessFailure, AvProcessor, FfmpegAvProcessor, processing_error,
};
use crate::resources::media::deck_processor::{
    DeckProcessor, PopplerDeckProcessor, UnsupportedDeckProcessor,
};
use crate::resources::media::model::MediaWrite;
use crate::resources::media::repository::MediaRepository;
use crate::resources::media::surreal_repo::SurrealMediaRepo;
use crate::resources::team::parse_owner_record_id;
use crate::settings::Settings;

#[derive(Clone, Default)]
struct ProcessingCoordinator {
    active_operations: Arc<RwLock<HashSet<String>>>,
    cancel_tokens: Arc<RwLock<HashMap<String, CancellationToken>>>,
    revision_sources: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl ProcessingCoordinator {
    async fn register(&self, operation_id: &str) -> CancellationToken {
        self.active_operations
            .write()
            .await
            .insert(operation_id.to_owned());
        let token = CancellationToken::new();
        self.cancel_tokens
            .write()
            .await
            .insert(operation_id.to_owned(), token.clone());
        token
    }

    async fn unregister(&self, operation_id: &str) {
        self.active_operations.write().await.remove(operation_id);
        self.cancel_tokens.write().await.remove(operation_id);
    }

    async fn cancel(&self, operation_id: &str) {
        if let Some(token) = self.cancel_tokens.read().await.get(operation_id) {
            token.cancel();
        }
    }

    pub async fn active_operation_ids(&self) -> HashSet<String> {
        self.active_operations.read().await.clone()
    }

    async fn register_source(&self, revision_id: &str, staging_op: &str) -> CancellationToken {
        let token = self.register(staging_op).await;
        self.revision_sources
            .write()
            .await
            .entry(revision_id.to_owned())
            .or_default()
            .insert(staging_op.to_owned());
        token
    }

    async fn unregister_source(&self, revision_id: &str, staging_op: &str) {
        self.unregister(staging_op).await;
        let mut map = self.revision_sources.write().await;
        if let Some(set) = map.get_mut(revision_id) {
            set.remove(staging_op);
            if set.is_empty() {
                map.remove(revision_id);
            }
        }
    }

    async fn revision_in_flight(&self, revision_id: &str) -> bool {
        self.revision_sources
            .read()
            .await
            .get(revision_id)
            .is_some_and(|set| !set.is_empty())
    }

    async fn other_sources_in_flight(&self, revision_id: &str, except: &str) -> bool {
        self.revision_sources
            .read()
            .await
            .get(revision_id)
            .is_some_and(|set| set.iter().any(|op| op != except))
    }

    async fn cancel_revision(&self, revision_id: &str) {
        let ops = self
            .revision_sources
            .read()
            .await
            .get(revision_id)
            .cloned()
            .unwrap_or_default();
        for op in ops {
            self.cancel(&op).await;
        }
    }
}

#[derive(Clone)]
pub struct MediaProcessingHandle {
    media_repo: SurrealMediaRepo,
    asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
    coordinator: ProcessingCoordinator,
    processor: Arc<dyn AvProcessor>,
    deck_processor: Arc<dyn DeckProcessor>,
    work_parent: PathBuf,
    deck_max_pages: usize,
    deck_write: Arc<tokio::sync::Mutex<()>>,
}

impl MediaProcessingHandle {
    pub fn build(
        db: Arc<crate::database::Database>,
        settings: &Settings,
        asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
    ) -> Self {
        let processor: Arc<dyn AvProcessor> = Arc::new(FfmpegAvProcessor {
            ffmpeg_path: settings.ffmpeg_path.clone(),
            ffprobe_path: settings.ffprobe_path.clone(),
            timeout: settings.media_processing_timeout(),
        });
        let deck_processor: Arc<dyn DeckProcessor> = Arc::new(PopplerDeckProcessor {
            pdfinfo_path: settings.pdfinfo_path.clone(),
            pdfseparate_path: settings.pdfseparate_path.clone(),
            timeout: settings.media_processing_timeout(),
        });
        Self::build_with_processors(db, settings, asset_svc, processor, deck_processor)
    }

    pub fn build_with_processor(
        db: Arc<crate::database::Database>,
        settings: &Settings,
        asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
        processor: Arc<dyn AvProcessor>,
    ) -> Self {
        Self::build_with_processors(
            db,
            settings,
            asset_svc,
            processor,
            Arc::new(UnsupportedDeckProcessor),
        )
    }

    pub fn build_with_processors(
        db: Arc<crate::database::Database>,
        settings: &Settings,
        asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
        processor: Arc<dyn AvProcessor>,
        deck_processor: Arc<dyn DeckProcessor>,
    ) -> Self {
        let work_parent = std::env::temp_dir().join("worshipviewer_media_work");
        Self {
            media_repo: SurrealMediaRepo::new(db),
            asset_svc,
            coordinator: ProcessingCoordinator::default(),
            processor,
            deck_processor,
            work_parent,
            deck_max_pages: settings.media_deck_max_pages as usize,
            deck_write: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub async fn active_processing_operation_ids(&self) -> HashSet<String> {
        self.coordinator.active_operation_ids().await
    }

    pub fn spawn_job(self: &Arc<Self>, media_id: String, operation_id: String) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(err) = this.run_job(&media_id, &operation_id).await {
                tracing::warn!(
                    media_id = %media_id,
                    operation_id = %operation_id,
                    error = %err,
                    "media processing job failed internally"
                );
            }
        });
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn begin_after_upload(
        self: &Arc<Self>,
        media_id: &str,
        operation_id: &str,
        kind: MediaAssetKind,
        replace_page_id: Option<String>,
    ) -> Result<(), AppError> {
        let media = self.media_repo.get_unscoped(media_id).await?;
        if is_deck_asset_kind(kind) && !is_deck_media(&media) {
            return Ok(());
        }
        validate_upload_kind(&media, kind)?;
        if is_deck_asset_kind(kind) {
            return self
                .begin_deck_source(media_id, operation_id, replace_page_id)
                .await;
        }
        if replace_page_id.is_some() {
            return Err(AppError::invalid_request(
                "replace_page is only valid for slide-deck uploads",
            ));
        }
        if let Some(old) = media.pending_revision.as_ref() {
            self.coordinator.cancel(&old.operation).await;
        }
        let is_replacement = is_ready_uploaded_replacement(&media);
        let pending = MediaPendingRevision::processing(operation_id);
        let write = if is_replacement {
            MediaWrite {
                title: media.title.clone(),
                status: MediaStatus::Ready,
                content: media.content.clone(),
                pending_revision: Some(pending),
                declared_kind: media.declared_kind,
            }
        } else {
            MediaWrite {
                title: media.title.clone(),
                status: MediaStatus::Processing,
                content: None,
                pending_revision: Some(pending),
                declared_kind: media.declared_kind,
            }
        };
        self.media_repo.update_unscoped(media_id, write).await?;
        self.coordinator.register(operation_id).await;
        self.spawn_job(media_id.to_owned(), operation_id.to_owned());
        Ok(())
    }

    pub async fn cancel_pending_for_user(
        &self,
        ctx: &crate::auth::AuthorizationContext,
        media_id: &str,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let media = self.media_repo.get(&write_teams, media_id).await?;
        let owner = parse_owner_record_id(&media.owner)?;
        ctx.require_write_access_to_owner(&owner)?;
        if let Some(pending) = media.pending_revision.as_ref() {
            if is_deck_media(&media) {
                self.coordinator.cancel_revision(&pending.operation).await;
                self.cleanup_draft_assets(&media, pending).await;
            } else {
                self.coordinator.cancel(&pending.operation).await;
            }
            let write = MediaWrite {
                title: media.title.clone(),
                status: media.status,
                content: media.content.clone(),
                pending_revision: None,
                declared_kind: media.declared_kind,
            };
            return self
                .media_repo
                .update(&write_teams, media_id, None, write)
                .await;
        }
        Ok(media)
    }

    pub async fn cancel_for_delete(&self, media: &Media) {
        if let Some(pending) = media.pending_revision.as_ref() {
            if is_deck_media(media) {
                self.coordinator.cancel_revision(&pending.operation).await;
            } else {
                self.coordinator.cancel(&pending.operation).await;
            }
        }
    }

    pub async fn reconcile_stranded(&self) -> Result<u64, AppError> {
        let media_rows = self.media_repo.list_processing_media().await?;
        let mut reconciled = 0u64;
        for media in media_rows {
            let needs_fail = media.status == MediaStatus::Processing
                || media
                    .pending_revision
                    .as_ref()
                    .is_some_and(|p| p.status == MediaStatus::Processing);
            if !needs_fail {
                continue;
            }
            if let Some(pending) = media.pending_revision.as_ref() {
                if is_deck_media(&media) {
                    self.coordinator.cancel_revision(&pending.operation).await;
                    self.cleanup_draft_assets(&media, pending).await;
                } else {
                    self.coordinator.cancel(&pending.operation).await;
                    if let Ok(staging) = self
                        .asset_svc
                        .get_staging_by_operation(&pending.operation)
                        .await
                    {
                        if let Some(op) = &staging.operation_id {
                            self.asset_svc.delete_staging_file(op);
                        }
                        let _ = self.asset_svc.delete_asset_record(&staging.id).await;
                    }
                }
            }
            let failure = processing_error(AvProcessFailure::Failed);
            let operation = media
                .pending_revision
                .as_ref()
                .map(|p| p.operation.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let write = if media.status == MediaStatus::Ready && media.content.is_some() {
                MediaWrite {
                    title: media.title.clone(),
                    status: MediaStatus::Ready,
                    content: media.content.clone(),
                    pending_revision: Some(MediaPendingRevision::failed(operation, failure)),
                    declared_kind: media.declared_kind,
                }
            } else {
                MediaWrite {
                    title: media.title.clone(),
                    status: MediaStatus::Failed,
                    content: None,
                    pending_revision: Some(MediaPendingRevision::failed(operation, failure)),
                    declared_kind: media.declared_kind,
                }
            };
            self.media_repo.update_unscoped(&media.id, write).await?;
            reconciled += 1;
        }
        Ok(reconciled)
    }

    async fn run_job(&self, media_id: &str, operation_id: &str) -> Result<(), AppError> {
        let cancel = self
            .coordinator
            .cancel_tokens
            .read()
            .await
            .get(operation_id)
            .cloned();
        if cancel.as_ref().is_some_and(|t| t.is_cancelled()) {
            self.cleanup_stale(operation_id).await;
            self.coordinator.unregister(operation_id).await;
            return Ok(());
        }

        let result = self.process_upload(media_id, operation_id).await;
        self.coordinator.unregister(operation_id).await;
        result
    }

    async fn process_upload(&self, media_id: &str, operation_id: &str) -> Result<(), AppError> {
        let media = self.media_repo.get_unscoped(media_id).await?;
        if !operation_matches(&media, operation_id) {
            self.cleanup_stale(operation_id).await;
            return Ok(());
        }

        let staging = match self.asset_svc.get_staging_by_operation(operation_id).await {
            Ok(s) => s,
            Err(_) => {
                self.fail_media(&media, operation_id, AvProcessFailure::InputInvalid)
                    .await?;
                return Ok(());
            }
        };
        let staging_path = self.asset_svc.staging_path(operation_id);
        let kind = staging.kind;

        let transcode_result = match self.processor.probe_input(&staging_path, kind).await {
            Err(failure) => {
                self.fail_media(&media, operation_id, failure).await?;
                self.cleanup_staging(operation_id, &staging.id).await;
                return Ok(());
            }
            Ok(_) => match self
                .processor
                .transcode(&staging_path, kind, &self.work_parent)
                .await
            {
                Ok(r) => r,
                Err(failure) => {
                    self.fail_media(&media, operation_id, failure).await?;
                    self.cleanup_staging(operation_id, &staging.id).await;
                    return Ok(());
                }
            },
        };

        if !operation_still_matches(media_id, operation_id, &self.media_repo).await? {
            self.cleanup_stale(operation_id).await;
            return Ok(());
        }

        let owner = parse_owner_record_id(&media.owner)?;
        let media_rid = RecordId::new("media", media_id.to_owned());
        let content_type = match kind {
            MediaAssetKind::Video => "video/mp4",
            MediaAssetKind::Audio => "audio/mp4",
            _ => "application/octet-stream",
        };
        let final_asset = self
            .asset_svc
            .ingest_processed_file(
                owner,
                media_rid,
                kind,
                content_type.into(),
                &transcode_result.output_path,
            )
            .await?;

        let superseded_blob = superseded_blob_id(&media);
        let content = match kind {
            MediaAssetKind::Video => MediaContent::Video {
                blob_id: final_asset.id.clone(),
                duration_ms: transcode_result.duration_ms,
                width: transcode_result.width.unwrap_or(0),
                height: transcode_result.height.unwrap_or(0),
            },
            MediaAssetKind::Audio => MediaContent::Audio {
                blob_id: final_asset.id.clone(),
                duration_ms: transcode_result.duration_ms,
            },
            _ => {
                self.cleanup_staging(operation_id, &staging.id).await;
                self.asset_svc.delete_final_file(&final_asset.id);
                let _ = self.asset_svc.delete_asset_record(&final_asset.id).await;
                return self
                    .fail_media(&media, operation_id, AvProcessFailure::InputUnsupported)
                    .await;
            }
        };

        let write = MediaWrite {
            title: media.title.clone(),
            status: MediaStatus::Ready,
            content: Some(content),
            pending_revision: None,
            declared_kind: None,
        };
        if !self
            .commit_if_operation(media_id, operation_id, write)
            .await?
        {
            self.asset_svc.delete_final_file(&final_asset.id);
            let _ = self.asset_svc.delete_asset_record(&final_asset.id).await;
            self.cleanup_stale(operation_id).await;
            return Ok(());
        }

        self.cleanup_staging(operation_id, &staging.id).await;
        if let Some(old_id) = superseded_blob {
            self.asset_svc.delete_final_file(&old_id);
            let _ = self.asset_svc.delete_asset_record(&old_id).await;
        }
        Ok(())
    }

    async fn fail_media(
        &self,
        media: &Media,
        operation_id: &str,
        failure: AvProcessFailure,
    ) -> Result<(), AppError> {
        let err = processing_error(failure);
        let is_replacement = is_ready_uploaded_replacement(media);
        let pending = MediaPendingRevision::failed(operation_id, err);
        let write = if is_replacement {
            MediaWrite {
                title: media.title.clone(),
                status: MediaStatus::Ready,
                content: media.content.clone(),
                pending_revision: Some(pending),
                declared_kind: media.declared_kind,
            }
        } else {
            MediaWrite {
                title: media.title.clone(),
                status: MediaStatus::Failed,
                content: None,
                pending_revision: Some(pending),
                declared_kind: media.declared_kind,
            }
        };
        self.commit_if_operation(&media.id, operation_id, write)
            .await?;
        Ok(())
    }

    async fn commit_if_operation(
        &self,
        media_id: &str,
        operation_id: &str,
        write: MediaWrite,
    ) -> Result<bool, AppError> {
        let current = self.media_repo.get_unscoped(media_id).await?;
        if !operation_matches(&current, operation_id) {
            return Ok(false);
        }
        self.media_repo.update_unscoped(media_id, write).await?;
        Ok(true)
    }

    async fn cleanup_staging(&self, operation_id: &str, staging_asset_id: &str) {
        self.asset_svc.delete_staging_file(operation_id);
        let _ = self.asset_svc.delete_asset_record(staging_asset_id).await;
    }

    async fn cleanup_stale(&self, operation_id: &str) {
        self.asset_svc.delete_staging_file(operation_id);
        if let Ok(staging) = self.asset_svc.get_staging_by_operation(operation_id).await {
            let _ = self.asset_svc.delete_asset_record(&staging.id).await;
        }
    }

    async fn begin_deck_source(
        self: &Arc<Self>,
        media_id: &str,
        staging_op: &str,
        replace_page_id: Option<String>,
    ) -> Result<(), AppError> {
        let media = self.media_repo.get_unscoped(media_id).await?;
        let (revision_id, pages) = match media.pending_revision.as_ref() {
            Some(pending) if pending.status != MediaStatus::Failed || !pending.pages.is_empty() => {
                (pending.operation.clone(), pending.pages.clone())
            }
            _ if is_ready_deck(&media) => {
                let revision_id = uuid::Uuid::new_v4().to_string();
                (
                    revision_id,
                    staged_pages_from_content(media.content.as_ref()),
                )
            }
            _ => (uuid::Uuid::new_v4().to_string(), Vec::new()),
        };
        if let Some(page_id) = replace_page_id.as_deref()
            && !pages.iter().any(|p| p.id == page_id)
        {
            return Err(AppError::invalid_request("replace_page does not exist"));
        }
        let is_replacement = is_ready_deck(&media);
        let pending = MediaPendingRevision {
            operation: revision_id.clone(),
            status: MediaStatus::Processing,
            processing_error: None,
            pages,
        };
        let write = MediaWrite {
            title: media.title.clone(),
            status: if is_replacement {
                MediaStatus::Ready
            } else {
                MediaStatus::Processing
            },
            content: media.content.clone(),
            pending_revision: Some(pending),
            declared_kind: media.declared_kind,
        };
        self.media_repo.update_unscoped(media_id, write).await?;
        self.coordinator
            .register_source(&revision_id, staging_op)
            .await;
        self.spawn_deck_job(
            media_id.to_owned(),
            revision_id,
            staging_op.to_owned(),
            replace_page_id,
        );
        Ok(())
    }

    fn spawn_deck_job(
        self: &Arc<Self>,
        media_id: String,
        revision_id: String,
        staging_op: String,
        replace_page_id: Option<String>,
    ) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(err) = this
                .run_deck_job(
                    &media_id,
                    &revision_id,
                    &staging_op,
                    replace_page_id.as_deref(),
                )
                .await
            {
                tracing::warn!(
                    media_id = %media_id,
                    revision_id = %revision_id,
                    error = %err,
                    "deck processing job failed internally"
                );
            }
        });
    }

    async fn run_deck_job(
        &self,
        media_id: &str,
        revision_id: &str,
        staging_op: &str,
        replace_page_id: Option<&str>,
    ) -> Result<(), AppError> {
        let cancel = self
            .coordinator
            .cancel_tokens
            .read()
            .await
            .get(staging_op)
            .cloned();
        if cancel.as_ref().is_some_and(|t| t.is_cancelled()) {
            self.cleanup_stale(staging_op).await;
            self.coordinator
                .unregister_source(revision_id, staging_op)
                .await;
            return Ok(());
        }
        let result = self
            .process_deck_source(media_id, revision_id, staging_op, replace_page_id)
            .await;
        self.coordinator
            .unregister_source(revision_id, staging_op)
            .await;
        result
    }

    async fn process_deck_source(
        &self,
        media_id: &str,
        revision_id: &str,
        staging_op: &str,
        replace_page_id: Option<&str>,
    ) -> Result<(), AppError> {
        let media = self.media_repo.get_unscoped(media_id).await?;
        if !operation_matches(&media, revision_id) {
            self.cleanup_stale(staging_op).await;
            return Ok(());
        }
        let staging = match self.asset_svc.get_staging_by_operation(staging_op).await {
            Ok(s) => s,
            Err(_) => {
                self.fail_deck_source(
                    &media,
                    revision_id,
                    staging_op,
                    AvProcessFailure::InputInvalid,
                )
                .await?;
                return Ok(());
            }
        };
        let staging_path = self.asset_svc.staging_path(staging_op);
        let current_pages = media
            .pending_revision
            .as_ref()
            .map(|p| p.pages.len())
            .unwrap_or(0);
        let occupied = if replace_page_id.is_some() {
            current_pages.saturating_sub(1)
        } else {
            current_pages
        };
        let remaining = self.deck_max_pages.saturating_sub(occupied);
        let expanded = match self
            .deck_processor
            .expand_source(&staging_path, staging.kind, &self.work_parent, remaining)
            .await
        {
            Ok(result) => result,
            Err(failure) => {
                self.fail_deck_source(&media, revision_id, staging_op, failure)
                    .await?;
                self.cleanup_staging(staging_op, &staging.id).await;
                return Ok(());
            }
        };

        if !operation_still_matches(media_id, revision_id, &self.media_repo).await? {
            self.cleanup_stale(staging_op).await;
            return Ok(());
        }

        let owner = parse_owner_record_id(&media.owner)?;
        let media_rid = RecordId::new("media", media_id.to_owned());
        let mut ingested = Vec::new();
        for page in &expanded.pages {
            match self
                .asset_svc
                .ingest_processed_file(
                    owner.clone(),
                    media_rid.clone(),
                    page.kind,
                    page.content_type.into(),
                    &page.path,
                )
                .await
            {
                Ok(asset) => ingested.push(MediaStagedDeckPage {
                    id: uuid::Uuid::new_v4().to_string(),
                    blob_id: asset.id,
                }),
                Err(_) => {
                    for page in &ingested {
                        self.asset_svc.delete_final_file(&page.blob_id);
                        let _ = self.asset_svc.delete_asset_record(&page.blob_id).await;
                    }
                    self.fail_deck_source(
                        &media,
                        revision_id,
                        staging_op,
                        AvProcessFailure::Failed,
                    )
                    .await?;
                    self.cleanup_staging(staging_op, &staging.id).await;
                    return Ok(());
                }
            }
        }

        let _guard = self.deck_write.lock().await;
        let latest = self.media_repo.get_unscoped(media_id).await?;
        if !operation_matches(&latest, revision_id) {
            for page in &ingested {
                self.asset_svc.delete_final_file(&page.blob_id);
                let _ = self.asset_svc.delete_asset_record(&page.blob_id).await;
            }
            self.cleanup_stale(staging_op).await;
            return Ok(());
        }
        let mut pages = latest
            .pending_revision
            .as_ref()
            .map(|p| p.pages.clone())
            .unwrap_or_default();
        if let Some(page_id) = replace_page_id {
            if let Some(index) = pages.iter().position(|p| p.id == page_id) {
                let old = pages.remove(index);
                if !content_blob_ids(latest.content.as_ref()).contains(&old.blob_id)
                    && !pages.iter().any(|p| p.blob_id == old.blob_id)
                {
                    self.asset_svc.delete_final_file(&old.blob_id);
                    let _ = self.asset_svc.delete_asset_record(&old.blob_id).await;
                }
                for (offset, page) in ingested.into_iter().enumerate() {
                    pages.insert(index + offset, page);
                }
            } else {
                pages.extend(ingested);
            }
        } else {
            pages.extend(ingested);
        }
        if pages.len() > self.deck_max_pages {
            self.fail_deck_source(
                &latest,
                revision_id,
                staging_op,
                AvProcessFailure::InputUnsupported,
            )
            .await?;
            self.cleanup_staging(staging_op, &staging.id).await;
            return Ok(());
        }
        let in_flight = self
            .coordinator
            .other_sources_in_flight(revision_id, staging_op)
            .await;
        let pending = MediaPendingRevision {
            operation: revision_id.to_owned(),
            status: if in_flight {
                MediaStatus::Processing
            } else {
                MediaStatus::Ready
            },
            processing_error: None,
            pages,
        };
        let write = MediaWrite {
            title: latest.title.clone(),
            status: latest.status,
            content: latest.content.clone(),
            pending_revision: Some(pending),
            declared_kind: latest.declared_kind,
        };
        if !self
            .commit_if_operation(media_id, revision_id, write)
            .await?
        {
            self.cleanup_stale(staging_op).await;
            return Ok(());
        }
        self.cleanup_staging(staging_op, &staging.id).await;
        Ok(())
    }

    async fn fail_deck_source(
        &self,
        media: &Media,
        revision_id: &str,
        staging_op: &str,
        failure: AvProcessFailure,
    ) -> Result<(), AppError> {
        let err = processing_error(failure);
        let latest = self.media_repo.get_unscoped(&media.id).await?;
        if !operation_matches(&latest, revision_id) {
            return Ok(());
        }
        let pages = latest
            .pending_revision
            .as_ref()
            .map(|p| p.pages.clone())
            .unwrap_or_default();
        let in_flight = self
            .coordinator
            .other_sources_in_flight(revision_id, staging_op)
            .await;
        let keep_draft = is_ready_deck(&latest) || !pages.is_empty();
        let pending = MediaPendingRevision {
            operation: revision_id.to_owned(),
            status: if in_flight {
                MediaStatus::Processing
            } else {
                MediaStatus::Failed
            },
            processing_error: Some(err),
            pages: if keep_draft { pages } else { Vec::new() },
        };
        let write = if is_ready_deck(&latest) {
            MediaWrite {
                title: latest.title.clone(),
                status: MediaStatus::Ready,
                content: latest.content.clone(),
                pending_revision: Some(pending),
                declared_kind: latest.declared_kind,
            }
        } else if keep_draft {
            MediaWrite {
                title: latest.title.clone(),
                status: MediaStatus::Processing,
                content: None,
                pending_revision: Some(pending),
                declared_kind: latest.declared_kind,
            }
        } else {
            MediaWrite {
                title: latest.title.clone(),
                status: MediaStatus::Failed,
                content: None,
                pending_revision: Some(pending),
                declared_kind: latest.declared_kind,
            }
        };
        self.commit_if_operation(&latest.id, revision_id, write)
            .await?;
        Ok(())
    }

    pub async fn begin_deck_revision_for_user(
        &self,
        ctx: &AuthorizationContext,
        media_id: &str,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let media = self.media_repo.get(&write_teams, media_id).await?;
        let owner = parse_owner_record_id(&media.owner)?;
        ctx.require_write_access_to_owner(&owner)?;
        if !is_ready_deck(&media) {
            return Err(AppError::invalid_request(
                "only a ready slide deck can start an edit revision",
            ));
        }
        if let Some(pending) = media.pending_revision.as_ref()
            && pending.status != MediaStatus::Failed
        {
            return Ok(media);
        }
        let pending = MediaPendingRevision {
            operation: uuid::Uuid::new_v4().to_string(),
            status: MediaStatus::Ready,
            processing_error: None,
            pages: staged_pages_from_content(media.content.as_ref()),
        };
        self.media_repo
            .update(
                &write_teams,
                media_id,
                None,
                MediaWrite {
                    title: media.title.clone(),
                    status: MediaStatus::Ready,
                    content: media.content.clone(),
                    pending_revision: Some(pending),
                    declared_kind: media.declared_kind,
                },
            )
            .await
    }

    pub async fn commit_deck_for_user(
        &self,
        ctx: &AuthorizationContext,
        media_id: &str,
        payload: CommitDeck,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let media = self.media_repo.get(&write_teams, media_id).await?;
        let owner = parse_owner_record_id(&media.owner)?;
        ctx.require_write_access_to_owner(&owner)?;
        let pending = media
            .pending_revision
            .as_ref()
            .ok_or_else(|| AppError::invalid_request("no staged slide-deck revision to commit"))?;
        if pending.operation != payload.operation {
            return Err(AppError::invalid_request("stale deck revision"));
        }
        if pending.status == MediaStatus::Processing
            || self
                .coordinator
                .revision_in_flight(&pending.operation)
                .await
        {
            return Err(AppError::invalid_request(
                "slide-deck expansion is still processing",
            ));
        }
        if payload.page_ids.is_empty() {
            return Err(AppError::invalid_request(
                "a slide deck requires at least one page",
            ));
        }
        let mut ordered = Vec::new();
        for page_id in &payload.page_ids {
            let page = pending
                .pages
                .iter()
                .find(|p| &p.id == page_id)
                .ok_or_else(|| AppError::invalid_request("unknown deck page id"))?;
            if ordered
                .iter()
                .any(|p: &MediaStagedDeckPage| p.id == page.id)
            {
                return Err(AppError::invalid_request("duplicate deck page id"));
            }
            ordered.push(page.clone());
        }
        let committed: Vec<MediaDeckPage> = ordered
            .iter()
            .map(|p| MediaDeckPage {
                blob_id: p.blob_id.clone(),
            })
            .collect();
        let keep: HashSet<String> = committed.iter().map(|p| p.blob_id.clone()).collect();
        let mut drop_ids = content_blob_ids(media.content.as_ref());
        for page in &pending.pages {
            drop_ids.insert(page.blob_id.clone());
        }
        drop_ids.retain(|id| !keep.contains(id));
        let updated = self
            .media_repo
            .update(
                &write_teams,
                media_id,
                None,
                MediaWrite {
                    title: media.title.clone(),
                    status: MediaStatus::Ready,
                    content: Some(MediaContent::SlideDeck { pages: committed }),
                    pending_revision: None,
                    declared_kind: None,
                },
            )
            .await?;
        for id in drop_ids {
            self.asset_svc.delete_final_file(&id);
            let _ = self.asset_svc.delete_asset_record(&id).await;
        }
        Ok(updated)
    }

    async fn cleanup_draft_assets(&self, media: &Media, pending: &MediaPendingRevision) {
        let keep = content_blob_ids(media.content.as_ref());
        for page in &pending.pages {
            if !keep.contains(&page.blob_id) {
                self.asset_svc.delete_final_file(&page.blob_id);
                let _ = self.asset_svc.delete_asset_record(&page.blob_id).await;
            }
        }
    }
}

fn validate_upload_kind(media: &Media, kind: MediaAssetKind) -> Result<(), AppError> {
    let declared = media
        .declared_kind
        .or_else(|| declared_from_content(&media.content));
    match declared {
        Some(DeclaredMediaKind::Video) if kind != MediaAssetKind::Video => Err(
            AppError::invalid_request("upload kind must match declared video media"),
        ),
        Some(DeclaredMediaKind::Audio) if kind != MediaAssetKind::Audio => Err(
            AppError::invalid_request("upload kind must match declared audio media"),
        ),
        Some(DeclaredMediaKind::SlideDeck) if !is_deck_asset_kind(kind) => Err(
            AppError::invalid_request("upload kind must be image, pdf, or svg for a slide deck"),
        ),
        Some(DeclaredMediaKind::SlideDeck) => Ok(()),
        None if media.content.is_some() && is_uploaded_content(media.content.as_ref().unwrap()) => {
            match media.content.as_ref() {
                Some(MediaContent::Video { .. }) if kind != MediaAssetKind::Video => Err(
                    AppError::invalid_request("upload kind must match video media"),
                ),
                Some(MediaContent::Audio { .. }) if kind != MediaAssetKind::Audio => Err(
                    AppError::invalid_request("upload kind must match audio media"),
                ),
                _ => Ok(()),
            }
        }
        None if media.declared_kind.is_none() && !is_url_content(media) => {
            Err(AppError::invalid_request("media does not accept uploads"))
        }
        None => Err(AppError::invalid_request("media does not accept uploads")),
        _ => Ok(()),
    }
}

fn declared_from_content(content: &Option<MediaContent>) -> Option<DeclaredMediaKind> {
    match content {
        Some(MediaContent::Video { .. }) => Some(DeclaredMediaKind::Video),
        Some(MediaContent::Audio { .. }) => Some(DeclaredMediaKind::Audio),
        Some(MediaContent::SlideDeck { .. }) => Some(DeclaredMediaKind::SlideDeck),
        _ => None,
    }
}

fn is_url_content(media: &Media) -> bool {
    matches!(
        media.content,
        Some(MediaContent::YouTube { .. })
            | Some(MediaContent::Livestream { .. })
            | Some(MediaContent::WebPage { .. })
    )
}

fn is_uploaded_content(content: &MediaContent) -> bool {
    matches!(
        content,
        MediaContent::Video { .. } | MediaContent::Audio { .. } | MediaContent::SlideDeck { .. }
    )
}

fn is_ready_uploaded_replacement(media: &Media) -> bool {
    media.status == MediaStatus::Ready && media.content.as_ref().is_some_and(is_uploaded_content)
}

fn is_deck_asset_kind(kind: MediaAssetKind) -> bool {
    matches!(
        kind,
        MediaAssetKind::Image | MediaAssetKind::Pdf | MediaAssetKind::Svg
    )
}

fn is_deck_media(media: &Media) -> bool {
    media.declared_kind == Some(DeclaredMediaKind::SlideDeck)
        || matches!(media.content, Some(MediaContent::SlideDeck { .. }))
}

fn is_ready_deck(media: &Media) -> bool {
    media.status == MediaStatus::Ready
        && matches!(media.content, Some(MediaContent::SlideDeck { .. }))
}

fn staged_pages_from_content(content: Option<&MediaContent>) -> Vec<MediaStagedDeckPage> {
    match content {
        Some(MediaContent::SlideDeck { pages }) => pages
            .iter()
            .map(|page| MediaStagedDeckPage {
                id: uuid::Uuid::new_v4().to_string(),
                blob_id: page.blob_id.clone(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn content_blob_ids(content: Option<&MediaContent>) -> HashSet<String> {
    match content {
        Some(MediaContent::SlideDeck { pages }) => {
            pages.iter().map(|p| p.blob_id.clone()).collect()
        }
        Some(MediaContent::Video { blob_id, .. } | MediaContent::Audio { blob_id, .. }) => {
            HashSet::from([blob_id.clone()])
        }
        _ => HashSet::new(),
    }
}

fn operation_matches(media: &Media, operation_id: &str) -> bool {
    media
        .pending_revision
        .as_ref()
        .is_some_and(|p| p.operation == operation_id)
}

async fn operation_still_matches(
    media_id: &str,
    operation_id: &str,
    repo: &SurrealMediaRepo,
) -> Result<bool, AppError> {
    let media = repo.get_unscoped(media_id).await?;
    Ok(operation_matches(&media, operation_id))
}

fn superseded_blob_id(media: &Media) -> Option<String> {
    match media.content.as_ref() {
        Some(MediaContent::Video { blob_id, .. }) => Some(blob_id.clone()),
        Some(MediaContent::Audio { blob_id, .. }) => Some(blob_id.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::time::Duration;

    use surrealdb::types::RecordId;

    use std::path::Path;

    use crate::process_runner::TempWorkDir;
    use crate::resources::media::av_processor::{ProbeResult, TranscodeResult};
    use crate::resources::media_asset::{CreateStagingAsset, MediaAssetRepository};
    use crate::test_helpers::{TeamFixture, auth_ctx_for_user, test_db};
    use shared::media::{CreateMedia, CreateMediaContent};

    struct MockAvProcessor {
        probe_fail: Option<AvProcessFailure>,
        transcode_fail: Option<AvProcessFailure>,
        output_bytes: Vec<u8>,
    }

    impl MockAvProcessor {
        fn success(bytes: &[u8]) -> Self {
            Self {
                probe_fail: None,
                transcode_fail: None,
                output_bytes: bytes.to_vec(),
            }
        }

        fn probe_fails(failure: AvProcessFailure) -> Self {
            Self {
                probe_fail: Some(failure),
                transcode_fail: None,
                output_bytes: vec![],
            }
        }
    }

    #[async_trait]
    impl AvProcessor for MockAvProcessor {
        async fn probe_input(
            &self,
            _input: &Path,
            _kind: MediaAssetKind,
        ) -> Result<ProbeResult, AvProcessFailure> {
            if let Some(failure) = self.probe_fail.clone() {
                return Err(failure);
            }
            Ok(ProbeResult {
                duration_ms: 1500,
                width: Some(640),
                height: Some(360),
                has_video: true,
                has_audio: true,
            })
        }

        async fn transcode(
            &self,
            _input: &Path,
            kind: MediaAssetKind,
            work_parent: &Path,
        ) -> Result<TranscodeResult, AvProcessFailure> {
            if let Some(failure) = self.transcode_fail.clone() {
                return Err(failure);
            }
            let work = TempWorkDir::new(work_parent).map_err(|_| AvProcessFailure::Failed)?;
            let ext = match kind {
                MediaAssetKind::Video => "mp4",
                MediaAssetKind::Audio => "m4a",
                _ => "bin",
            };
            let output_path = work.path().join(format!("output.{ext}"));
            std::fs::write(&output_path, &self.output_bytes)
                .map_err(|_| AvProcessFailure::Failed)?;
            Ok(TranscodeResult::new(
                work,
                output_path,
                1500,
                Some(640),
                Some(360),
            ))
        }
    }

    fn media_settings(dir: &tempfile::TempDir) -> Settings {
        Settings {
            media_staging_dir: dir.path().join("staging").to_string_lossy().into_owned(),
            media_final_dir: dir.path().join("final").to_string_lossy().into_owned(),
            media_processing_enabled: false,
            ..Settings::default()
        }
    }

    async fn stage_bytes(
        asset_svc: &crate::resources::media_asset::service::MediaAssetServiceHandle,
        media_id: &str,
        owner: RecordId,
        operation_id: &str,
        bytes: &[u8],
        kind: MediaAssetKind,
    ) {
        asset_svc.ensure_directories().await.unwrap();
        let staging_path = asset_svc.staging_path(operation_id);
        std::fs::create_dir_all(staging_path.parent().unwrap()).unwrap();
        std::fs::write(&staging_path, bytes).unwrap();
        let etag = crate::http_range::etag_from_file_bytes(bytes);
        let media_rid = RecordId::new("media", media_id.to_owned());
        asset_svc
            .repo
            .create_staging(CreateStagingAsset {
                owner,
                media_id: media_rid,
                kind,
                content_type: "application/octet-stream".into(),
                byte_length: bytes.len() as u64,
                operation_id: operation_id.to_owned(),
                etag,
            })
            .await
            .unwrap();
    }

    async fn wait_for_status(
        repo: &SurrealMediaRepo,
        media_id: &str,
        status: MediaStatus,
    ) -> Media {
        for _ in 0..100 {
            let media = repo.get_unscoped(media_id).await.unwrap();
            if media.status == status {
                return media;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("timed out waiting for status {status:?}");
    }

    #[test]
    fn operation_match_requires_pending_revision() {
        let media = Media {
            id: "m1".into(),
            owner: "t1".into(),
            title: "t".into(),
            status: MediaStatus::Processing,
            content: None,
            pending_revision: Some(MediaPendingRevision::processing("op1")),
            declared_kind: Some(DeclaredMediaKind::Video),
        };
        assert!(operation_matches(&media, "op1"));
        assert!(!operation_matches(&media, "op2"));
    }

    #[test]
    fn ready_uploaded_replacement_detection() {
        let media = Media {
            id: "m1".into(),
            owner: "t1".into(),
            title: "t".into(),
            status: MediaStatus::Ready,
            content: Some(MediaContent::Video {
                blob_id: "b1".into(),
                duration_ms: 1,
                width: 1,
                height: 1,
            }),
            pending_revision: None,
            declared_kind: None,
        };
        assert!(is_ready_uploaded_replacement(&media));
    }

    #[tokio::test]
    async fn initial_upload_success_becomes_ready() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processor: Arc<dyn AvProcessor> =
            Arc::new(MockAvProcessor::success(b"processed-video"));
        let processing = Arc::new(MediaProcessingHandle::build_with_processor(
            db.clone(),
            &settings,
            asset_svc.clone(),
            processor,
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );

        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Clip".into(),
                    content: CreateMediaContent::Video,
                },
            )
            .await
            .unwrap();
        assert_eq!(shell.status, MediaStatus::Processing);

        let operation_id = uuid::Uuid::new_v4().to_string();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &operation_id,
            b"source-bytes",
            MediaAssetKind::Video,
        )
        .await;

        processing
            .begin_after_upload(&shell.id, &operation_id, MediaAssetKind::Video, None)
            .await
            .unwrap();

        let repo = SurrealMediaRepo::new(db.clone());
        let ready = wait_for_status(&repo, &shell.id, MediaStatus::Ready).await;
        assert!(matches!(
            ready.content,
            Some(MediaContent::Video {
                duration_ms: 1500,
                width: 640,
                height: 360,
                ..
            })
        ));
        assert!(ready.pending_revision.is_none());
    }

    #[tokio::test]
    async fn initial_probe_failure_marks_failed() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processor: Arc<dyn AvProcessor> =
            Arc::new(MockAvProcessor::probe_fails(AvProcessFailure::InputInvalid));
        let processing = Arc::new(MediaProcessingHandle::build_with_processor(
            db.clone(),
            &settings,
            asset_svc.clone(),
            processor,
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );

        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Bad".into(),
                    content: CreateMediaContent::Audio,
                },
            )
            .await
            .unwrap();

        let operation_id = uuid::Uuid::new_v4().to_string();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &operation_id,
            b"bad",
            MediaAssetKind::Audio,
        )
        .await;

        processing
            .begin_after_upload(&shell.id, &operation_id, MediaAssetKind::Audio, None)
            .await
            .unwrap();

        let repo = SurrealMediaRepo::new(db.clone());
        let failed = wait_for_status(&repo, &shell.id, MediaStatus::Failed).await;
        assert!(failed.content.is_none());
        assert_eq!(
            failed
                .pending_revision
                .as_ref()
                .unwrap()
                .processing_error
                .as_ref()
                .unwrap()
                .code,
            "media_input_invalid"
        );
    }

    #[tokio::test]
    async fn stale_operation_completion_is_ignored() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processor: Arc<dyn AvProcessor> = Arc::new(MockAvProcessor::success(b"new"));
        let processing = Arc::new(MediaProcessingHandle::build_with_processor(
            db.clone(),
            &settings,
            asset_svc.clone(),
            processor,
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );

        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Stale".into(),
                    content: CreateMediaContent::Video,
                },
            )
            .await
            .unwrap();

        let old_op = uuid::Uuid::new_v4().to_string();
        let new_op = uuid::Uuid::new_v4().to_string();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner.clone(),
            &old_op,
            b"old",
            MediaAssetKind::Video,
        )
        .await;
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &new_op,
            b"new",
            MediaAssetKind::Video,
        )
        .await;

        processing
            .begin_after_upload(&shell.id, &old_op, MediaAssetKind::Video, None)
            .await
            .unwrap();
        processing
            .begin_after_upload(&shell.id, &new_op, MediaAssetKind::Video, None)
            .await
            .unwrap();

        let repo = SurrealMediaRepo::new(db.clone());
        let ready = wait_for_status(&repo, &shell.id, MediaStatus::Ready).await;
        assert!(ready.pending_revision.is_none());
        assert!(matches!(ready.content, Some(MediaContent::Video { .. })));
    }

    #[tokio::test]
    async fn reconcile_stranded_processing_media() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processing = Arc::new(MediaProcessingHandle::build(
            db.clone(),
            &settings,
            asset_svc.clone(),
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );

        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Stranded".into(),
                    content: CreateMediaContent::Video,
                },
            )
            .await
            .unwrap();

        let operation_id = uuid::Uuid::new_v4().to_string();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &operation_id,
            b"orphan",
            MediaAssetKind::Video,
        )
        .await;

        let repo = SurrealMediaRepo::new(db.clone());
        repo.update_unscoped(
            &shell.id,
            MediaWrite {
                title: shell.title.clone(),
                status: MediaStatus::Processing,
                content: None,
                pending_revision: Some(MediaPendingRevision::processing(operation_id.clone())),
                declared_kind: Some(DeclaredMediaKind::Video),
            },
        )
        .await
        .unwrap();

        let count = processing.reconcile_stranded().await.unwrap();
        assert_eq!(count, 1);
        let failed = repo.get_unscoped(&shell.id).await.unwrap();
        assert_eq!(failed.status, MediaStatus::Failed);
    }

    struct MockDeckProcessor {
        fail: Option<AvProcessFailure>,
        pages: usize,
    }

    impl MockDeckProcessor {
        fn pages(n: usize) -> Self {
            Self {
                fail: None,
                pages: n,
            }
        }

        fn fail(failure: AvProcessFailure) -> Self {
            Self {
                fail: Some(failure),
                pages: 0,
            }
        }
    }

    #[async_trait]
    impl DeckProcessor for MockDeckProcessor {
        async fn expand_source(
            &self,
            _input: &Path,
            _kind: MediaAssetKind,
            work_parent: &Path,
            remaining_page_budget: usize,
        ) -> Result<crate::resources::media::deck_processor::DeckExpandResult, AvProcessFailure>
        {
            if let Some(failure) = self.fail.clone() {
                return Err(failure);
            }
            if self.pages > remaining_page_budget {
                return Err(AvProcessFailure::InputUnsupported);
            }
            let work = TempWorkDir::new(work_parent).map_err(|_| AvProcessFailure::Failed)?;
            let mut pages = Vec::new();
            for i in 0..self.pages {
                let path = work.path().join(format!("page-{i}.png"));
                std::fs::write(&path, format!("page-{i}").as_bytes())
                    .map_err(|_| AvProcessFailure::Failed)?;
                pages.push(crate::resources::media::deck_processor::DeckPageOutput {
                    path,
                    content_type: "image/png",
                    kind: MediaAssetKind::Image,
                });
            }
            Ok(crate::resources::media::deck_processor::DeckExpandResult::new(pages, Some(work)))
        }
    }

    async fn wait_for_draft_pages(
        repo: &SurrealMediaRepo,
        media_id: &str,
        min_pages: usize,
    ) -> Media {
        for _ in 0..100 {
            let media = repo.get_unscoped(media_id).await.unwrap();
            let pages = media
                .pending_revision
                .as_ref()
                .map(|p| p.pages.len())
                .unwrap_or(0);
            if pages >= min_pages
                && media
                    .pending_revision
                    .as_ref()
                    .is_some_and(|p| p.status != MediaStatus::Processing)
            {
                return media;
            }
            if media.status == MediaStatus::Failed {
                return media;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("timed out waiting for draft pages");
    }

    #[tokio::test]
    async fn deck_upload_expands_then_commit_becomes_ready() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processing = Arc::new(MediaProcessingHandle::build_with_processors(
            db.clone(),
            &settings,
            asset_svc.clone(),
            Arc::new(MockAvProcessor::success(b"unused")),
            Arc::new(MockDeckProcessor::pages(2)),
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );
        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Deck".into(),
                    content: CreateMediaContent::SlideDeck,
                },
            )
            .await
            .unwrap();
        assert_eq!(shell.status, MediaStatus::Processing);
        assert_eq!(shell.declared_kind, Some(DeclaredMediaKind::SlideDeck));

        let operation_id = uuid::Uuid::new_v4().to_string();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &operation_id,
            b"source",
            MediaAssetKind::Image,
        )
        .await;
        processing
            .begin_after_upload(&shell.id, &operation_id, MediaAssetKind::Image, None)
            .await
            .unwrap();

        let repo = SurrealMediaRepo::new(db.clone());
        let draft = wait_for_draft_pages(&repo, &shell.id, 2).await;
        assert_eq!(draft.status, MediaStatus::Processing);
        let pending = draft.pending_revision.clone().unwrap();
        assert_eq!(pending.pages.len(), 2);
        assert_eq!(pending.status, MediaStatus::Ready);

        let committed = processing
            .commit_deck_for_user(
                &ctx,
                &shell.id,
                CommitDeck {
                    operation: pending.operation,
                    page_ids: pending.pages.iter().rev().map(|p| p.id.clone()).collect(),
                },
            )
            .await
            .unwrap();
        assert_eq!(committed.status, MediaStatus::Ready);
        match committed.content {
            Some(MediaContent::SlideDeck { pages }) => assert_eq!(pages.len(), 2),
            other => panic!("expected slide deck, got {other:?}"),
        }
        assert!(committed.pending_revision.is_none());
    }

    #[tokio::test]
    async fn deck_source_failure_marks_failed_without_partial_ready() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processing = Arc::new(MediaProcessingHandle::build_with_processors(
            db.clone(),
            &settings,
            asset_svc.clone(),
            Arc::new(MockAvProcessor::success(b"unused")),
            Arc::new(MockDeckProcessor::fail(AvProcessFailure::InputInvalid)),
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );
        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Bad deck".into(),
                    content: CreateMediaContent::SlideDeck,
                },
            )
            .await
            .unwrap();
        let operation_id = uuid::Uuid::new_v4().to_string();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &operation_id,
            b"bad",
            MediaAssetKind::Pdf,
        )
        .await;
        processing
            .begin_after_upload(&shell.id, &operation_id, MediaAssetKind::Pdf, None)
            .await
            .unwrap();
        let repo = SurrealMediaRepo::new(db.clone());
        let failed = wait_for_status(&repo, &shell.id, MediaStatus::Failed).await;
        assert!(failed.content.is_none());
    }

    #[tokio::test]
    async fn empty_deck_commit_is_rejected() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processing = Arc::new(MediaProcessingHandle::build_with_processors(
            db.clone(),
            &settings,
            asset_svc.clone(),
            Arc::new(MockAvProcessor::success(b"unused")),
            Arc::new(MockDeckProcessor::pages(1)),
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );
        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Empty".into(),
                    content: CreateMediaContent::SlideDeck,
                },
            )
            .await
            .unwrap();
        let err = processing
            .commit_deck_for_user(
                &ctx,
                &shell.id,
                CommitDeck {
                    operation: "missing".into(),
                    page_ids: vec![],
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn further_deck_sources_append_to_the_same_revision() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processing = Arc::new(MediaProcessingHandle::build_with_processors(
            db.clone(),
            &settings,
            asset_svc.clone(),
            Arc::new(MockAvProcessor::success(b"unused")),
            Arc::new(MockDeckProcessor::pages(1)),
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );
        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Append".into(),
                    content: CreateMediaContent::SlideDeck,
                },
            )
            .await
            .unwrap();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        let first = uuid::Uuid::new_v4().to_string();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner.clone(),
            &first,
            b"one",
            MediaAssetKind::Image,
        )
        .await;
        processing
            .begin_after_upload(&shell.id, &first, MediaAssetKind::Image, None)
            .await
            .unwrap();
        let repo = SurrealMediaRepo::new(db.clone());
        let after_first = wait_for_draft_pages(&repo, &shell.id, 1).await;
        let revision = after_first.pending_revision.clone().unwrap().operation;
        let second = uuid::Uuid::new_v4().to_string();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &second,
            b"two",
            MediaAssetKind::Pdf,
        )
        .await;
        processing
            .begin_after_upload(&shell.id, &second, MediaAssetKind::Pdf, None)
            .await
            .unwrap();
        let after_second = wait_for_draft_pages(&repo, &shell.id, 2).await;
        let pending = after_second.pending_revision.unwrap();
        assert_eq!(pending.operation, revision);
        assert_eq!(pending.pages.len(), 2);
        assert_eq!(after_second.status, MediaStatus::Processing);
    }

    #[tokio::test]
    async fn deck_stale_commit_cancel_and_begin_revision_preserve_ready() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let ctx = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let settings = media_settings(&dir);
        let asset_svc = crate::resources::media_asset::service::MediaAssetServiceHandle::build(
            db.clone(),
            &settings,
        );
        let processing = Arc::new(MediaProcessingHandle::build_with_processors(
            db.clone(),
            &settings,
            asset_svc.clone(),
            Arc::new(MockAvProcessor::success(b"unused")),
            Arc::new(MockDeckProcessor::pages(2)),
        ));
        let media_svc = crate::resources::media::service::MediaServiceHandle::build(
            db.clone(),
            asset_svc.clone(),
            processing.clone(),
        );
        let shell = media_svc
            .create_for_user(
                &ctx,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Revise".into(),
                    content: CreateMediaContent::SlideDeck,
                },
            )
            .await
            .unwrap();
        let owner = parse_owner_record_id(&fixture.shared_team_id).unwrap();
        let operation_id = uuid::Uuid::new_v4().to_string();
        stage_bytes(
            &asset_svc,
            &shell.id,
            owner,
            &operation_id,
            b"source",
            MediaAssetKind::Image,
        )
        .await;
        processing
            .begin_after_upload(&shell.id, &operation_id, MediaAssetKind::Image, None)
            .await
            .unwrap();
        let repo = SurrealMediaRepo::new(db.clone());
        let draft = wait_for_draft_pages(&repo, &shell.id, 2).await;
        let pending = draft.pending_revision.clone().unwrap();
        let stale = processing
            .commit_deck_for_user(
                &ctx,
                &shell.id,
                CommitDeck {
                    operation: "stale-op".into(),
                    page_ids: pending.pages.iter().map(|p| p.id.clone()).collect(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(stale, AppError::InvalidRequest(_)));
        let still_draft = repo.get_unscoped(&shell.id).await.unwrap();
        assert_eq!(still_draft.status, MediaStatus::Processing);
        assert!(still_draft.content.is_none());

        let committed = processing
            .commit_deck_for_user(
                &ctx,
                &shell.id,
                CommitDeck {
                    operation: pending.operation,
                    page_ids: pending.pages.iter().map(|p| p.id.clone()).collect(),
                },
            )
            .await
            .unwrap();
        let original_blobs: Vec<String> = match &committed.content {
            Some(MediaContent::SlideDeck { pages }) => {
                pages.iter().map(|p| p.blob_id.clone()).collect()
            }
            other => panic!("expected ready deck, got {other:?}"),
        };

        let revision = processing
            .begin_deck_revision_for_user(&ctx, &shell.id)
            .await
            .unwrap();
        let revision_pages = revision.pending_revision.clone().unwrap().pages;
        assert_eq!(revision_pages.len(), 2);
        assert_eq!(
            revision_pages
                .iter()
                .map(|p| p.blob_id.clone())
                .collect::<Vec<_>>(),
            original_blobs
        );
        assert_ne!(revision_pages[0].id, original_blobs[0]);

        let cancelled = processing
            .cancel_pending_for_user(&ctx, &shell.id)
            .await
            .unwrap();
        assert!(cancelled.pending_revision.is_none());
        match cancelled.content {
            Some(MediaContent::SlideDeck { pages }) => {
                assert_eq!(
                    pages.iter().map(|p| p.blob_id.clone()).collect::<Vec<_>>(),
                    original_blobs
                );
            }
            other => panic!("expected ready deck after cancel, got {other:?}"),
        }
    }
}
