//! BLC coverage migrated from `tests/blobs.yml` (database-level; requires [`Settings`] for filesystem).

mod support;

use backend::error::AppError;
use shared::api::ListQuery;
use shared::blob::{CreateBlob, FileType};
use shared::team::TeamRole;

use support::{
    configure_personal_team_members, create_user, init_settings_for_files, personal_team_id,
    test_db,
};

#[tokio::test]
async fn blc_blob_crud() {
    init_settings_for_files();
    let db = test_db().await.expect("db");
    let owner = create_user(&db, "blob-owner@test.local").await.expect("o");
    let other = create_user(&db, "blob-other@test.local").await.expect("x");
    let team_id = personal_team_id(&db, &owner).await.expect("team");
    configure_personal_team_members(
        &db,
        &owner,
        &team_id,
        vec![(other.id.clone(), TeamRole::Guest)],
    )
    .await
    .expect("acl");

    let b = db
        .create_blob_for_user(
            &owner,
            CreateBlob {
                file_type: FileType::PNG,
                width: 10,
                height: 10,
                ocr: "hi".into(),
            },
        )
        .await
        .expect("create");

    let list = db
        .list_blobs_for_user(&owner, ListQuery::default())
        .await
        .expect("list");
    assert!(list.iter().any(|x| x.id == b.id));

    db.get_blob_for_user(&owner, &b.id).await.expect("get");

    db.get_blob_for_user(&other, &b.id)
        .await
        .expect("guest read");

    let miss = db.get_blob_for_user(&other, "never-created").await;
    assert!(matches!(miss, Err(AppError::NotFound(_))));

    db.delete_blob_for_user(&owner, &b.id)
        .await
        .expect("delete");
}
