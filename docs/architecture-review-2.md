# Backend Architecture Review

> A frank, friendly review of the `worship_viewer` backend.
> Scored for **performance**, **testability**, and **clean architecture**.

---

## TL;DR Scorecard

| Area | Grade | Vibe |
|------|-------|------|
| Layered Architecture | **A** | Textbook ports-and-adapters. Chef's kiss. |
| Testability | **A-** | Trait-based DI, mocks, real-DB tests — very solid |
| Error Handling | **B+** | Clean `AppError`, but some blind spots |
| Configuration | **B-** | Global singleton — works, but bites in tests |
| Database Layer | **B** | Powerful migration system, raw queries need care |
| Auth & Middleware | **B+** | Solid session + OIDC, middleware is clear |
| Performance | **B** | Good concurrent patterns, some low-hanging fruit |
| Code Organization | **A-** | Consistent vertical slices, some coupling to clean up |

---

## 1. The Good Stuff (What You're Doing Right)

### 1.1 Ports and Adapters — You Actually Did It

This is the highlight of the codebase. Every resource domain follows the same clean pattern:

```
repository.rs    → trait (the "port")
surreal_repo.rs  → SurrealDB implementation (the "adapter")
service.rs       → business logic, generic over traits
rest.rs          → HTTP handlers, thin delegation
model.rs         → DB record shapes and conversions
```

The `SongService` signature is a textbook example:

```46:57:backend/src/resources/song/service.rs
pub struct SongService<R, T, L, C, U> {
    pub repo: R,
    pub teams: T,
    pub likes: L,
    pub collections: C,
    pub user_updater: U,
}

impl<R, T, L, C, U> SongService<R, T, L, C, U> {
    pub fn new(repo: R, teams: T, likes: L, collections: C, user_updater: U) -> Self {
        Self { repo, teams, likes, collections, user_updater }
    }
}
```

Every dependency is a generic type parameter bounded by a trait. This means:
- You can swap SurrealDB for Postgres tomorrow without touching business logic
- Tests can inject mocks for any dependency independently
- The compiler enforces the contract — not just convention

The `*ServiceHandle` type alias pattern is also clean — it gives you a one-liner for "this is the production wiring" without polluting the generic service:

```216:222:backend/src/resources/song/service.rs
pub type SongServiceHandle = SongService<
    SurrealSongRepo,
    crate::resources::team::SurrealTeamResolver,
    Data<Database>,
    crate::resources::collection::SurrealCollectionRepo,
    Data<SurrealUserRepo>,
>;
```

**Verdict:** This is genuinely well-done. Many Rust projects skip this and end up with untestable monoliths. You didn't.

### 1.2 The `UserPermissions` Cache — Elegant Request-Scoped Optimization

```34:86:backend/src/resources/team/resolver.rs
pub struct UserPermissions<'a, T: TeamResolver> {
    user: &'a User,
    resolver: &'a T,
    read_teams: OnceCell<Vec<Thing>>,
    write_teams: OnceCell<Vec<Thing>>,
    personal_team: OnceCell<Thing>,
}
// ...
    pub async fn read_teams(&self) -> Result<Vec<Thing>, AppError> {
        let user = self.user;
        let resolver = self.resolver;
        self.read_teams
            .get_or_try_init(|| async move { resolver.content_read_teams(user).await })
            .await
            .map(Vec::clone)
    }
```

This is a great pattern. The team resolver hits the database, but you only pay for it once per request. The `OnceCell` ensures laziness (write teams are never resolved if the request only reads), and the cache lives for exactly the right scope — a single request handler.

**Verdict:** Smart and idiomatic. This saves you from a class of "N+1" problems at the authorization layer.

### 1.3 Consistent Error Handling with `AppError`

```7:21:backend/src/error.rs
#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}
```

And the `ResponseError` impl is solid — internal errors get logged, user-facing errors stay clean:

```113:118:backend/src/error.rs
    fn error_response(&self) -> HttpResponse {
        if matches!(self, AppError::Internal(_)) {
            error!("{}", self);
        }
        HttpResponse::build(self.status_code()).json(json!({ "error": self.to_string() }))
    }
```

The `From<surrealdb::Error>` conversion is also well-thought-out — mapping DB-level errors to semantic HTTP errors rather than leaking database internals.

**Verdict:** This is how application errors should work. Clean separation between internal and external error surfaces.

### 1.4 Migration System — Production-Grade

The custom migration runner with SHA-256 checksums is solid:
- Ordered `.surql` files with timestamp prefixes
- Transactional application (BEGIN/COMMIT)
- Checksum verification prevents silent drift
- Clear logging with duration tracking

This is the kind of thing people skip and regret later. You didn't skip it.

### 1.5 Testing Strategy — Two-Layer Approach

You have both:

1. **Fast mock-based unit tests** (e.g., `setlist/service.rs` tests with `MockRepo`, `MockTeams`, `MockLikes`)
2. **Integration tests against a real in-memory SurrealDB** (e.g., `blc_song_crud_search_likes`)

The mock tests verify service logic in isolation. The integration tests catch SurrealQL bugs and schema issues. This is the right combination.

The `resolver.rs` tests are particularly clever — they run the *same* queries through both the optimized SurrealQL path and a naive Rust-side filter, then assert they produce identical results:

```256:263:backend/src/resources/team/resolver.rs
    #[tokio::test]
    async fn content_read_teams_matches_naive_rust_filter() {
        let db = test_db().await.expect("test db");
        let user = seed_user(&db).await.expect("user");
        let dbref: &Database = db.as_ref();
        let a = content_read_team_things(dbref, &user).await.expect("sql read");
        let b = naive_read_teams(dbref, &user).await.expect("rust read");
        assert_eq!(thing_key_set(&a), thing_key_set(&b));
    }
```

That's a property-based testing mindset applied to SQL — really smart.

---

## 2. The Rough Edges (What Needs Work)

### 2.1 `Settings::global()` — The Global Singleton Problem

**Severity: Medium | Impact: Testability, Flexibility**

```4:4:backend/src/settings.rs
static SETTINGS: OnceCell<Settings> = OnceCell::new();
```

`Settings::global()` is called directly from:
- `Database::new()`
- `RequireUser` middleware
- `Mail::send()`
- OIDC client setup
- Frontend/auth routes

This creates invisible coupling. Any code that calls `Settings::global()` has a hidden dependency that doesn't show up in its function signature. And in tests, it's worse — look at your test helper:

```95:113:backend/src/test_helpers.rs
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
            // ...
        }
        Settings::init().expect("Settings::init in tests");
    });
}
```

`unsafe` + `std::mem::forget` + environment variable mutation. This works, but it's a code smell that signals the architecture is fighting you. Settings should be injectable, not global.

**Recommendation:** Pass `&Settings` (or relevant subsets) as constructor parameters where needed. For `Database::new()`, you already have `Database::connect()` with explicit params — consider making `connect()` the *only* constructor and removing `new()`. For middleware and mail, consider wrapping the needed config values in `Data<T>` app state.

### 2.2 Raw SurrealQL String Building

**Severity: Medium | Impact: Maintainability, Safety**

The query construction in `surreal_repo.rs` files builds SQL strings by concatenation:

```45:59:backend/src/resources/song/surreal_repo.rs
        let mut query = if q_nonempty {
            String::from(
                "SELECT *, ((search::score(0) ?? 0) * 100 + (search::score(1) ?? 0) * 10 + (search::score(2) ?? 0) * 1) AS score FROM song WHERE owner IN $teams",
            )
        } else {
            String::from("SELECT * FROM song WHERE owner IN $teams")
        };
        if q_nonempty {
            query.push_str(
                " AND (data.titles @0@ $q OR data.artists @1@ $q OR search_content @2@ $q) ORDER BY score DESC",
            );
        }
        if pagination.to_offset_limit().is_some() {
            query.push_str(" LIMIT $limit START $start");
        }
```

To be fair — the *values* are properly parameterized with `.bind()`, so there's no injection risk. But the conditional string building is fragile:
- Easy to introduce syntax errors (a missing space, a stray AND)
- Duplicated patterns across multiple repos
- Hard to review or refactor

**Recommendation:** Consider a small query-builder helper or at least extract the query templates as constants. Even a simple enum of query variants would be easier to audit than string concatenation spread across files.

### 2.3 `HttpResponse` Leaking into the Service Layer

**Severity: Low-Medium | Impact: Clean Architecture**

The `export_song_for_user` method returns `HttpResponse` directly from the service layer:

```119:128:backend/src/resources/song/service.rs
    pub async fn export_song_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
        format: Format,
    ) -> Result<HttpResponse, AppError> {
        let read_teams = perms.read_teams().await?;
        let song = self.repo.get_song(read_teams, id).await?;
        export(vec![song], format).await
    }
```

The service layer should be framework-agnostic. If you ever wanted to reuse `SongService` in a CLI tool, a gRPC service, or a background job, `HttpResponse` would be a dependency you'd have to refactor out.

**Recommendation:** Return the export data as bytes + content type from the service, and let `rest.rs` build the `HttpResponse`. Something like:

```rust
pub struct ExportResult {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
    pub filename: String,
}
```

### 2.4 `Data<Database>` as an `actix_web` Type in the Repository Layer

**Severity: Low-Medium | Impact: Framework Coupling**

Every Surreal repo wraps its database handle in `actix_web::web::Data<Database>`:

```22:29:backend/src/resources/song/surreal_repo.rs
pub struct SurrealSongRepo {
    db: Data<Database>,
}

impl SurrealSongRepo {
    pub fn new(db: Data<Database>) -> Self {
        Self { db }
    }
```

`Data<T>` is `Arc<T>` with actix-web-specific extras. The repository layer — which is supposed to be a pure persistence adapter — shouldn't know about the web framework at all.

**Recommendation:** Use `Arc<Database>` instead. The conversion is trivial (`Data::from(arc)` and `Data::into_inner()`), and your repos become framework-independent. This also simplifies test wiring — your `test_helpers.rs` already uses `Arc<Database>` internally and has to convert it:

```117:119:backend/src/test_helpers.rs
pub fn blob_service(db: &Arc<Database>) -> BlobServiceHandle {
    let data = Data::from(db.clone());
    BlobServiceHandle::build(data)
}
```

With `Arc<Database>` in the repos, the `Data::from()` dance disappears from test helpers entirely.

### 2.5 `list_teams_for_user` Fetches All Teams

**Severity: Medium | Impact: Performance**

```40:57:backend/src/resources/team/service.rs
    pub async fn list_teams_for_user(&self, user: &User) -> Result<Vec<Team>, AppError> {
        let app_admin = user.role == UserRole::Admin;
        let rows = self.repo.fetch_all_teams().await?;
        let mut by_id: BTreeMap<String, Team> = BTreeMap::new();
        for row in rows {
            let stored = team_fetched_to_stored(&row)?;
            if can_read_team(&user.id, &stored, app_admin) {
                let team = row.into_team()?;
                by_id.insert(team.id.clone(), team);
            }
        }
        // ...
    }
```

`fetch_all_teams()` loads *every team in the database*, then filters in Rust. For a small worship team app, this is fine today. But it's an O(n) full-table scan that will bite you as the user base grows.

**Recommendation:** Push the filter into the database query, similar to how `content_read_team_things` already does it in `resolver.rs`. The resolver already has the optimized SurrealQL for this — you could reuse or adapt that pattern.

### 2.6 `Mail::send()` Is Blocking

**Severity: Medium | Impact: Performance**

```28:53:backend/src/mail.rs
    pub fn send(self) -> Result<(), AppError> {
        let settings = Settings::global();
        let response = SmtpTransport::relay("smtp.gmail.com")
            .map_err(AppError::mail)?
            .credentials(Credentials::new(
                settings.gmail_from.to_owned(),
                settings.gmail_app_password.to_owned(),
            ))
            .build()
            .send(/* ... */)
            .map_err(AppError::mail)?;
        // ...
    }
```

`SmtpTransport::send()` from `lettre` is a **synchronous, blocking** call. When called from an async handler (which it presumably is via OTP), it blocks the entire Tokio worker thread until the SMTP round-trip completes. Under load, this can starve the async runtime.

Additionally, a new SMTP connection is created for *every* email sent — no connection pooling.

**Recommendation:**
1. Use `lettre::AsyncSmtpTransport` with the `tokio1` feature instead
2. Create the transport once at startup and share it via app state
3. If you can't switch to async, at least wrap the call in `tokio::task::spawn_blocking()`

### 2.7 The Benchmark File Is Likely Stale

The exploration found that `benches/repo_perf.rs` references `UserModel` and `db.create_user()` — types and methods that no longer exist in the current API. This means your benchmarks don't compile, which means they're not running in CI, which means you have no performance regression safety net.

**Recommendation:** Either fix the benchmarks to match the current API, or remove them to avoid confusion. Dead code that looks alive is worse than no code.

---

## 3. Architectural Patterns — Deeper Analysis

### 3.1 Authorization Model: The Good and The Tricky

Your authorization works at two levels:

1. **Middleware level:** `RequireUser` and `RequireAdmin` handle authentication and role checks
2. **Service level:** `UserPermissions` + `TeamResolver` handle content ACL via team membership

This is correct — you're not trying to do fine-grained authorization in middleware (which would require loading the resource before you can check permissions). Instead, the service layer resolves teams and passes them down to the repository, where the query itself enforces access:

```sql
DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE
```

This is a **query-level authorization pattern** — the DB query simply returns nothing if you don't have access. It's simple, performant, and impossible to bypass accidentally. Nice.

One subtlety worth noting: for `get_song`, you do a `SELECT` then check ownership in Rust:

```80:87:backend/src/resources/song/surreal_repo.rs
    async fn get_song(&self, read_teams: Vec<Thing>, id: &str) -> Result<Song, AppError> {
        let db = self.inner();
        let record: Option<SongRecord> = db.db.select(resource_id("song", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, &read_teams) => Ok(r.into_song()),
            _ => Err(AppError::NotFound("song not found".into())),
        }
    }
```

This is fine for reads (no security issue — the data is just discarded), but it's a slightly different pattern than the writes. Consider unifying: either always check in Rust, or always check in the query. Consistency makes auditing easier.

### 3.2 The `CreateSong` Re-Use Anti-Pattern

`CreateSong` is used for both creation and updates. This is pragmatic but hides intent:

```174:182:backend/src/resources/song/service.rs
    pub async fn update_song_for_user(
        &self,
        perms: &UserPermissions<'_, T>,
        id: &str,
        song: CreateSong,
    ) -> Result<Song, AppError> {
```

When reading this signature, you'd expect "create" semantics, not "replace" semantics. An `UpdateSong` type (even if it's structurally identical) would improve readability and let you add update-specific validation later without breaking the create path.

### 3.3 The `update_song` Upsert Logic Is Surprising

```100:150:backend/src/resources/song/surreal_repo.rs
    async fn update_song(
        &self,
        // ...
    ) -> Result<Song, AppError> {
        // First: try UPDATE ... WHERE owner IN $teams
        // If no rows returned:
        //   Check if record exists → if yes, return NotFound (no permission)
        //   If record doesn't exist → CREATE it as a new record with the given ID
```

An "update" endpoint that silently creates records if they don't exist is an upsert. This is probably intentional (maybe for import/sync use cases?), but it's the kind of hidden behavior that can cause very confusing bugs. At minimum, this deserves a doc comment explaining *why* the upsert exists.

---

## 4. Performance Opportunities

### 4.1 `tokio::try_join!` — Already Good, Can Go Further

You're already using `try_join!` for concurrent resolution:

```74:77:backend/src/resources/song/service.rs
        let (liked_set, read_teams) = tokio::try_join!(
            self.likes.liked_song_ids(&user_id),
            perms.read_teams()
        )?;
```

But there are places where sequential `await`s could be parallelized. For example, in `create_song_for_user`, the default collection creation and the song creation are sequential — the collection step could potentially be spawned as a background task since it's not critical path.

### 4.2 `Vec<Thing>` Cloning in Team Resolution

`UserPermissions::read_teams()` returns `Vec::clone()` every time:

```58:65:backend/src/resources/team/resolver.rs
    pub async fn read_teams(&self) -> Result<Vec<Thing>, AppError> {
        // ...
        self.read_teams
            .get_or_try_init(|| async move { resolver.content_read_teams(user).await })
            .await
            .map(Vec::clone)
    }
```

This means every service method that calls `perms.read_teams()` gets a fresh clone of the Vec. For a typical request that calls `read_teams()` once, this is fine. But if a handler calls multiple service methods that each call `read_teams()`, you're cloning unnecessarily.

**Recommendation:** Return `&[Thing]` instead of `Vec<Thing>`, or use `Arc<Vec<Thing>>` for cheap cloning. This would require adjusting the repository trait signatures to accept `&[Thing]` instead of `Vec<Thing>`.

### 4.3 No Pagination Default Limits

Your `ListQuery` doesn't enforce a maximum page size. An API consumer (or an attacker) can request `page_size=1000000` and pull the entire dataset. Consider adding a server-side cap.

### 4.4 SMTP Connection Per Email

As mentioned in 2.6, every email creates a new SMTP connection. For OTP flows under load, this could become a bottleneck. Pool it.

---

## 5. Testability Deep Dive

### 5.1 What's Working Well

- **Trait-based mocking:** Your `MockRepo`, `MockTeams`, `MockLikes` pattern is clean and doesn't require a mocking framework — the traits are simple enough to implement by hand.
- **In-memory DB tests:** `test_db()` gives you a real SurrealDB with migrations applied, catching schema and query bugs that mocks would miss.
- **Test helper module:** `test_helpers.rs` provides a nice factory layer (`create_user`, `song_service`, `setlist_with_songs`) that keeps tests readable.

### 5.2 What Could Be Better

**The `init_settings_for_files()` unsafe block** is the most fragile piece. It relies on:
- Tests being single-threaded (not guaranteed with `cargo test`)
- `Settings::init()` not having been called yet by another test
- Environment variables not being modified concurrently

This will break eventually when you add enough tests that Cargo runs them in parallel threads. The fix is the same as 2.1: make Settings injectable rather than global.

**No HTTP-level unit tests.** You have service-level tests and external YAML integration tests, but no tests that exercise the actix handler wiring (extractors, response serialization, error mapping). Consider adding a few tests using `actix_web::test::TestRequest` to verify the full request-response cycle within Rust.

**Large monolithic tests.** The `blc_setl_009_create_owner_and_title` test is 300+ lines long and tests create, list, paginate, search, get, songs, player, export, update, and delete all in one function. This makes failures hard to diagnose — when it breaks, you'll know *which assertion* failed but not easily *which operation* caused it without reading the whole function. Consider splitting into focused tests.

---

## 6. Summary of Recommendations

Ranked by impact-to-effort ratio:

| # | Recommendation | Effort | Impact |
|---|---------------|--------|--------|
| 1 | Replace `Data<Database>` with `Arc<Database>` in repos | Small | Decouples from actix |
| 2 | Return `ExportResult` instead of `HttpResponse` from services | Small | Clean architecture |
| 3 | Fix or remove stale benchmarks | Small | CI hygiene |
| 4 | Use `lettre::AsyncSmtpTransport` or `spawn_blocking` for mail | Small | Performance under load |
| 5 | Add pagination limits (max page size) | Small | Safety |
| 6 | Push team filtering into DB query for `list_teams_for_user` | Medium | Performance at scale |
| 7 | Return `&[Thing]` from `UserPermissions` instead of cloned Vecs | Medium | Fewer allocations |
| 8 | Make `Settings` injectable instead of global | Medium | Testability, flexibility |
| 9 | Add document comments explaining the upsert in `update_song` | Tiny | Clarity |
| 10 | Split large integration tests into focused tests | Medium | Debuggability |
| 11 | Add actix `TestRequest`-level handler tests | Medium | Coverage of HTTP layer |

---

## 7. Final Thoughts

This is a well-structured backend. The ports-and-adapters pattern is genuinely and consistently applied — not just aspirational. The authorization model is thoughtful, the migration system is production-ready, and the test suite has good coverage with a smart mix of mocks and real-DB tests.

The main themes for improvement are:
1. **Reduce global state** (`Settings::global()`, `Data<Database>` coupling)
2. **Keep the service layer framework-agnostic** (no `HttpResponse`, no `Data<T>`)
3. **Performance hygiene** (async mail, pagination limits, push filters to DB)

None of these are urgent — the codebase works and is maintainable today. But addressing them will make the architecture more resilient as the app grows.

Nice work.

---

## 8. Action Plan

A step-by-step implementation plan for every recommendation above (except pagination limits).
Each phase is self-contained — it compiles and passes tests before moving to the next.
Phases are ordered so that earlier work unblocks or simplifies later work.

---

### Phase 1: `Data<Database>` → `Arc<Database>` in Repos

**Goal:** Decouple the entire repository + resolver layer from `actix_web`.

**Why first:** This is a mechanical refactor with the highest impact-to-effort ratio. It removes `use actix_web::web::Data` from 10+ files and simplifies test wiring. Every later phase benefits from repos being framework-agnostic.

**Files to change:**

| File | What changes |
|------|-------------|
| `resources/blob/surreal_repo.rs` | `db: Data<Database>` → `db: Arc<Database>`, `new(db: Arc<Database>)`, remove `use actix_web::web::Data`, add `use std::sync::Arc` |
| `resources/collection/surreal_repo.rs` | Same |
| `resources/setlist/surreal_repo.rs` | Same |
| `resources/song/surreal_repo.rs` | Same |
| `resources/team/surreal_repo.rs` | Same |
| `resources/team/invitation_surreal_repo.rs` | Same |
| `resources/team/resolver.rs` | `SurrealTeamResolver` — same change |
| `resources/user/surreal_repo.rs` | Same |
| `resources/user/session/surreal_repo.rs` | Same |
| `resources/song/service.rs` | `SongServiceHandle` type alias: `Data<Database>` → `Arc<Database>` for the `LikedSongIds` impl, `Data<SurrealUserRepo>` → `Arc<SurrealUserRepo>` for `UserCollectionUpdater`; update `build()` |
| `resources/collection/service.rs` | `CollectionServiceHandle`: `Data<Database>` → `Arc<Database>` for `LikedSongIds`; update `build()` |
| `resources/setlist/service.rs` | `SetlistServiceHandle`: same pattern; update `build()` |
| `resources/blob/service.rs` | `BlobServiceHandle`: update `build()` to pass `Arc<Database>` into repos |
| `resources/team/service.rs` | `TeamServiceHandle`: update `build()` |
| `resources/user/service.rs` | `UserServiceHandle`: update `build()` |
| `resources/user/session/service.rs` | `SessionServiceHandle`: update `build()` |
| `resources/song/liked.rs` | `LikedSongIds` impl: change from `impl LikedSongIds for Data<Database>` to `impl LikedSongIds for Arc<Database>` |
| `main.rs` | Create `let db = Arc::new(database::Database::new().await?);` early, then wrap in `Data::new(db.clone())` only for `app_data`. Pass `Arc` to all `build()` calls instead of `Data`. |
| `test_helpers.rs` | Remove all `Data::from(db.clone())` → pass `Arc` directly. Massively simplifies every helper. |
| `benches/repo_perf.rs` | `SurrealSetlistRepo::new(db.clone())` instead of `Data::from(...)` |

**Steps:**

1. In every `surreal_repo.rs` and `resolver.rs`: replace `Data<Database>` with `Arc<Database>`, update `new()` signature, replace `self.db.get_ref()` with `self.db.as_ref()` (or just `&*self.db`), update imports.
2. Update `song/liked.rs`: change the `impl LikedSongIds for Data<Database>` to `impl LikedSongIds for Arc<Database>`.
3. Update `song/service.rs` `UserCollectionUpdater` impl: from `Data<SurrealUserRepo>` to `Arc<SurrealUserRepo>`.
4. Update every `*ServiceHandle` type alias and `build()` method to accept `Arc<Database>`.
5. Update `main.rs`: `let db = Arc::new(...)`, pass `Arc` into all `build()` calls, create `Data::new(db.clone())` only for `.app_data(...)`.
6. Update `test_helpers.rs`: drop all `Data::from(...)` calls; pass `db.clone()` directly.
7. `cargo test` — everything should compile and pass.

**Estimated effort:** ~1 hour of mechanical find-and-replace. No logic changes.

---

### Phase 2: `HttpResponse` Out of the Service Layer

**Goal:** Make export methods return domain types, not HTTP types.

**Why now:** Phase 1 removed actix from repos. This phase removes it from services. After this, the entire service + repo + model stack is framework-free.

**Files to change:**

| File | What changes |
|------|-------------|
| `resources/song/export.rs` | Split `export()` into two functions: a pure `export_bytes()` → `ExportResult` and the existing `export()` that wraps it in `HttpResponse`. The service calls `export_bytes()`, the rest handler calls the wrapper. |
| `resources/song/service.rs` | `export_song_for_user` returns `ExportResult` instead of `HttpResponse` |
| `resources/setlist/service.rs` | `export_setlist_for_user` returns `ExportResult` instead of `HttpResponse` |
| `resources/collection/service.rs` | `export_collection_for_user` returns `ExportResult` instead of `HttpResponse` |
| `resources/song/rest.rs` | `get_song_export` handler builds `HttpResponse` from `ExportResult` |
| `resources/setlist/rest.rs` | Same for setlist export handler |
| `resources/collection/rest.rs` | Same for collection export handler |

**Steps:**

1. Define `ExportResult` in `song/export.rs`:
   ```rust
   pub struct ExportResult {
       pub bytes: Vec<u8>,
       pub content_type: &'static str,
       pub filename: String,
   }
   ```
2. Refactor `export()` to return `ExportResult` internally. Keep a thin `export_http()` wrapper (or inline it in the rest handler) that converts to `HttpResponse`.
3. The `export_pdf` function already calls `Settings::global()` for the printer address — that's okay for now (Phase 5 addresses it). For this phase, just make it return `ExportResult` instead of `HttpResponse`.
4. Update the three service `export_*_for_user` methods to return `Result<ExportResult, AppError>`.
5. Update the three rest handlers to convert `ExportResult` → `HttpResponse`.
6. Remove `use actix_web::HttpResponse` from all three `service.rs` files.
7. `cargo test`

**Estimated effort:** ~30 minutes.

---

### Phase 3: Fix or Remove Stale Benchmarks

**Goal:** Make `cargo bench` compile again (or remove dead code).

**Files to change:**

| File | What changes |
|------|-------------|
| `benches/repo_perf.rs` | Fix references to match current API |

**Steps:**

1. Remove the `UserModel` import — it no longer exists.
2. Replace `db.create_user(User::new("bench@local"))` with the current user creation path. After Phase 1, you can use `UserServiceHandle::build(db.clone())` + `svc.create_user(User::new("bench@local"))` directly.
3. After Phase 1, `SurrealSetlistRepo::new(db.clone())` takes `Arc<Database>` directly — update accordingly.
4. `cargo bench` — verify it compiles and runs.
5. Consider adding a benchmark for `content_write_team_things` alongside the existing read benchmark, and one for `get_songs` with a populated dataset.

**Estimated effort:** ~15 minutes.

---

### Phase 4: Async Mail + Connection Pooling

**Goal:** Stop blocking the Tokio runtime on SMTP and stop creating a new connection per email.

**Files to change:**

| File | What changes |
|------|-------------|
| `Cargo.toml` | Add `lettre` feature: `features = ["tokio1-native-tls"]` (or `tokio1-rustls-tls` to match your TLS story) |
| `mail.rs` | Rewrite to use `AsyncSmtpTransport<Tokio1Executor>`, make `send()` async, accept config params instead of calling `Settings::global()` |
| `main.rs` | Build the `AsyncSmtpTransport` once at startup, wrap in `Data<Arc<...>>`, register as app_data |
| `auth/otp/rest.rs` | Extract the mail transport from app_data, call `mail.send().await` |

**Steps:**

1. Update `Cargo.toml` — add the `tokio1-native-tls` or `tokio1-rustls-tls` feature to `lettre`.
2. Rewrite `Mail` to be a thin struct around `AsyncSmtpTransport`. Constructor takes `from`, `credentials` — built once at startup. `send()` becomes `async fn send(&self, to, subject, body) -> Result<(), AppError>`.
3. In `main.rs`, build the transport:
   ```rust
   let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")?
       .credentials(Credentials::new(settings.gmail_from.clone(), settings.gmail_app_password.clone()))
       .build();
   let mail_service = Data::new(MailService::new(mailer, settings.gmail_from.clone()));
   ```
4. In `otp/rest.rs`, inject `Data<MailService>` and call `.send(...).await`.
5. This also eliminates the `Settings::global()` call inside `mail.rs` (one down for Phase 5).
6. `cargo test` — OTP tests that touch mail need the mock mailer pattern. If there are no existing OTP unit tests (there aren't — only integration YAML), this is safe.

**Estimated effort:** ~45 minutes.

---

### Phase 5: Make `Settings` Injectable

**Goal:** Remove `Settings::global()` from all production code paths. Settings becomes an explicit dependency.

**Why this order:** Phases 1-4 already eliminated several `Settings::global()` call sites (repos, services, mail). This phase tackles the remaining 11 callers.

**Current `Settings::global()` callers after Phases 1-4:**

| File | What it reads | Replacement strategy |
|------|--------------|---------------------|
| `database/mod.rs` (`Database::new()`) | `db_address`, `db_namespace`, etc. | **Delete `Database::new()`** — `main.rs` already calls `Database::connect()` with explicit params. The only user of `new()` is `main.rs` itself. Inline the settings read there. |
| `auth/middleware.rs` (`RequireUser`) | `cookie_name` | Extract a `CookieConfig { name }` struct, store in `Data<CookieConfig>`, read it from `req.app_data()` in the middleware. |
| `auth/rest.rs` (logout + `empty_cookie`) | `cookie_name`, `cookie_secure` | Same `CookieConfig` from app_data. The `empty_cookie()` fn takes `&CookieConfig`. |
| `auth/oidc/rest.rs` (callback + `session_cookie`) | `session_ttl_seconds`, `cookie_name`, `cookie_secure`, `post_login_path` | `CookieConfig` + `SessionConfig { ttl_seconds }` from app_data. The `resolve_frontend_redirect` already takes `&Settings` — change to take `post_login_path: &str`. |
| `auth/otp/rest.rs` (`otp_verify` + `session_cookie`) | `session_ttl_seconds`, `cookie_name`, `cookie_secure` | Same `CookieConfig` + `SessionConfig`. |
| `auth/otp/model.rs` (`otp_hmac` + `remember_otp`) | `otp_pepper`, `otp_ttl_seconds` | Refactor `Model` trait methods to accept `pepper: &str` and `ttl_seconds: u64` as params. Callers (`otp/rest.rs`) read from an `OtpConfig` in app_data. |
| `frontend.rs` (`scope` + `index_file`) | `static_dir` | `scope()` already captures the dir in a closure. Change `index_file()` to take `&str`. Thread the value from `main.rs`. |
| `resources/blob/storage.rs` (`FsBlobStorage::new()`) | `blob_dir` | `new(blob_dir: String)`. Caller (`blob/service.rs build()`) passes it. |
| `resources/song/export.rs` (`export_pdf`) | `printer_address`, `printer_api_key` | Accept as params or extract a `PrinterConfig` from app_data. Since `export_pdf` is called from the service layer (after Phase 2, it returns `ExportResult`), the cleanest path is for the service to accept a `PrinterConfig` or for the `export_bytes` function to take `printer_address` + `printer_api_key` as params. |
| `resources/user/session/rest.rs` | `session_ttl_seconds` | Same `SessionConfig` from app_data. |

**Steps:**

1. Define small config structs in `settings.rs` (keep the big `Settings` struct for `envy` parsing):
   ```rust
   #[derive(Clone)]
   pub struct CookieConfig {
       pub name: String,
       pub secure: bool,
   }

   #[derive(Clone)]
   pub struct SessionConfig {
       pub ttl_seconds: u64,
   }

   #[derive(Clone)]
   pub struct OtpConfig {
       pub ttl_seconds: u64,
       pub pepper: String,
   }

   #[derive(Clone)]
   pub struct PrinterConfig {
       pub address: String,
       pub api_key: String,
   }
   ```
   Add `impl Settings` methods to produce these from the parsed settings.
2. In `main.rs`, register them as `app_data`:
   ```rust
   .app_data(Data::new(settings.cookie_config()))
   .app_data(Data::new(settings.session_config()))
   .app_data(Data::new(settings.otp_config()))
   .app_data(Data::new(settings.printer_config()))
   ```
3. Update each caller file one at a time. Each file is independent — change, compile, test.
4. Delete `Database::new()`. Update `main.rs` to call `Database::connect()` directly with the settings fields.
5. Update `FsBlobStorage::new(blob_dir)` to take a `String` param. Update `BlobServiceHandle::build()`.
6. Update `frontend::rest::scope()` to accept `static_dir: &str` as a param from `main.rs`.
7. Once all callers are gone: delete `Settings::global()`, delete the `OnceCell`. Keep `Settings::init()` only for the `envy` parse in `main.rs` (or just inline `envy::from_env()` in main).
8. Update `test_helpers.rs`: remove `init_settings_for_files()` entirely. Instead, pass config values explicitly where needed (e.g., `FsBlobStorage::new(tempdir.path())`). Remove all `unsafe` env var manipulation.
9. `cargo test` — the test suite should be cleaner and safer now.

**Estimated effort:** ~2-3 hours. Largest phase, but each file is a small independent change.

---

### Phase 6: Push Team Filtering into the Database

**Goal:** `list_teams_for_user` should not fetch all teams, then filter in Rust.

**Files to change:**

| File | What changes |
|------|-------------|
| `resources/team/repository.rs` | Add `fetch_teams_for_user(user_id: &str, is_admin: bool) -> Result<Vec<TeamFetched>>` to the `TeamRepository` trait (or change `fetch_all_teams` to accept filter params) |
| `resources/team/surreal_repo.rs` | Implement the filtered query — reuse the pattern from `resolver.rs` |
| `resources/team/service.rs` | `list_teams_for_user` calls the new filtered method instead of `fetch_all_teams()` + Rust filter |

**Steps:**

1. Look at `content_read_team_things` in `resolver.rs` for the query pattern. The team list query needs something similar but returns full `TeamFetched` rows instead of just IDs:
   ```sql
   -- For admin:
   SELECT * FROM team FETCH owner, members.user
   -- For regular user:
   SELECT * FROM team WHERE owner = $user
     OR array::len(members[WHERE user = $user]) > 0
     FETCH owner, members.user
   ```
2. Add `fetch_teams_for_user` to `TeamRepository` trait and implement in `SurrealTeamRepo`.
3. Update `list_teams_for_user` in the service to call it. The Rust-side `can_read_team` filter can remain as a safety net (belt-and-suspenders), but the DB query does the heavy lifting.
4. `cargo test` — the existing team tests validate the behavior.
5. Consider adding a test similar to the resolver's "matches naive filter" pattern: run the old `fetch_all_teams` + filter path alongside the new query and assert identical results.

**Estimated effort:** ~30 minutes.

---

### Phase 7: Return `&[Thing]` from `UserPermissions`

**Goal:** Eliminate unnecessary `Vec::clone()` on every `read_teams()` / `write_teams()` call.

**Files to change:**

| File | What changes |
|------|-------------|
| `resources/team/resolver.rs` | `read_teams()` returns `&Vec<Thing>` (or `&[Thing]`), `write_teams()` same |
| Every `repository.rs` trait | Change `read_teams: Vec<Thing>` → `read_teams: &[Thing]` in method signatures |
| Every `surreal_repo.rs` | Update `.bind("teams", ...)` to clone/collect the slice into a Vec for the query bind (SurrealDB bind needs owned values) |
| Every `service.rs` | Remove `.clone()` where teams were cloned to pass ownership |

**Steps:**

1. Change `UserPermissions::read_teams()` to return `Result<&[Thing], AppError>`. The `OnceCell::get_or_try_init` returns `&Vec<Thing>` — return it as `&[Thing]`.
2. Update all repository trait methods: `read_teams: Vec<Thing>` → `read_teams: &[Thing]`.
3. In each `surreal_repo.rs`, the `.bind("teams", read_teams)` call needs an owned `Vec<Thing>`. Add `.bind("teams", read_teams.to_vec())` — this is the *only* place the clone happens now, right before the DB call, not in the resolver.
4. For `get_song` / `get_setlist` / etc. that call `belongs_to(&r.owner, &read_teams)`: these already take `&[Thing]`, no change needed.
5. `cargo test`

**Net effect:** One clone per DB call (unavoidable — SurrealDB needs owned values) instead of one clone per `read_teams()` invocation. When a service method calls `read_teams()` then passes it to the repo, the clone happens in the repo, not in the resolver. When a request handler calls multiple service methods, the resolver returns the same borrowed slice each time instead of cloning.

**Estimated effort:** ~45 minutes. Mechanical but touches many files.

---

### Phase 8: Document the Upsert in `update_song`

**Goal:** Make the hidden upsert behavior explicit and intentional.

**Files to change:**

| File | What changes |
|------|-------------|
| `resources/song/surreal_repo.rs` | Add doc comment to `update_song` explaining the upsert fallback |
| `resources/song/repository.rs` | Add doc comment to the trait method clarifying the contract |

**Steps:**

1. Add to `SongRepository::update_song` in `repository.rs`:
   ```rust
   /// Update an existing song. If the song doesn't exist but the user has
   /// write access, creates it with the given ID (upsert semantics).
   /// This supports import/sync workflows where the caller specifies the ID.
   ```
2. Add a corresponding implementation note in `surreal_repo.rs` explaining the three-step logic (try UPDATE → check existence → CREATE fallback).
3. If the upsert is *not* intentional, add a `// TODO:` instead and consider removing the fallback path.

**Estimated effort:** ~5 minutes.

---

### Phase 9: Split Large Integration Tests

**Goal:** Make test failures easier to diagnose by splitting monolithic tests.

**Files to change:**

| File | What changes |
|------|-------------|
| `resources/setlist/service.rs` (tests) | Split `blc_setl_009_create_owner_and_title` (~300 lines) into focused tests |

**Steps:**

1. Extract the shared fixture (`four_user_setlist_fixture`) into a top-level helper in the test module (it's already duplicated between two tests — deduplicate it).
2. Split the monolith into focused tests, each testing one concern:
   - `blc_setl_create_owner_and_title` — create, assert owner and title
   - `blc_setl_list_pagination` — list with page/page_size combos
   - `blc_setl_search` — list with `q` parameter
   - `blc_setl_get_acl` — get for owner, reader, no-perm, bad-id, not-found
   - `blc_setl_songs_acl` — setlist_songs for owner, reader, no-perm
   - `blc_setl_player_acl` — player for owner, reader, no-perm
   - `blc_setl_export_acl` — export for owner, reader, no-perm
   - `blc_setl_update_acl` — update for owner, writer, reader (rejected), no-perm (rejected)
   - `blc_setl_delete_acl` — delete for writer, owner, re-delete (not found)
3. Each test creates its own fixture (or shares a lazy-init fixture via a helper). The per-test DB is cheap (in-memory).
4. `cargo test` — all should pass. Run count should increase but wall-clock time stays similar (tests run in parallel).

**Estimated effort:** ~1 hour. No logic changes, just restructuring.

---

### Phase 10: Add HTTP-Level Handler Tests

**Goal:** Test the actix wiring — extractors, response status codes, JSON serialization, error mapping.

**Files to change:**

| File | What changes |
|------|-------------|
| `resources/song/rest.rs` (new `#[cfg(test)]` module) | Add tests using `actix_web::test` |
| `resources/team/rest.rs` (new `#[cfg(test)]` module) | Same |
| (optional) other rest.rs files | Same pattern |

**Steps:**

1. Start with `song/rest.rs` as the template. Add a `#[cfg(test)]` module at the bottom.
2. Use `actix_web::test::init_service` with a mini `App` that has the services wired up:
   ```rust
   let db = test_db().await.unwrap();
   let song_svc = SongServiceHandle::build(db.clone());
   let session_svc = SessionServiceHandle::build(db.clone());
   // ... create a test user + session ...
   let app = test::init_service(
       App::new()
           .app_data(Data::new(song_svc))
           .app_data(Data::new(session_svc))
           .app_data(Data::from(db))
           .service(scope().wrap(RequireUser))
   ).await;
   ```
3. Write a few representative tests:
   - `GET /songs` with valid session → 200 + JSON array
   - `GET /songs` without session → 401
   - `GET /songs/{id}` with non-existent ID → 404 + JSON error body
   - `POST /songs` with valid payload → 201 + JSON body with `id`
   - `POST /songs` with malformed JSON → 400
4. These tests verify that the full middleware → handler → service → repo → DB chain works end-to-end within `cargo test`, without needing external test runners.
5. Add a similar small set for `team/rest.rs` (create shared team, accept invitation).

**Estimated effort:** ~1.5 hours for the first resource, ~30 min for each additional.

---

### Summary: Execution Order and Total Effort

| Phase | Description | Effort | Depends on |
|-------|------------|--------|-----------|
| **1** | `Data<Database>` → `Arc<Database>` | ~1h | — |
| **2** | `HttpResponse` out of services | ~30m | — |
| **3** | Fix stale benchmarks | ~15m | Phase 1 |
| **4** | Async mail + pooling | ~45m | — |
| **5** | Injectable `Settings` | ~2-3h | Phases 1, 2, 4 |
| **6** | Push team filtering to DB | ~30m | — |
| **7** | `&[Thing]` from `UserPermissions` | ~45m | — |
| **8** | Document the upsert | ~5m | — |
| **9** | Split large tests | ~1h | — |
| **10** | HTTP-level handler tests | ~2h | Phases 1, 5 |

**Total: ~9-10 hours of focused work.**

Phases 1, 2, 4, 6, 7, 8, 9 are all independent and can be done in any order or in parallel. Phase 5 is the big one and benefits from 1, 2, 4 being done first. Phase 10 benefits from everything else being done.

A reasonable sprint plan:
- **Day 1:** Phases 1 + 2 + 3 + 8 (mechanical refactors, ~2h)
- **Day 2:** Phase 5 (the big settings refactor, ~2-3h)
- **Day 3:** Phases 4 + 6 + 7 (performance, ~2h)
- **Day 4:** Phases 9 + 10 (test quality, ~3h)
