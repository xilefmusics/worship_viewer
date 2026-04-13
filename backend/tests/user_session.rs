//! BLC coverage migrated from `tests/users.yml` / `tests/sessions.yml` (database-level).

mod support;

use backend::resources::{Session, SessionModel, User, UserModel, UserRole};
use chrono::Utc;

use support::{create_user, test_db};

#[tokio::test]
async fn blc_user_admin_and_session_lifecycle() {
    let db = test_db().await.expect("db");
    let admin = db
        .create_user(User {
            id: String::new(),
            email: "admin-sess@test.local".into(),
            role: UserRole::Admin,
            default_collection: None,
            created_at: Utc::now(),
            last_login_at: None,
            request_count: 0,
        })
        .await
        .expect("admin");

    let u = create_user(&db, "user-sess@test.local")
        .await
        .expect("user");

    let sess = db
        .create_session(Session::new(u.clone(), 3600))
        .await
        .expect("session");
    assert!(!sess.id.is_empty());

    let got = db.get_session(&sess.id).await.expect("get session");
    assert_eq!(got.user.id, u.id);

    let by_user = db.get_sessions_by_user_id(&u.id).await.expect("by user");
    assert!(by_user.iter().any(|s| s.id == sess.id));

    db.delete_session(&sess.id).await.expect("delete session");
    let _ = admin;
}
