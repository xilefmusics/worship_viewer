//! Uploaded audio/video processing jobs, lifecycle transitions, and reconciliation.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use shared::MediaAssetKind;
use shared::media::{DeclaredMediaKind, Media, MediaContent, MediaPendingRevision, MediaStatus};
use surrealdb::types::RecordId;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::error::AppError;
use crate::resources::media::av_processor::{
    AvProcessFailure, AvProcessor, FfmpegAvProcessor, processing_error,
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
}

#[derive(Clone)]
pub struct MediaProcessingHandle {
    media_repo: SurrealMediaRepo,
    asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
    coordinator: ProcessingCoordinator,
    processor: Arc<dyn AvProcessor>,
    work_parent: PathBuf,
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
        Self::build_with_processor(db, settings, asset_svc, processor)
    }

    pub fn build_with_processor(
        db: Arc<crate::database::Database>,
        _settings: &Settings,
        asset_svc: crate::resources::media_asset::service::MediaAssetServiceHandle,
        processor: Arc<dyn AvProcessor>,
    ) -> Self {
        let work_parent = std::env::temp_dir().join("worshipviewer_media_work");
        Self {
            media_repo: SurrealMediaRepo::new(db),
            asset_svc,
            coordinator: ProcessingCoordinator::default(),
            processor,
            work_parent,
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
    ) -> Result<(), AppError> {
        let media = self.media_repo.get_unscoped(media_id).await?;
        validate_upload_kind(&media, kind)?;
        if let Some(old) = media.pending_revision.as_ref() {
            self.coordinator.cancel(&old.operation).await;
        }
        let is_replacement = is_ready_uploaded_replacement(&media);
        let pending = MediaPendingRevision {
            operation: operation_id.to_owned(),
            status: MediaStatus::Processing,
            processing_error: None,
        };
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
            self.coordinator.cancel(&pending.operation).await;
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
            self.coordinator.cancel(&pending.operation).await;
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
            let failure = processing_error(AvProcessFailure::Failed);
            let write = if media.status == MediaStatus::Ready && media.content.is_some() {
                MediaWrite {
                    title: media.title.clone(),
                    status: MediaStatus::Ready,
                    content: media.content.clone(),
                    pending_revision: Some(MediaPendingRevision {
                        operation: media
                            .pending_revision
                            .as_ref()
                            .map(|p| p.operation.clone())
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                        status: MediaStatus::Failed,
                        processing_error: Some(failure),
                    }),
                    declared_kind: media.declared_kind,
                }
            } else {
                MediaWrite {
                    title: media.title.clone(),
                    status: MediaStatus::Failed,
                    content: None,
                    pending_revision: Some(MediaPendingRevision {
                        operation: media
                            .pending_revision
                            .as_ref()
                            .map(|p| p.operation.clone())
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                        status: MediaStatus::Failed,
                        processing_error: Some(failure),
                    }),
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
        let pending = MediaPendingRevision {
            operation: operation_id.to_owned(),
            status: MediaStatus::Failed,
            processing_error: Some(err),
        };
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
        MediaContent::Video { .. } | MediaContent::Audio { .. }
    )
}

fn is_ready_uploaded_replacement(media: &Media) -> bool {
    media.status == MediaStatus::Ready && media.content.as_ref().is_some_and(is_uploaded_content)
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
            pending_revision: Some(MediaPendingRevision {
                operation: "op1".into(),
                status: MediaStatus::Processing,
                processing_error: None,
            }),
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
            .begin_after_upload(&shell.id, &operation_id, MediaAssetKind::Video)
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
            .begin_after_upload(&shell.id, &operation_id, MediaAssetKind::Audio)
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
            .begin_after_upload(&shell.id, &old_op, MediaAssetKind::Video)
            .await
            .unwrap();
        processing
            .begin_after_upload(&shell.id, &new_op, MediaAssetKind::Video)
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
                pending_revision: Some(MediaPendingRevision {
                    operation: operation_id.clone(),
                    status: MediaStatus::Processing,
                    processing_error: None,
                }),
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
}
