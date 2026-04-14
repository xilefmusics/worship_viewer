use std::sync::{Arc, Once};

use actix_web::web::Data;
use anyhow::Result as AnyResult;
use chordlib::types::Song as SongData;

use crate::database::Database;
use crate::resources::blob::service::BlobServiceHandle;
use crate::resources::collection::service::CollectionServiceHandle;
use crate::resources::setlist::{SetlistService, SetlistServiceHandle, SurrealSetlistRepo};
use crate::resources::song::service::SongServiceHandle;
use crate::resources::team::SurrealTeamResolver;
use crate::resources::{User, UserModel};
use crate::settings::Settings;
use shared::setlist::CreateSetlist;
use shared::song::CreateSong;
use shared::team::{TeamMemberInput, TeamRole, TeamUserRef, UpdateTeam};

pub async fn test_db() -> AnyResult<Arc<Database>> {
    let db = Database::connect("mem://", "test", "test", None, None).await?;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/db-migrations");
    db.migrate(path).await?;
    Ok(Arc::new(db))
}

pub async fn seed_user(db: &Arc<Database>) -> AnyResult<User> {
    Ok(db.create_user(User::new("smoke@test.local")).await?)
}

pub async fn create_user(db: &Arc<Database>, email: &str) -> AnyResult<User> {
    Ok(db.create_user(User::new(email)).await?)
}

/// Personal team id for the user (matches API `team.id` — record id string only).
pub async fn personal_team_id(db: &Arc<Database>, user: &User) -> AnyResult<String> {
    let teams = db.list_teams_for_user(user).await?;
    let personal = teams
        .into_iter()
        .find(|t| t.owner.as_ref().map(|o| o.id == user.id).unwrap_or(false))
        .ok_or_else(|| anyhow::anyhow!("personal team not found"))?;
    Ok(personal.id)
}

pub fn minimal_song_data() -> SongData {
    serde_json::from_str(r#"{"titles":["T"],"sections":[]}"#).expect("song data")
}

pub async fn create_song_with_title(
    db: &Arc<Database>,
    user: &User,
    title: &str,
) -> AnyResult<shared::song::Song> {
    let mut data = minimal_song_data();
    data.titles = vec![title.to_string()];
    let create = CreateSong {
        not_a_song: false,
        blobs: vec![],
        data,
    };
    Ok(song_service(db).create_song_for_user(user, create).await?)
}

/// Adds non-owner members to the owner's personal team.
pub async fn configure_personal_team_members(
    db: &Arc<Database>,
    owner: &User,
    team_id: &str,
    members: Vec<(String, TeamRole)>,
) -> AnyResult<()> {
    let inputs: Vec<TeamMemberInput> = members
        .into_iter()
        .map(|(id, role)| TeamMemberInput {
            user: TeamUserRef { id },
            role,
        })
        .collect();
    db.update_team_for_user(
        owner,
        team_id,
        UpdateTeam {
            name: "Personal".into(),
            members: Some(inputs),
        },
    )
    .await?;
    Ok(())
}

/// Initializes [`Settings`] once (required for blob file I/O and similar).
pub fn init_settings_for_files() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path().to_string_lossy().to_string();
        std::mem::forget(dir);
        // SAFETY: single-threaded test process; env is set before Settings::init.
        unsafe {
            std::env::set_var("BLOB_DIR", &p);
            std::env::set_var("STATIC_DIR", &p);
            std::env::set_var("GMAIL_APP_PASSWORD", "test");
            std::env::set_var("GMAIL_FROM", "test@local");
            std::env::set_var("OTP_PEPPER", "test");
            std::env::set_var("PRINTER_ADDRESS", "http://127.0.0.1:9");
            std::env::set_var("PRINTER_API_KEY", "test");
        }
        Settings::init().expect("Settings::init in tests");
    });
}

/// Blob application service (same wiring as HTTP `main`).
pub fn blob_service(db: &Arc<Database>) -> BlobServiceHandle {
    let data = Data::from(db.clone());
    BlobServiceHandle::build(data)
}

/// Collection application service (same wiring as HTTP `main`).
pub fn collection_service(db: &Arc<Database>) -> CollectionServiceHandle {
    let data = Data::from(db.clone());
    CollectionServiceHandle::build(data)
}

/// Song application service (same wiring as HTTP `main`).
pub fn song_service(db: &Arc<Database>) -> SongServiceHandle {
    let data = Data::from(db.clone());
    SongServiceHandle::build(data)
}

/// Setlist application service (same wiring as HTTP `main`).
pub fn setlist_service(db: &Arc<Database>) -> SetlistServiceHandle {
    let data = Data::from(db.clone());
    SetlistService::new(
        SurrealSetlistRepo::new(data.clone()),
        SurrealTeamResolver::new(data.clone()),
        data.clone(),
    )
}

pub fn setlist_with_songs(title: &str, song_ids: &[(&str, Option<&str>)]) -> CreateSetlist {
    CreateSetlist {
        title: title.into(),
        songs: song_ids
            .iter()
            .map(|(id, nr)| shared::song::Link {
                id: (*id).into(),
                nr: nr.map(|s| s.into()),
                key: None,
            })
            .collect(),
    }
}
