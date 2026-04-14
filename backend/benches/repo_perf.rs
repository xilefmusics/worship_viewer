//! Criterion benchmarks for team resolution and setlist repository (in-memory SurrealDB).
use std::sync::Arc;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tokio::runtime::Runtime;

use backend::database::Database;
use backend::resources::User;
use backend::resources::setlist::{SetlistRepository, SurrealSetlistRepo};
use backend::resources::team::content_read_team_things;
use backend::resources::user::UserServiceHandle;
use shared::api::ListQuery;

fn setup() -> (Runtime, Arc<Database>, User) {
    let rt = Runtime::new().expect("runtime");
    let db = rt.block_on(async {
        let db = Database::connect("mem://", "bench", "bench", None, None)
            .await
            .expect("connect");
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/db-migrations");
        db.migrate(path).await.expect("migrate");
        Arc::new(db)
    });
    let user = rt.block_on(async {
        UserServiceHandle::build(db.clone())
            .create_user(User::new("bench@local"))
            .await
            .expect("user")
    });
    (rt, db, user)
}

fn bench_repo_hot_paths(c: &mut Criterion) {
    let (rt, db, user) = setup();

    c.bench_function("content_read_team_things", |b| {
        b.iter(|| {
            rt.block_on(async {
                content_read_team_things(black_box(db.as_ref()), black_box(&user))
                    .await
                    .unwrap();
            });
        });
    });

    let teams = rt
        .block_on(content_read_team_things(db.as_ref(), &user))
        .unwrap();

    let repo = SurrealSetlistRepo::new(db.clone());
    c.bench_function("get_setlists_empty", |b| {
        b.iter(|| {
            rt.block_on(async {
                repo.get_setlists(black_box(teams.clone()), black_box(ListQuery::default()))
                    .await
                    .unwrap();
            });
        });
    });
}

criterion_group!(benches, bench_repo_hot_paths);
criterion_main!(benches);
