//! Request-bound uploaded-media processing.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use shared::MediaAssetKind;
use shared::media::{
    CommitDeck, CreateUploadedMedia, Media, MediaContent, MediaDeckPage, MediaPendingRevision,
    MediaStagedDeckPage, UploadedMediaKind,
};
use surrealdb::types::RecordId;
use tokio::sync::{Mutex, RwLock};

use crate::auth::AuthorizationContext;
use crate::error::AppError;
use crate::resources::media::av_processor::{
    AvProcessor, FfmpegAvProcessor, app_error_from_failure,
};
use crate::resources::media::deck_processor::{
    DeckProcessor, LopdfDeckProcessor, UnsupportedDeckProcessor,
};
use crate::resources::media::model::MediaWrite;
use crate::resources::media::repository::MediaRepository;
use crate::resources::media::surreal_repo::SurrealMediaRepo;
use crate::resources::team::parse_owner_record_id;
use crate::settings::Settings;

#[derive(Debug, Clone)]
pub struct UploadedSource {
    pub path: PathBuf,
    pub kind: MediaAssetKind,
}

struct FinalAssetCleanup {
    asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
    asset_ids: Vec<String>,
}

impl FinalAssetCleanup {
    fn new(asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle) -> Self {
        Self {
            asset_svc,
            asset_ids: Vec::new(),
        }
    }

    fn track(&mut self, asset_id: String) {
        self.asset_ids.push(asset_id);
    }

    fn disarm(&mut self) {
        self.asset_ids.clear();
    }
}

impl Drop for FinalAssetCleanup {
    fn drop(&mut self) {
        let asset_ids = std::mem::take(&mut self.asset_ids);
        if asset_ids.is_empty() {
            return;
        }
        for asset_id in &asset_ids {
            self.asset_svc.delete_final_file(asset_id);
        }
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let asset_svc = self.asset_svc.clone();
            runtime.spawn(async move {
                for asset_id in asset_ids {
                    let _ = asset_svc.delete_asset_record(&asset_id).await;
                }
            });
        }
    }
}

struct StagingAssetCleanup {
    asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
    operation_id: Option<String>,
}

impl StagingAssetCleanup {
    fn new(
        asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
        operation_id: &str,
    ) -> Self {
        Self {
            asset_svc,
            operation_id: Some(operation_id.to_owned()),
        }
    }

    async fn cleanup_now(&mut self) {
        let Some(operation_id) = self.operation_id.take() else {
            return;
        };
        self.asset_svc.delete_staging_file(&operation_id);
        if let Ok(staging) = self.asset_svc.get_staging_by_operation(&operation_id).await {
            let _ = self.asset_svc.delete_asset_record(&staging.id).await;
        }
    }
}

impl Drop for StagingAssetCleanup {
    fn drop(&mut self) {
        let Some(operation_id) = self.operation_id.take() else {
            return;
        };
        self.asset_svc.delete_staging_file(&operation_id);
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let asset_svc = self.asset_svc.clone();
            runtime.spawn(async move {
                if let Ok(staging) = asset_svc.get_staging_by_operation(&operation_id).await {
                    let _ = asset_svc.delete_asset_record(&staging.id).await;
                }
            });
        }
    }
}

#[derive(Clone)]
pub struct MediaProcessingHandle {
    media_repo: SurrealMediaRepo,
    asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
    processor: Arc<dyn AvProcessor>,
    deck_processor: Arc<dyn DeckProcessor>,
    work_parent: PathBuf,
    deck_max_pages: usize,
    media_locks: Arc<RwLock<HashMap<String, Arc<Mutex<()>>>>>,
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
        Self::build_with_processors(
            db,
            settings,
            asset_svc,
            processor,
            Arc::new(LopdfDeckProcessor {
                timeout: settings.media_processing_timeout(),
            }),
        )
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
        Self {
            media_repo: SurrealMediaRepo::new(db),
            asset_svc,
            processor,
            deck_processor,
            work_parent: std::env::temp_dir().join("worshipviewer_media_work"),
            deck_max_pages: settings.media_deck_max_pages as usize,
            media_locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn media_lock(&self, media_id: &str) -> Arc<Mutex<()>> {
        if let Some(lock) = self.media_locks.read().await.get(media_id).cloned() {
            return lock;
        }
        self.media_locks
            .write()
            .await
            .entry(media_id.to_owned())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    pub async fn create_uploaded_for_user(
        &self,
        ctx: &AuthorizationContext,
        metadata: CreateUploadedMedia,
        kind: UploadedMediaKind,
        sources: Vec<UploadedSource>,
    ) -> Result<Media, AppError> {
        validate_source_count(kind, &sources)?;
        let owner = match metadata.owner {
            None => ctx.personal_team()?,
            Some(value) => {
                let owner = parse_owner_record_id(&value)?;
                ctx.require_write_access_to_owner(&owner)?;
                owner
            }
        };
        let title = checked_title(metadata.title)?;
        let media_id = uuid::Uuid::new_v4().to_string();
        let mut cleanup = FinalAssetCleanup::new(self.asset_svc.clone());
        let content = self
            .process_new_content(&media_id, owner.clone(), kind, &sources, &mut cleanup)
            .await?;
        let media = self
            .media_repo
            .create_with_id(
                &media_id,
                owner,
                MediaWrite {
                    title,
                    content,
                    pending_revision: None,
                },
            )
            .await?;
        cleanup.disarm();
        Ok(media)
    }

    async fn process_new_content(
        &self,
        media_id: &str,
        owner: RecordId,
        kind: UploadedMediaKind,
        sources: &[UploadedSource],
        cleanup: &mut FinalAssetCleanup,
    ) -> Result<MediaContent, AppError> {
        match kind {
            UploadedMediaKind::Video | UploadedMediaKind::Audio => {
                let expected = if kind == UploadedMediaKind::Video {
                    MediaAssetKind::Video
                } else {
                    MediaAssetKind::Audio
                };
                if sources[0].kind != expected {
                    return Err(AppError::invalid_request(
                        "upload kind does not match the source",
                    ));
                }
                self.process_av_source(media_id, owner, expected, &sources[0].path, cleanup)
                    .await
            }
            UploadedMediaKind::SlideDeck => {
                let mut pages = Vec::new();
                for source in sources {
                    let expanded = self
                        .deck_processor
                        .expand_source(
                            &source.path,
                            source.kind,
                            &self.work_parent,
                            self.deck_max_pages.saturating_sub(pages.len()),
                        )
                        .await
                        .map_err(app_error_from_failure)?;
                    for page in &expanded.pages {
                        let asset = self
                            .asset_svc
                            .ingest_processed_file(
                                owner.clone(),
                                RecordId::new("media", media_id.to_owned()),
                                page.kind,
                                page.content_type.into(),
                                &page.path,
                            )
                            .await?;
                        cleanup.track(asset.id.clone());
                        pages.push(MediaDeckPage { blob_id: asset.id });
                    }
                }
                if pages.is_empty() || pages.len() > self.deck_max_pages {
                    return Err(AppError::invalid_request(
                        "a slide deck requires between 1 and the configured maximum number of pages",
                    ));
                }
                Ok(MediaContent::SlideDeck { pages })
            }
        }
    }

    async fn process_av_source(
        &self,
        media_id: &str,
        owner: RecordId,
        kind: MediaAssetKind,
        source: &Path,
        cleanup: &mut FinalAssetCleanup,
    ) -> Result<MediaContent, AppError> {
        self.processor
            .probe_input(source, kind)
            .await
            .map_err(app_error_from_failure)?;
        let output = self
            .processor
            .transcode(source, kind, &self.work_parent)
            .await
            .map_err(app_error_from_failure)?;
        let content_type = match kind {
            MediaAssetKind::Video => "video/mp4",
            MediaAssetKind::Audio => "audio/mp4",
            _ => {
                return Err(AppError::invalid_request(
                    "unsupported audio/video upload kind",
                ));
            }
        };
        let asset = self
            .asset_svc
            .ingest_processed_file(
                owner,
                RecordId::new("media", media_id.to_owned()),
                kind,
                content_type.into(),
                &output.output_path,
            )
            .await?;
        cleanup.track(asset.id.clone());
        let content = match kind {
            MediaAssetKind::Video => MediaContent::Video {
                blob_id: asset.id.clone(),
                duration_ms: output.duration_ms,
                width: output.width.unwrap_or(0),
                height: output.height.unwrap_or(0),
            },
            MediaAssetKind::Audio => MediaContent::Audio {
                blob_id: asset.id.clone(),
                duration_ms: output.duration_ms,
            },
            _ => unreachable!(),
        };
        Ok(content)
    }

    pub async fn replace_after_upload_for_user(
        &self,
        ctx: &AuthorizationContext,
        media_id: &str,
        operation_id: &str,
        kind: MediaAssetKind,
        replace_page_id: Option<&str>,
    ) -> Result<Media, AppError> {
        let mut cleanup = StagingAssetCleanup::new(self.asset_svc.clone(), operation_id);
        let lock = self.media_lock(media_id).await;
        let _guard = lock.lock().await;
        let result = self
            .replace_after_upload_locked(ctx, media_id, operation_id, kind, replace_page_id)
            .await;
        cleanup.cleanup_now().await;
        result
    }

    async fn replace_after_upload_locked(
        &self,
        ctx: &AuthorizationContext,
        media_id: &str,
        operation_id: &str,
        kind: MediaAssetKind,
        replace_page_id: Option<&str>,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let media = self.media_repo.get(&write_teams, media_id).await?;
        let owner = parse_owner_record_id(&media.owner)?;
        ctx.require_write_access_to_owner(&owner)?;
        let staging = self
            .asset_svc
            .get_staging_by_operation(operation_id)
            .await?;
        if staging.kind != kind || staging.media_id != media_id {
            return Err(AppError::invalid_request(
                "staged upload does not match media",
            ));
        }
        let staging_path = self.asset_svc.staging_path(operation_id);
        match (&media.content, kind) {
            (MediaContent::Video { .. }, MediaAssetKind::Video)
            | (MediaContent::Audio { .. }, MediaAssetKind::Audio) => {
                if replace_page_id.is_some() {
                    return Err(AppError::invalid_request(
                        "replace_page is only valid for slide decks",
                    ));
                }
                let mut cleanup = FinalAssetCleanup::new(self.asset_svc.clone());
                let content = self
                    .process_av_source(media_id, owner, kind, &staging_path, &mut cleanup)
                    .await?;
                let old_ids = content_blob_ids(&media.content);
                let updated = self
                    .media_repo
                    .update(
                        &write_teams,
                        media_id,
                        None,
                        MediaWrite {
                            title: media.title,
                            content,
                            pending_revision: None,
                        },
                    )
                    .await?;
                cleanup.disarm();
                self.cleanup_final_assets(&old_ids.into_iter().collect::<Vec<_>>())
                    .await;
                Ok(updated)
            }
            (MediaContent::SlideDeck { .. }, deck_kind) if is_deck_asset_kind(deck_kind) => {
                self.add_deck_source(
                    &write_teams,
                    media,
                    owner,
                    &staging_path,
                    kind,
                    replace_page_id,
                )
                .await
            }
            _ => Err(AppError::invalid_request(
                "upload kind must match the existing media content",
            )),
        }
    }

    async fn add_deck_source(
        &self,
        write_teams: &[RecordId],
        media: Media,
        owner: RecordId,
        source: &Path,
        kind: MediaAssetKind,
        replace_page_id: Option<&str>,
    ) -> Result<Media, AppError> {
        let mut pending = media
            .pending_revision
            .clone()
            .unwrap_or_else(|| MediaPendingRevision {
                revision_id: uuid::Uuid::new_v4().to_string(),
                pages: staged_pages_from_content(&media.content),
            });
        let replace_index = match replace_page_id {
            Some(page_id) => Some(
                pending
                    .pages
                    .iter()
                    .position(|page| page.id == page_id)
                    .ok_or_else(|| AppError::invalid_request("replace_page does not exist"))?,
            ),
            None => None,
        };
        let occupied = pending
            .pages
            .len()
            .saturating_sub(usize::from(replace_index.is_some()));
        let expanded = self
            .deck_processor
            .expand_source(
                source,
                kind,
                &self.work_parent,
                self.deck_max_pages.saturating_sub(occupied),
            )
            .await
            .map_err(app_error_from_failure)?;
        let mut cleanup = FinalAssetCleanup::new(self.asset_svc.clone());
        let mut ingested = Vec::new();
        for page in &expanded.pages {
            let asset = self
                .asset_svc
                .ingest_processed_file(
                    owner.clone(),
                    RecordId::new("media", media.id.clone()),
                    page.kind,
                    page.content_type.into(),
                    &page.path,
                )
                .await?;
            cleanup.track(asset.id.clone());
            ingested.push(MediaStagedDeckPage {
                id: uuid::Uuid::new_v4().to_string(),
                blob_id: asset.id,
            });
        }
        let mut replaced_asset = None;
        if let Some(index) = replace_index {
            replaced_asset = Some(pending.pages.remove(index).blob_id);
            for (offset, page) in ingested.into_iter().enumerate() {
                pending.pages.insert(index + offset, page);
            }
        } else {
            pending.pages.extend(ingested);
        }
        let updated = self
            .media_repo
            .update(
                write_teams,
                &media.id,
                None,
                MediaWrite {
                    title: media.title,
                    content: media.content.clone(),
                    pending_revision: Some(pending),
                },
            )
            .await?;
        cleanup.disarm();
        if let Some(asset_id) = replaced_asset
            && !content_blob_ids(&media.content).contains(&asset_id)
        {
            self.cleanup_final_assets(&[asset_id]).await;
        }
        Ok(updated)
    }

    pub async fn begin_deck_revision_for_user(
        &self,
        ctx: &AuthorizationContext,
        media_id: &str,
    ) -> Result<Media, AppError> {
        let lock = self.media_lock(media_id).await;
        let _guard = lock.lock().await;
        let write_teams = ctx.write_teams();
        let media = self.media_repo.get(&write_teams, media_id).await?;
        let owner = parse_owner_record_id(&media.owner)?;
        ctx.require_write_access_to_owner(&owner)?;
        if !matches!(media.content, MediaContent::SlideDeck { .. }) {
            return Err(AppError::invalid_request(
                "only a slide deck can start an edit revision",
            ));
        }
        if media.pending_revision.is_some() {
            return Ok(media);
        }
        self.media_repo
            .update(
                &write_teams,
                media_id,
                None,
                MediaWrite {
                    title: media.title,
                    content: media.content.clone(),
                    pending_revision: Some(MediaPendingRevision {
                        revision_id: uuid::Uuid::new_v4().to_string(),
                        pages: staged_pages_from_content(&media.content),
                    }),
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
        let lock = self.media_lock(media_id).await;
        let _guard = lock.lock().await;
        let write_teams = ctx.write_teams();
        let media = self.media_repo.get(&write_teams, media_id).await?;
        let owner = parse_owner_record_id(&media.owner)?;
        ctx.require_write_access_to_owner(&owner)?;
        let pending = media
            .pending_revision
            .as_ref()
            .ok_or_else(|| AppError::invalid_request("no staged slide-deck revision to commit"))?;
        if pending.revision_id != payload.revision_id {
            return Err(AppError::invalid_request("stale deck revision"));
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
                .find(|page| &page.id == page_id)
                .ok_or_else(|| AppError::invalid_request("unknown deck page id"))?;
            if ordered
                .iter()
                .any(|candidate: &MediaStagedDeckPage| candidate.id == page.id)
            {
                return Err(AppError::invalid_request("duplicate deck page id"));
            }
            ordered.push(page.clone());
        }
        let committed = ordered
            .iter()
            .map(|page| MediaDeckPage {
                blob_id: page.blob_id.clone(),
            })
            .collect::<Vec<_>>();
        let keep = committed
            .iter()
            .map(|page| page.blob_id.clone())
            .collect::<HashSet<_>>();
        let mut drop_ids = content_blob_ids(&media.content);
        drop_ids.extend(pending.pages.iter().map(|page| page.blob_id.clone()));
        drop_ids.retain(|id| !keep.contains(id));
        let updated = self
            .media_repo
            .update(
                &write_teams,
                media_id,
                None,
                MediaWrite {
                    title: media.title,
                    content: MediaContent::SlideDeck { pages: committed },
                    pending_revision: None,
                },
            )
            .await?;
        self.cleanup_final_assets(&drop_ids.into_iter().collect::<Vec<_>>())
            .await;
        Ok(updated)
    }

    pub async fn delete_for_user(
        &self,
        ctx: &AuthorizationContext,
        media_id: &str,
    ) -> Result<Media, AppError> {
        let lock = self.media_lock(media_id).await;
        let _guard = lock.lock().await;
        let write_teams = ctx.write_teams();
        self.media_repo.get(&write_teams, media_id).await?;
        self.asset_svc.delete_assets_for_media(media_id).await?;
        self.media_repo.delete(&write_teams, media_id).await
    }

    async fn cleanup_final_assets(&self, asset_ids: &[String]) {
        for asset_id in asset_ids {
            self.asset_svc.delete_final_file(asset_id);
            let _ = self.asset_svc.delete_asset_record(asset_id).await;
        }
    }
}

fn checked_title(title: String) -> Result<String, AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::invalid_request("title must not be empty"));
    }
    Ok(title.to_owned())
}

fn validate_source_count(
    kind: UploadedMediaKind,
    sources: &[UploadedSource],
) -> Result<(), AppError> {
    match kind {
        UploadedMediaKind::Video | UploadedMediaKind::Audio if sources.len() != 1 => Err(
            AppError::invalid_request("audio and video creation require exactly one file"),
        ),
        UploadedMediaKind::SlideDeck if sources.is_empty() => Err(AppError::invalid_request(
            "a slide deck requires at least one file",
        )),
        _ => Ok(()),
    }
}

fn is_deck_asset_kind(kind: MediaAssetKind) -> bool {
    matches!(
        kind,
        MediaAssetKind::Image | MediaAssetKind::Pdf | MediaAssetKind::Svg
    )
}

fn staged_pages_from_content(content: &MediaContent) -> Vec<MediaStagedDeckPage> {
    match content {
        MediaContent::SlideDeck { pages } => pages
            .iter()
            .map(|page| MediaStagedDeckPage {
                id: uuid::Uuid::new_v4().to_string(),
                blob_id: page.blob_id.clone(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn content_blob_ids(content: &MediaContent) -> HashSet<String> {
    match content {
        MediaContent::SlideDeck { pages } => {
            pages.iter().map(|page| page.blob_id.clone()).collect()
        }
        MediaContent::Video { blob_id, .. } | MediaContent::Audio { blob_id, .. } => {
            HashSet::from([blob_id.clone()])
        }
        _ => HashSet::new(),
    }
}
