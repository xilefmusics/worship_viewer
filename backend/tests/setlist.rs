//! BLC coverage migrated from `tests/setlists.yml` (database-level behavior).
//! HTTP-only cases (BLC-AUTH-*, BLC-HTTP-002 for missing JSON body, etc.) remain covered by Venom or future actix tests.

mod support;

use backend::error::AppError;
use backend::resources::User;
use backend::resources::song::Format;
use shared::api::ListQuery;
use shared::team::TeamRole;

use support::{
    configure_personal_team_members, create_song_with_title, create_user, personal_team_id,
    setlist_with_songs, test_db,
};

async fn four_user_setlist_fixture() -> (backend::database::Database, User, User, User, User, String)
{
    let db = test_db().await.expect("db");
    let owner = create_user(&db, "setlist-owner@test.local")
        .await
        .expect("owner");
    let read_u = create_user(&db, "setlist-read@test.local")
        .await
        .expect("read");
    let write_u = create_user(&db, "setlist-write@test.local")
        .await
        .expect("write");
    let noperm = create_user(&db, "setlist-noperm@test.local")
        .await
        .expect("noperm");

    let team_id = personal_team_id(&db, &owner).await.expect("team id");
    configure_personal_team_members(
        &db,
        &owner,
        &team_id,
        vec![
            (read_u.id.clone(), TeamRole::Guest),
            (write_u.id.clone(), TeamRole::ContentMaintainer),
        ],
    )
    .await
    .expect("acl");

    (db, owner, read_u, write_u, noperm, team_id)
}

/// BLC-SETL-002, BLC-TEAM-011: team ACL for setlist access
#[tokio::test]
async fn blc_setl_002_team_acl_configured() {
    let (_db, _owner, _read, _write, _noperm, _team) = four_user_setlist_fixture().await;
}

/// BLC-SONG-009: songs for setlist linking
/// BLC-SETL-009: create assigns owner to personal team; BLC-SETL-008 title/songs
#[tokio::test]
async fn blc_setl_009_create_owner_and_title() {
    let (db, owner, read_u, write_u, noperm, team_id) = four_user_setlist_fixture().await;
    let s1 = create_song_with_title(&db, &owner, "Setlist Song One")
        .await
        .expect("s1");
    let s2 = create_song_with_title(&db, &owner, "Setlist Song Two")
        .await
        .expect("s2");

    let created = db
        .create_setlist_for_user(
            &owner,
            setlist_with_songs(
                "Sunday Morning Set",
                &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
            ),
        )
        .await
        .expect("create");

    assert_eq!(created.owner, team_id);
    assert_eq!(created.title, "Sunday Morning Set");
    assert_eq!(created.songs.len(), 2);

    let for_delete = db
        .create_setlist_for_user(
            &owner,
            setlist_with_songs(
                "Setlist Delete By Write User",
                &[(s1.id.as_str(), Some("1"))],
            ),
        )
        .await
        .expect("for_delete");

    // BLC-SETL-010, BLC-SETL-005, BLC-LP-001, BLC-LP-002 — list counts
    let all = db
        .list_setlists_for_user(&owner, ListQuery::default())
        .await
        .expect("list");
    assert_eq!(all.len(), 2);

    let page = db
        .list_setlists_for_user(&owner, ListQuery::new().with_page(0).with_page_size(1))
        .await
        .expect("page");
    assert_eq!(page.len(), 1);

    let read_list = db
        .list_setlists_for_user(&read_u, ListQuery::default())
        .await
        .expect("read list");
    assert_eq!(read_list.len(), 2);

    let noperm_list = db
        .list_setlists_for_user(&noperm, ListQuery::default())
        .await
        .expect("noperm list");
    assert_eq!(noperm_list.len(), 0);

    // BLC-LP-003 search
    let q = db
        .list_setlists_for_user(&owner, ListQuery::new().with_q("Sunday"))
        .await
        .expect("q");
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].id, created.id);

    let q_page = db
        .list_setlists_for_user(
            &owner,
            ListQuery::new()
                .with_q("Sunday")
                .with_page(0)
                .with_page_size(1),
        )
        .await
        .expect("q page");
    assert_eq!(q_page.len(), 1);

    let q_empty = db
        .list_setlists_for_user(
            &owner,
            ListQuery::new().with_q("SetlistNoSuchTokenEver999zz"),
        )
        .await
        .expect("q empty");
    assert_eq!(q_empty.len(), 0);

    // BLC-LP-005: blank / whitespace q behaves like no search (full list)
    let q_blank = db
        .list_setlists_for_user(&owner, ListQuery::new().with_q(" "))
        .await
        .expect("q blank");
    assert_eq!(q_blank.len(), 2);

    // BLC-LP-006, BLC-LP-007, BLC-LP-008 pagination edge cases
    let zero_size = db
        .list_setlists_for_user(&owner, ListQuery::new().with_page(0).with_page_size(0))
        .await
        .expect("zero size");
    assert_eq!(zero_size.len(), 2);

    let page_only = db
        .list_setlists_for_user(&owner, ListQuery::new().with_page(1))
        .await
        .expect("page only");
    assert_eq!(page_only.len(), 2);

    let page_size_only = db
        .list_setlists_for_user(&owner, ListQuery::new().with_page_size(1))
        .await
        .expect("page only");
    assert_eq!(page_size_only.len(), 2);

    let beyond = db
        .list_setlists_for_user(&owner, ListQuery::new().with_page(10).with_page_size(10))
        .await
        .expect("beyond");
    assert_eq!(beyond.len(), 0);

    // BLC-SETL-011 GET one / songs
    let g1 = db
        .get_setlist_for_user(&owner, &created.id)
        .await
        .expect("get owner");
    assert_eq!(g1.songs.len(), 2);

    db.get_setlist_for_user(&read_u, &created.id)
        .await
        .expect("get read");

    let miss = db.get_setlist_for_user(&noperm, &created.id).await;
    assert!(matches!(miss, Err(AppError::NotFound(_))));

    // BLC-HTTP-001 invalid id shape
    let bad_id = db.get_setlist_for_user(&owner, "song:invalid").await;
    assert!(matches!(bad_id, Err(AppError::InvalidRequest(_))));

    let notfound = db
        .get_setlist_for_user(&owner, "never-created-setlist")
        .await;
    assert!(matches!(notfound, Err(AppError::NotFound(_))));

    // songs sub-resource
    let songs = db
        .setlist_songs_for_user(&owner, &created.id)
        .await
        .expect("songs owner");
    assert_eq!(songs.len(), 2);

    let songs_read = db
        .setlist_songs_for_user(&read_u, &created.id)
        .await
        .expect("songs read");
    assert_eq!(songs_read.len(), 2);

    let songs_noperm = db.setlist_songs_for_user(&noperm, &created.id).await;
    assert!(matches!(songs_noperm, Err(AppError::NotFound(_))));

    let songs_bad = db.setlist_songs_for_user(&owner, "song:invalid").await;
    assert!(matches!(songs_bad, Err(AppError::InvalidRequest(_))));

    let songs_nf = db
        .setlist_songs_for_user(&owner, "never-created-setlist")
        .await;
    assert!(matches!(songs_nf, Err(AppError::NotFound(_))));

    // player
    let player = db
        .setlist_player_for_user(&owner, &created.id)
        .await
        .expect("player");
    assert_eq!(player.toc().len(), 2);

    db.setlist_player_for_user(&read_u, &created.id)
        .await
        .expect("player read");

    let pl_noperm = db.setlist_player_for_user(&noperm, &created.id).await;
    assert!(matches!(pl_noperm, Err(AppError::NotFound(_))));

    let pl_bad = db.setlist_player_for_user(&owner, "song:invalid").await;
    assert!(matches!(pl_bad, Err(AppError::InvalidRequest(_))));

    let pl_nf = db
        .setlist_player_for_user(&owner, "never-created-setlist")
        .await;
    assert!(matches!(pl_nf, Err(AppError::NotFound(_))));

    // export (non-PDF to avoid printer)
    db.export_setlist_for_user(&owner, &created.id, Format::WorshipPro)
        .await
        .expect("export owner");
    db.export_setlist_for_user(&read_u, &created.id, Format::WorshipPro)
        .await
        .expect("export read");

    let ex_noperm = db
        .export_setlist_for_user(&noperm, &created.id, Format::WorshipPro)
        .await;
    assert!(matches!(ex_noperm, Err(AppError::NotFound(_))));

    let ex_bad = db
        .export_setlist_for_user(&owner, "song:invalid", Format::WorshipPro)
        .await;
    assert!(matches!(ex_bad, Err(AppError::InvalidRequest(_))));

    let ex_nf = db
        .export_setlist_for_user(&owner, "never-created-setlist", Format::WorshipPro)
        .await;
    assert!(matches!(ex_nf, Err(AppError::NotFound(_))));

    // BLC-SETL-008, BLC-SETL-003 PUT
    let updated = db
        .update_setlist_for_user(
            &owner,
            &created.id,
            setlist_with_songs(
                "Updated Sunday Set",
                &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
            ),
        )
        .await
        .expect("put owner");
    assert_eq!(updated.title, "Updated Sunday Set");

    let write_updated = db
        .update_setlist_for_user(
            &write_u,
            &created.id,
            setlist_with_songs(
                "Write User Updated Set",
                &[(s1.id.as_str(), Some("10")), (s2.id.as_str(), Some("20"))],
            ),
        )
        .await
        .expect("put write");
    assert_eq!(write_updated.title, "Write User Updated Set");

    let got = db
        .get_setlist_for_user(&owner, &created.id)
        .await
        .expect("get after write");
    assert_eq!(got.title, "Write User Updated Set");

    let put_noperm = db
        .update_setlist_for_user(
            &noperm,
            &created.id,
            setlist_with_songs("Should Fail", &[(s1.id.as_str(), None)]),
        )
        .await;
    assert!(matches!(put_noperm, Err(AppError::NotFound(_))));

    let put_read = db
        .update_setlist_for_user(
            &read_u,
            &created.id,
            setlist_with_songs(
                "Read User Put",
                &[(s1.id.as_str(), Some("1")), (s2.id.as_str(), Some("2"))],
            ),
        )
        .await;
    assert!(matches!(put_read, Err(AppError::NotFound(_))));

    let put_bad = db
        .update_setlist_for_user(
            &owner,
            "song:invalid",
            setlist_with_songs("x", &[(s1.id.as_str(), None)]),
        )
        .await;
    assert!(matches!(put_bad, Err(AppError::InvalidRequest(_))));

    let put_nf = db
        .update_setlist_for_user(
            &owner,
            "never-created-setlist",
            setlist_with_songs("Unknown Setlist", &[(s1.id.as_str(), None)]),
        )
        .await;
    assert!(matches!(put_nf, Err(AppError::NotFound(_))));

    // DELETE BLC-SETL-007, BLC-SETL-008, BLC-SETL-012
    let del_noperm = db.delete_setlist_for_user(&noperm, &created.id).await;
    assert!(matches!(del_noperm, Err(AppError::NotFound(_))));

    let del_bad = db.delete_setlist_for_user(&owner, "song:invalid").await;
    assert!(matches!(del_bad, Err(AppError::InvalidRequest(_))));

    db.delete_setlist_for_user(&write_u, &for_delete.id)
        .await
        .expect("delete write");

    db.delete_setlist_for_user(&owner, &created.id)
        .await
        .expect("delete owner");

    let again = db.delete_setlist_for_user(&owner, &created.id).await;
    assert!(matches!(again, Err(AppError::NotFound(_))));
}
