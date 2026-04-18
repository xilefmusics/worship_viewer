use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::resources::blob::PatchBlob;
use crate::resources::collection::PatchCollection;
use crate::resources::setlist::PatchSetlist;
use crate::resources::song::{PatchSong, PatchSongData};
use crate::resources::user::Role;
use crate::resources::{
    Blob, Collection, CreateBlob, CreateCollection, CreateSetlist, CreateSong, CreateUserRequest,
    Session, Setlist, Song, User,
};
use shared::api::{SongListQuery, SongSort};
use shared::auth::otp::{OtpRequest, OtpVerify};
use shared::blob::FileType;
pub use shared::error::{ErrorResponse, ProblemDetails};
use shared::like::LikeStatus;
use shared::player::{Orientation, Player, PlayerItem, ScrollType, TocItem};
use shared::song::{Link as SongLink, SongUserSpecificAddons};
use shared::team::{
    CreateTeam, PatchTeam, Team, TeamInvitation, TeamMember, TeamMemberInput, TeamRole, TeamUser,
    TeamUserRef, UpdateTeam,
};

pub mod rest {
    use super::{ApiDoc, OpenApi};
    use utoipa_swagger_ui::SwaggerUi;

    pub fn scope() -> SwaggerUi {
        SwaggerUi::new("/api/docs/{_:.*}").url("/api/docs/openapi.json", ApiDoc::openapi())
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Worship Viewer API",
        version = "1.0.0",
        description = "Versioned REST API under `/api/v1`. Authentication flows live at `/auth/*` (unversioned); clients should treat that split as stable for this major API generation.\n\n\
            **CSRF:** Cookie sessions use `SameSite=Lax`; state-changing methods are `POST`/`PUT`/`PATCH`/`DELETE` (not `GET`). Cross-site simple requests cannot mutate state via cookies under typical browser rules. Browser `fetch` from the SPA should use `credentials: 'same-origin'` (or include cookies only on same-site requests). API clients using bearer tokens should still avoid exposing tokens to third-party origins.\n\n\
            **Errors:** Error responses use `application/problem+json` ([RFC 7807](https://www.rfc-editor.org/rfc/rfc7807)) with `type`, `title`, `status`, `detail`, and `code`.\n\n\
            **Examples:** See schema `example` fields on core DTOs in the components section.",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
    servers(
        (url = "/", description = "Same origin as the web app (override per deployment).")
    ),
    paths(
        crate::auth::oidc::rest::login,
        crate::auth::oidc::rest::callback,
        crate::auth::otp::rest::otp_request,
        crate::auth::otp::rest::otp_verify,
        crate::auth::rest::logout,
        crate::resources::user::rest::get_users_me,
        crate::resources::user::rest::get_users,
        crate::resources::user::rest::get_user,
        crate::resources::user::rest::create_user,
        crate::resources::user::rest::delete_user,
        crate::resources::user::session::rest::get_sessions_for_current_user,
        crate::resources::user::session::rest::get_session_for_current_user,
        crate::resources::user::session::rest::delete_session_for_current_user,
        crate::resources::user::session::rest::get_sessions_for_user,
        crate::resources::user::session::rest::get_session_for_user,
        crate::resources::user::session::rest::create_session_for_user,
        crate::resources::user::session::rest::delete_session_for_user,
        crate::resources::song::rest::get_songs,
        crate::resources::song::rest::get_song,
        crate::resources::song::rest::get_song_player,
        crate::resources::song::rest::create_song,
        crate::resources::song::rest::update_song,
        crate::resources::song::rest::patch_song,
        crate::resources::song::rest::delete_song,
        crate::resources::song::rest::get_song_like_status,
        crate::resources::song::rest::put_song_like,
        crate::resources::song::rest::delete_song_like,
        crate::resources::collection::rest::get_collections,
        crate::resources::collection::rest::get_collection,
        crate::resources::collection::rest::get_collection_player,
        crate::resources::collection::rest::get_collection_songs,
        crate::resources::collection::rest::create_collection,
        crate::resources::collection::rest::update_collection,
        crate::resources::collection::rest::patch_collection,
        crate::resources::collection::rest::delete_collection,
        crate::resources::blob::rest::get_blobs,
        crate::resources::blob::rest::get_blob,
        crate::resources::blob::rest::create_blob,
        crate::resources::blob::rest::update_blob,
        crate::resources::blob::rest::patch_blob,
        crate::resources::blob::rest::delete_blob,
        crate::resources::blob::rest::download_blob_image,
        crate::resources::blob::rest::upload_blob_data,
        crate::resources::setlist::rest::get_setlists,
        crate::resources::setlist::rest::get_setlist,
        crate::resources::setlist::rest::get_setlist_player,
        crate::resources::setlist::rest::get_setlist_songs,
        crate::resources::setlist::rest::create_setlist,
        crate::resources::setlist::rest::update_setlist,
        crate::resources::setlist::rest::patch_setlist,
        crate::resources::setlist::rest::delete_setlist,
        crate::resources::team::rest::get_teams,
        crate::resources::team::rest::get_team,
        crate::resources::team::rest::create_team,
        crate::resources::team::rest::update_team,
        crate::resources::team::rest::patch_team,
        crate::resources::team::rest::delete_team,
        crate::resources::team::invitation::rest::create_team_invitation,
        crate::resources::team::invitation::rest::list_team_invitations,
        crate::resources::team::invitation::rest::get_team_invitation,
        crate::resources::team::invitation::rest::delete_team_invitation,
        crate::resources::team::invitation::rest::accept_team_invitation
    ),
    components(
        schemas(
            User,
            Session,
            Role,
            CreateUserRequest,
            OtpRequest,
            OtpVerify,
            SongListQuery,
            SongSort,
            ErrorResponse,
            ProblemDetails,
            Song,
            CreateSong,
            PatchSong,
            PatchSongData,
            SongUserSpecificAddons,
            Collection,
            CreateCollection,
            PatchCollection,
            Setlist,
            CreateSetlist,
            PatchSetlist,
            Blob,
            CreateBlob,
            PatchBlob,
            FileType,
            SongLink,
            LikeStatus,
            Player,
            PlayerItem,
            TocItem,
            ScrollType,
            Orientation,
            Team,
            TeamMember,
            TeamRole,
            TeamUser,
            TeamUserRef,
            CreateTeam,
            UpdateTeam,
            PatchTeam,
            TeamMemberInput,
            TeamInvitation
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Users", description = "User resources"),
        (name = "Songs", description = "Song resources"),
        (name = "Collections", description = "Collection resources"),
        (name = "Blobs", description = "Blob resources"),
        (name = "Setlists", description = "Setlist resources"),
        (name = "Teams", description = "Team resources")
    ),
    modifiers(&SessionSecurity)
)]
pub struct ApiDoc;

struct SessionSecurity;

impl Modify for SessionSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "SessionCookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                "sso_session",
                "Session cookie returned after a successful authentication flow",
            ))),
        );
        components.add_security_scheme(
            "SessionToken",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "Authorization",
                "Session override using `Authorization: Bearer <session>` header",
            ))),
        );
    }
}
