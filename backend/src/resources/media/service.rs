use std::sync::Arc;

use reqwest::Url;
use shared::MoveOwner;
use shared::api::ListQuery;
use shared::media::{
    CreateMedia, CreateMediaContent, DeclaredMediaKind, DuplicateMedia, LivestreamType, Media,
    MediaContent, MediaStatus, UpdateMedia,
};
use tracing::instrument;

use crate::auth::AuthorizationContext;
use crate::database::Database;
use crate::error::AppError;
use crate::resources::common::{read_teams_for_query, resolve_owner_team};
use crate::resources::media::processing::MediaProcessingHandle;
use crate::resources::media_asset::service::MediaAssetServiceHandle;
use crate::resources::team::{parse_owner_record_id, thing_record_key};

use super::model::MediaWrite;
use super::repository::MediaRepository;
use super::surreal_repo::SurrealMediaRepo;

const YOUTUBE_ID_LENGTH: usize = 11;

#[derive(Clone)]
pub struct MediaService<R> {
    pub repo: R,
    pub asset_svc: MediaAssetServiceHandle,
    pub processing: Arc<MediaProcessingHandle>,
}

impl<R> MediaService<R> {
    pub fn new(
        repo: R,
        asset_svc: MediaAssetServiceHandle,
        processing: Arc<MediaProcessingHandle>,
    ) -> Self {
        Self {
            repo,
            asset_svc,
            processing,
        }
    }
}

impl<R: MediaRepository> MediaService<R> {
    #[instrument(level = "debug", err, skip(self, ctx))]
    pub async fn list_for_user(
        &self,
        ctx: &AuthorizationContext,
        query: ListQuery,
    ) -> Result<Vec<Media>, AppError> {
        let teams = read_teams_for_query(&ctx.read_teams(), query.team.as_deref())?;
        self.repo.list(&teams, query).await
    }

    pub async fn count_for_user(
        &self,
        ctx: &AuthorizationContext,
        query: &ListQuery,
    ) -> Result<u64, AppError> {
        let teams = read_teams_for_query(&ctx.read_teams(), query.team.as_deref())?;
        self.repo.count(&teams, query.q.as_deref()).await
    }

    pub async fn get_for_user(
        &self,
        ctx: &AuthorizationContext,
        id: &str,
    ) -> Result<Media, AppError> {
        self.repo.get(&ctx.read_teams(), id).await
    }

    #[instrument(level = "debug", err, skip(self, ctx, payload))]
    pub async fn create_for_user(
        &self,
        ctx: &AuthorizationContext,
        mut payload: CreateMedia,
    ) -> Result<Media, AppError> {
        let owner = match payload.owner.take() {
            None => ctx.personal_team()?,
            Some(value) => {
                let owner = parse_owner_record_id(&value)?;
                ctx.require_write_access_to_owner(&owner)?;
                owner
            }
        };
        let value = create_write(payload.title, payload.content)?;
        self.repo.create(owner, value).await
    }

    #[instrument(level = "debug", err, skip(self, ctx, payload))]
    pub async fn update_for_user(
        &self,
        ctx: &AuthorizationContext,
        id: &str,
        payload: UpdateMedia,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let existing = self.repo.get(&write_teams, id).await?;
        let owner = resolve_owner_team(&write_teams, payload.owner)?;
        let value = update_write(existing, payload.title, payload.content)?;
        self.repo.update(&write_teams, id, owner, value).await
    }

    #[instrument(level = "debug", err, skip(self, ctx, payload))]
    pub async fn move_for_user(
        &self,
        ctx: &AuthorizationContext,
        id: &str,
        payload: MoveOwner,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let media = self.repo.get(&write_teams, id).await?;
        let current = parse_owner_record_id(&media.owner)?;
        let destination = parse_owner_record_id(&payload.owner)?;
        ctx.require_write_access_to_owner(&current)?;
        ctx.require_write_access_to_owner(&destination)?;
        if thing_record_key(&current) == thing_record_key(&destination) {
            return Ok(media);
        }
        let moved = self
            .repo
            .move_owner(&write_teams, id, destination.clone())
            .await?;
        self.asset_svc
            .update_owner_for_media(id, destination)
            .await?;
        Ok(moved)
    }

    #[instrument(level = "debug", err, skip(self, ctx, payload))]
    pub async fn duplicate_for_user(
        &self,
        ctx: &AuthorizationContext,
        id: &str,
        payload: DuplicateMedia,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let source = self.repo.get(&write_teams, id).await?;
        let source_owner = parse_owner_record_id(&source.owner)?;
        let owner = match payload.owner {
            Some(value) => {
                let destination = parse_owner_record_id(&value)?;
                ctx.require_write_access_to_owner(&destination)?;
                destination
            }
            None => source_owner,
        };
        let source_title = source.title.clone();
        let title = checked_title(payload.title.unwrap_or(source_title))?;
        let ready_uploaded = is_ready_uploaded(&source);
        let uploaded_shell = is_uploaded_shell(&source);
        let created = if ready_uploaded {
            let shell = self
                .repo
                .create(
                    owner.clone(),
                    MediaWrite {
                        title: title.clone(),
                        status: MediaStatus::Ready,
                        content: None,
                        pending_revision: None,
                        declared_kind: None,
                    },
                )
                .await?;
            let content = self
                .asset_svc
                .duplicate_uploaded_content(
                    &source.id,
                    &shell.id,
                    source.content.as_ref().unwrap(),
                    owner,
                )
                .await?;
            self.repo
                .update_unscoped(
                    &shell.id,
                    MediaWrite {
                        title,
                        status: MediaStatus::Ready,
                        content: Some(content),
                        pending_revision: None,
                        declared_kind: None,
                    },
                )
                .await?
        } else {
            self.repo
                .create(
                    owner,
                    MediaWrite {
                        title,
                        status: source.status,
                        content: source.content.clone(),
                        pending_revision: if uploaded_shell {
                            None
                        } else {
                            source.pending_revision.clone()
                        },
                        declared_kind: source.declared_kind,
                    },
                )
                .await?
        };
        Ok(created)
    }

    pub async fn delete_for_user(
        &self,
        ctx: &AuthorizationContext,
        id: &str,
    ) -> Result<Media, AppError> {
        let write_teams = ctx.write_teams();
        let media = self.repo.get(&write_teams, id).await?;
        self.processing.cancel_for_delete(&media).await;
        self.asset_svc.delete_assets_for_media(id).await?;
        self.repo.delete(&write_teams, id).await
    }

    pub async fn cancel_processing_for_user(
        &self,
        ctx: &AuthorizationContext,
        id: &str,
    ) -> Result<Media, AppError> {
        self.processing.cancel_pending_for_user(ctx, id).await
    }
}

fn create_write(title: String, content: CreateMediaContent) -> Result<MediaWrite, AppError> {
    match content {
        CreateMediaContent::Video | CreateMediaContent::Audio => {
            processing_shell_write(title, content)
        }
        _ => ready_write(title, content),
    }
}

fn update_write(
    existing: Media,
    title: String,
    content: Option<CreateMediaContent>,
) -> Result<MediaWrite, AppError> {
    if is_uploaded_shell(&existing) || existing.declared_kind.is_some() {
        if content.is_some() {
            return Err(AppError::invalid_request(
                "uploaded media title updates cannot change content",
            ));
        }
        Ok(MediaWrite {
            title: checked_title(title)?,
            status: existing.status,
            content: existing.content,
            pending_revision: existing.pending_revision,
            declared_kind: existing.declared_kind,
        })
    } else {
        let content = content.ok_or_else(|| AppError::invalid_request("content is required"))?;
        ready_write(title, content)
    }
}

fn processing_shell_write(
    title: String,
    content: CreateMediaContent,
) -> Result<MediaWrite, AppError> {
    let declared_kind = match content {
        CreateMediaContent::Video => DeclaredMediaKind::Video,
        CreateMediaContent::Audio => DeclaredMediaKind::Audio,
        _ => return Err(AppError::invalid_request("invalid upload create content")),
    };
    Ok(MediaWrite {
        title: checked_title(title)?,
        status: MediaStatus::Processing,
        content: None,
        pending_revision: None,
        declared_kind: Some(declared_kind),
    })
}

fn is_ready_uploaded(media: &Media) -> bool {
    media.status == MediaStatus::Ready
        && media
            .content
            .as_ref()
            .is_some_and(|c| matches!(c, MediaContent::Video { .. } | MediaContent::Audio { .. }))
}

fn is_uploaded_shell(media: &Media) -> bool {
    media.declared_kind.is_some()
        || media
            .content
            .as_ref()
            .is_some_and(|c| matches!(c, MediaContent::Video { .. } | MediaContent::Audio { .. }))
}

fn ready_write(title: String, content: CreateMediaContent) -> Result<MediaWrite, AppError> {
    Ok(MediaWrite {
        title: checked_title(title)?,
        status: MediaStatus::Ready,
        content: Some(normalize_content(content)?),
        pending_revision: None,
        declared_kind: None,
    })
}

fn checked_title(title: String) -> Result<String, AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::invalid_request("title must not be empty"));
    }
    Ok(title.to_owned())
}

pub(crate) fn normalize_content(value: CreateMediaContent) -> Result<MediaContent, AppError> {
    match value {
        CreateMediaContent::Video | CreateMediaContent::Audio => Err(AppError::invalid_request(
            "uploaded content cannot be set directly",
        )),
        CreateMediaContent::YouTube { url } => normalize_youtube(&url),
        CreateMediaContent::Livestream { url } => {
            let url = normalize_https_url(&url)?;
            let stream_type = if url.path().to_ascii_lowercase().ends_with(".m3u8") {
                LivestreamType::Hls
            } else {
                LivestreamType::Direct
            };
            Ok(MediaContent::Livestream {
                url: url.to_string(),
                stream_type,
            })
        }
        CreateMediaContent::WebPage { url } => {
            let url = normalize_https_url(&url)?;
            Ok(MediaContent::WebPage {
                url: url.to_string(),
            })
        }
    }
}

fn parsed_url(raw: &str) -> Result<Url, AppError> {
    Url::parse(raw).map_err(|_| AppError::media_invalid_url("URL is malformed"))
}

fn ensure_safe_common(url: &Url) -> Result<(), AppError> {
    if url.scheme() != "https" {
        return Err(AppError::media_unsupported_url("URL must use HTTPS"));
    }
    if url.host_str().is_none() {
        return Err(AppError::media_invalid_url("URL must include a valid host"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::media_invalid_url(
            "URL credentials are not allowed",
        ));
    }
    if url.fragment().is_some() {
        return Err(AppError::media_invalid_url("URL fragments are not allowed"));
    }
    Ok(())
}

fn normalize_https_url(raw: &str) -> Result<Url, AppError> {
    let url = parsed_url(raw)?;
    ensure_safe_common(&url)?;
    Ok(url)
}

fn normalize_youtube(raw: &str) -> Result<MediaContent, AppError> {
    let url = parsed_url(raw)?;
    ensure_safe_common(&url)?;
    let host = url.host_str().unwrap().to_ascii_lowercase();
    let segments: Vec<_> = url
        .path_segments()
        .map(|v| v.filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let video_id: String = match host.as_str() {
        "youtu.be" => segments.first().map(|value| (*value).to_owned()),
        "youtube.com" | "www.youtube.com" | "m.youtube.com" | "music.youtube.com" => {
            match segments.first().copied() {
                Some("watch") | None => url
                    .query_pairs()
                    .find(|(key, _)| key == "v")
                    .map(|(_, value)| value.into_owned())
                    .as_deref()
                    .map(str::to_owned),
                Some("embed" | "shorts" | "live") => segments.get(1).copied().map(str::to_owned),
                _ => None,
            }
        }
        _ => return Err(AppError::media_unsupported_url("unsupported YouTube host")),
    }
    .ok_or_else(|| {
        AppError::media_invalid_url("YouTube URL does not contain a supported video id")
    })?;
    if video_id.len() != YOUTUBE_ID_LENGTH
        || !video_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(AppError::media_invalid_url("YouTube video id is invalid"));
    }
    Ok(MediaContent::YouTube {
        canonical_url: format!("https://www.youtube.com/watch?v={video_id}"),
        video_id,
    })
}

pub type MediaServiceHandle = MediaService<SurrealMediaRepo>;

impl MediaServiceHandle {
    pub fn build(
        db: Arc<Database>,
        asset_svc: MediaAssetServiceHandle,
        processing: Arc<MediaProcessingHandle>,
    ) -> Self {
        Self::new(SurrealMediaRepo::new(db), asset_svc, processing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{
        TeamFixture, auth_ctx_for_user, media_service, test_db, two_shared_teams_for_user,
    };

    #[test]
    fn youtube_normalization_table() {
        for raw in [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtu.be/dQw4w9WgXcQ?t=2",
            "https://m.youtube.com/shorts/dQw4w9WgXcQ",
            "https://youtube.com/embed/dQw4w9WgXcQ",
            "https://youtube.com/live/dQw4w9WgXcQ",
        ] {
            assert_eq!(
                normalize_youtube(raw).unwrap(),
                MediaContent::YouTube {
                    video_id: "dQw4w9WgXcQ".into(),
                    canonical_url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into(),
                }
            );
        }
    }

    #[test]
    fn youtube_rejects_unsafe_or_misleading_urls() {
        for raw in [
            "http://youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtube.com.evil.test/watch?v=dQw4w9WgXcQ",
            "https://user@youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtube.com/watch?v=short",
            "https://youtube.com/watch?v=dQw4w9WgXcQ#fragment",
            "not a URL",
        ] {
            assert!(normalize_youtube(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn https_media_normalizes_and_classifies() {
        assert_eq!(
            normalize_content(CreateMediaContent::Livestream {
                url: "https://example.com/live.M3U8?token=x".into()
            })
            .unwrap(),
            MediaContent::Livestream {
                url: "https://example.com/live.M3U8?token=x".into(),
                stream_type: LivestreamType::Hls,
            }
        );
        assert!(
            normalize_content(CreateMediaContent::WebPage {
                url: "javascript:alert(1)".into()
            })
            .is_err()
        );
        assert!(
            normalize_content(CreateMediaContent::WebPage {
                url: "https://user:pass@example.com/".into()
            })
            .is_err()
        );
        assert!(
            normalize_content(CreateMediaContent::Livestream {
                url: "https://example.com/live#x".into()
            })
            .is_err()
        );
    }

    fn youtube(owner: Option<String>, title: &str) -> CreateMedia {
        CreateMedia {
            owner,
            title: title.into(),
            content: CreateMediaContent::YouTube {
                url: "https://youtu.be/dQw4w9WgXcQ".into(),
            },
        }
    }

    #[tokio::test]
    async fn media_crud_search_pagination_acl_and_independent_duplicate() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let writer = auth_ctx_for_user(&db, &fixture.writer).await.unwrap();
        let guest = auth_ctx_for_user(&db, &fixture.guest).await.unwrap();
        let outsider = auth_ctx_for_user(&db, &fixture.non_member).await.unwrap();
        let platform_admin = auth_ctx_for_user(&db, &fixture.platform_admin)
            .await
            .unwrap();
        let service = media_service(&db);

        let first = service
            .create_for_user(
                &writer,
                youtube(Some(fixture.shared_team_id.clone()), "Alpha video"),
            )
            .await
            .unwrap();
        let second = service
            .create_for_user(
                &writer,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Beta stream".into(),
                    content: CreateMediaContent::Livestream {
                        url: "https://media.example/live.m3u8".into(),
                    },
                },
            )
            .await
            .unwrap();
        service
            .create_for_user(
                &writer,
                CreateMedia {
                    owner: Some(fixture.shared_team_id.clone()),
                    title: "Gamma page".into(),
                    content: CreateMediaContent::WebPage {
                        url: "https://example.com/info".into(),
                    },
                },
            )
            .await
            .unwrap();

        assert_eq!(
            service.get_for_user(&guest, &first.id).await.unwrap(),
            first
        );
        assert!(matches!(
            service.get_for_user(&outsider, &first.id).await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service.get_for_user(&platform_admin, &first.id).await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service
                .create_for_user(
                    &guest,
                    youtube(Some(fixture.shared_team_id.clone()), "Denied")
                )
                .await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service
                .update_for_user(
                    &guest,
                    &first.id,
                    UpdateMedia {
                        title: "Denied".into(),
                        content: Some(CreateMediaContent::WebPage {
                            url: "https://example.com/".into()
                        }),
                        owner: None,
                    }
                )
                .await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service
                .move_for_user(
                    &guest,
                    &first.id,
                    MoveOwner {
                        owner: fixture.shared_team_id.clone()
                    }
                )
                .await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service
                .duplicate_for_user(&guest, &first.id, DuplicateMedia::default())
                .await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            service.delete_for_user(&guest, &first.id).await,
            Err(AppError::NotFound(_))
        ));

        let page = service
            .list_for_user(
                &guest,
                ListQuery {
                    page: Some(0),
                    page_size: Some(1),
                    q: Some("stream".into()),
                    team: Some(fixture.shared_team_id.clone()),
                },
            )
            .await
            .unwrap();
        assert_eq!(page, vec![second]);

        let copy = service
            .duplicate_for_user(
                &writer,
                &first.id,
                DuplicateMedia {
                    title: Some("Alpha copy".into()),
                    owner: None,
                },
            )
            .await
            .unwrap();
        assert_ne!(copy.id, first.id);
        assert_eq!(copy.content, first.content);

        service
            .update_for_user(
                &writer,
                &first.id,
                UpdateMedia {
                    title: "Changed original".into(),
                    content: Some(CreateMediaContent::WebPage {
                        url: "https://example.org/".into(),
                    }),
                    owner: None,
                },
            )
            .await
            .unwrap();
        service.delete_for_user(&writer, &first.id).await.unwrap();
        assert_eq!(
            service.get_for_user(&guest, &copy.id).await.unwrap().title,
            "Alpha copy"
        );
    }

    #[tokio::test]
    async fn move_requires_write_access_to_both_teams() {
        let db = test_db().await.unwrap();
        let fixture = TeamFixture::build(&db).await.unwrap();
        let (source, destination) = two_shared_teams_for_user(&db, &fixture.admin_user)
            .await
            .unwrap();
        let admin = auth_ctx_for_user(&db, &fixture.admin_user).await.unwrap();
        let service = media_service(&db);
        let media = service
            .create_for_user(&admin, youtube(Some(source), "Move me"))
            .await
            .unwrap();
        let moved = service
            .move_for_user(
                &admin,
                &media.id,
                MoveOwner {
                    owner: destination.clone(),
                },
            )
            .await
            .unwrap();
        assert_eq!(moved.owner, destination);

        let guest = auth_ctx_for_user(&db, &fixture.guest).await.unwrap();
        assert!(matches!(
            service
                .move_for_user(
                    &guest,
                    &media.id,
                    MoveOwner {
                        owner: fixture.shared_team_id
                    }
                )
                .await,
            Err(AppError::NotFound(_))
        ));
    }
}
