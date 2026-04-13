//! BLC coverage migrated from `tests/collections.yml` (database-level).

mod support;

use backend::error::AppError;
use shared::api::ListQuery;
use shared::collection::CreateCollection;
use shared::song::Link as SongLink;
use shared::team::TeamRole;

use support::{
    configure_personal_team_members, create_song_with_title, create_user, personal_team_id, test_db,
};

#[tokio::test]
async fn blc_collection_crud_and_acl() {
    let db = test_db().await.expect("db");
    let owner = create_user(&db, "coll-owner@test.local").await.expect("o");
    let guest = create_user(&db, "coll-guest@test.local").await.expect("g");
    let team_id = personal_team_id(&db, &owner).await.expect("team");
    configure_personal_team_members(
        &db,
        &owner,
        &team_id,
        vec![(guest.id.clone(), TeamRole::Guest)],
    )
    .await
    .expect("acl");

    let song = create_song_with_title(&db, &owner, "Coll Song")
        .await
        .expect("song");

    let col = db
        .create_collection_for_user(
            &owner,
            CreateCollection {
                title: "My Collection".into(),
                cover: "mysongs".into(),
                songs: vec![SongLink {
                    id: song.id.clone(),
                    nr: None,
                    key: None,
                }],
            },
        )
        .await
        .expect("create");

    assert_eq!(col.owner, team_id);

    let list = db
        .list_collections_for_user(&owner, ListQuery::default())
        .await
        .expect("list");
    assert!(list.iter().any(|c| c.id == col.id));

    db.get_collection_for_user(&guest, &col.id)
        .await
        .expect("guest read");

    let upd = db
        .update_collection_for_user(
            &owner,
            &col.id,
            CreateCollection {
                title: "Updated".into(),
                cover: "mysongs".into(),
                songs: vec![SongLink {
                    id: song.id.clone(),
                    nr: Some("1".into()),
                    key: None,
                }],
            },
        )
        .await
        .expect("update");
    assert_eq!(upd.title, "Updated");

    let put_guest = db
        .update_collection_for_user(
            &guest,
            &col.id,
            CreateCollection {
                title: "Nope".into(),
                cover: "mysongs".into(),
                songs: vec![],
            },
        )
        .await;
    assert!(matches!(put_guest, Err(AppError::NotFound(_))));

    db.delete_collection_for_user(&owner, &col.id)
        .await
        .expect("delete");
}
