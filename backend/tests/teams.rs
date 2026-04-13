//! BLC coverage migrated from `tests/0_teams.yml` / `z_team_invitations.yml` (database-level).

mod support;

use backend::error::AppError;
use shared::team::CreateTeam;

use support::{create_user, test_db};

#[tokio::test]
async fn blc_team_shared_create_and_list() {
    let db = test_db().await.expect("db");
    let u = create_user(&db, "team-creator@test.local")
        .await
        .expect("u");
    let t = db
        .create_shared_team_for_user(
            &u,
            CreateTeam {
                name: "Band".into(),
                members: vec![],
            },
        )
        .await
        .expect("shared team");

    assert!(!t.id.is_empty());
    assert_eq!(t.name, "Band");

    let teams = db.list_teams_for_user(&u).await.expect("teams");
    assert!(teams.iter().any(|x| x.id == t.id));
}

#[tokio::test]
async fn blc_team_personal_cannot_delete() {
    let db = test_db().await.expect("db");
    let u = create_user(&db, "team-personal@test.local")
        .await
        .expect("u");
    let teams = db.list_teams_for_user(&u).await.expect("teams");
    let personal = teams
        .iter()
        .find(|t| t.owner.as_ref().map(|o| o.id == u.id).unwrap_or(false))
        .expect("personal");
    let err = db.delete_team_for_user(&u, &personal.id).await;
    assert!(matches!(err, Err(AppError::Forbidden)));
}

#[tokio::test]
async fn blc_team_delete_shared_empty_team() {
    let db = test_db().await.expect("db");
    let u = create_user(&db, "team-del@test.local").await.expect("u");
    let shared = db
        .create_shared_team_for_user(
            &u,
            CreateTeam {
                name: "ToRemove".into(),
                members: vec![],
            },
        )
        .await
        .expect("shared");
    db.delete_team_for_user(&u, &shared.id)
        .await
        .expect("delete");
}
