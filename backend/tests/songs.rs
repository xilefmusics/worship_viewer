//! BLC coverage migrated from `tests/songs.yml` (database-level).

mod support;

use shared::api::ListQuery;
use shared::team::TeamRole;

use support::{
    configure_personal_team_members, create_song_with_title, create_user, personal_team_id,
    setlist_service, test_db,
};

#[tokio::test]
async fn blc_song_crud_search_likes() {
    let db = test_db().await.expect("db");
    let owner = create_user(&db, "song-owner@test.local").await.expect("o");
    let other = create_user(&db, "song-other@test.local").await.expect("x");
    let team_id = personal_team_id(&db, &owner).await.expect("team");
    configure_personal_team_members(
        &db,
        &owner,
        &team_id,
        vec![(other.id.clone(), TeamRole::Guest)],
    )
    .await
    .expect("acl");

    let s1 = create_song_with_title(&db, &owner, "Unique Song Alpha")
        .await
        .expect("s1");
    let _s2 = create_song_with_title(&db, &owner, "Other Beta")
        .await
        .expect("s2");

    let list = db
        .list_songs_for_user(&owner, ListQuery::default())
        .await
        .expect("list");
    assert!(list.len() >= 2);

    let q = db
        .list_songs_for_user(&owner, ListQuery::new().with_q("Alpha"))
        .await
        .expect("search");
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].id, s1.id);

    db.get_song_for_user(&owner, &s1.id).await.expect("get");

    // Guest on same team can read (BLC-SONG-002 read path)
    db.get_song_for_user(&other, &s1.id)
        .await
        .expect("guest read");

    let bad = db.get_song_for_user(&owner, "setlist:not-a-song").await;
    assert!(bad.is_err(), "wrong table id should not resolve: {bad:?}");

    db.set_song_like_status_for_user(&owner, &s1.id, true)
        .await
        .expect("like");
    let st = db
        .song_like_status_for_user(&owner, &s1.id)
        .await
        .expect("like status");
    assert!(st.liked);

    db.delete_song_for_user(&owner, &s1.id).await.expect("del");
}

/// BLC-SONG-018 / cascade: deleting song should remove from setlist/collection references — covered by model triggers; smoke delete.
#[tokio::test]
async fn blc_song_delete_after_setlist_link() {
    let db = test_db().await.expect("db");
    let u = create_user(&db, "song-del@test.local").await.expect("u");
    let song = create_song_with_title(&db, &u, "ToDelete")
        .await
        .expect("song");
    let sl = setlist_service(&db)
        .create_setlist_for_user(
            &u,
            support::setlist_with_songs("L", &[(song.id.as_str(), Some("1"))]),
        )
        .await
        .expect("setlist");
    db.delete_song_for_user(&u, &song.id)
        .await
        .expect("del song");
    let g = setlist_service(&db)
        .get_setlist_for_user(&u, &sl.id)
        .await
        .expect("get setlist");
    assert!(g.songs.is_empty());
}
