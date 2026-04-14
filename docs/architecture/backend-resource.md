# Backend Resource Architecture

Every backend resource follows the same layered pattern, split across a
**shared crate** (DTOs) and a **resource module** inside `backend/src/resources/<name>/`.

---

## High-Level Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│  shared/src/<name>/                                                │
│  ┌──────────┐  ┌──────────────┐                                   │
│  │  mod.rs   │  │  <name>.rs   │   Platform-agnostic DTOs          │
│  │ re-export │  │  Xxx         │   (used by frontend + backend)    │
│  │          │  │  CreateXxx   │                                   │
│  └──────────┘  └──────┬───────┘                                   │
└───────────────────────┼───────────────────────────────────────────┘
                        │  pub use
                        ▼
┌────────────────────────────────────────────────────────────────────┐
│  backend/src/resources/<name>/                                     │
│                                                                    │
│  ┌──────────┐                                                      │
│  │  mod.rs   │  re-exports DTOs, declares sub-modules              │
│  └────┬─────┘                                                      │
│       │ declares                                                   │
│       ├──────────────────────────────────────┐                     │
│       ▼                                      ▼                     │
│  ┌──────────┐    defines trait          ┌──────────┐               │
│  │ model.rs │◄──────────────────────────│ rest.rs  │               │
│  │          │                           │          │               │
│  │ Record   │    ┌───────────────┐      │ scope()  │               │
│  │ struct   │◄───│repository.rs  │      │ handlers │               │
│  └──────────┘    │               │      └────┬─────┘               │
│                  │ trait          │           │                     │
│                  │ Repository     │           │ calls               │
│                  └───────┬───────┘           ▼                     │
│                          │           ┌─────────────┐               │
│                implements│           │ service.rs   │               │
│                          ▼           │              │               │
│                  ┌───────────────┐   │ Service<R,T> │               │
│                  │surreal_repo.rs│◄──│ orchestrates │               │
│                  │               │   │ ACL + repo   │               │
│                  │ SurrealRepo   │   └──────┬───────┘               │
│                  └───────┬───────┘          │                      │
│                          │                  │ depends on            │
└──────────────────────────┼──────────────────┼──────────────────────┘
                           │                  │
                           ▼                  ▼
                  ┌──────────────┐   ┌──────────────────────────────────────┐
                  │   Database   │   │   team/resolver.rs                   │
                  │  (SurrealDB) │   │   TeamResolver trait (ACL resolution) │
                  └──────────────┘   │   UserPermissions<T>  (lazy cache    │
                                     │     wrapper per HTTP request)         │
                                     └──────────────────────────────────────┘
```

---

## File Roles

### `shared/src/<name>/<name>.rs` — DTO

Platform-agnostic types shared between frontend (WASM) and backend.
Every resource defines at least two structs:

| Struct | Purpose |
|---|---|
| `Xxx` | Response DTO — includes `id` and `owner` |
| `CreateXxx` | Request body DTO — excludes `id` and `owner` |

- Derives `Serialize`, `Deserialize`, `Clone`.
- Backend feature-gates `utoipa::ToSchema` for OpenAPI generation.
- `From<Xxx> for CreateXxx` strips server-assigned fields for convenience.

### `shared/src/<name>/mod.rs` — Re-exports

Barrel file that re-exports the DTOs. No logic.

---

### `mod.rs` — Module Root

Re-exports the shared DTOs from the shared crate and declares all
sub-modules. This is the public surface of the resource within the backend
crate.

```rust
pub use shared::<name>::{Xxx, CreateXxx};

mod model;
mod repository;
mod surreal_repo;
pub mod service;
pub mod rest;

pub use repository::XxxRepository;
pub use service::{XxxService, XxxServiceHandle};
pub use surreal_repo::SurrealXxxRepo;
```

Every resource's `mod.rs` follows this shape. The exact re-exports vary
but the structure is consistent.

---

### `model.rs` — Database Record

Contains the private struct that maps 1:1 to the database table, plus
conversion methods bridging between the record and the shared DTO.

```rust
struct XxxRecord {
    id:    Option<Thing>,
    owner: Option<Thing>,
    // ... domain fields ...
}
```

| Method | Direction |
|---|---|
| `into_xxx(self) -> Xxx` | DB record → API response |
| `from_payload(id, owner, payload) -> Self` | API request → DB insert |

The record is private to the resource module — only `surreal_repo.rs` uses
it directly.

---

### `repository.rs` — Repository Trait

Defines an async trait with CRUD methods. Methods receive **pre-resolved**
`Vec<Thing>` teams, keeping authorization concerns out of the data access
layer.

```rust
pub trait XxxRepository {
    async fn get_all(&self, read_teams: Vec<Thing>, pagination: ListQuery) -> Result<Vec<Xxx>>;
    async fn get_one(&self, read_teams: Vec<Thing>, id: &str) -> Result<Xxx>;
    async fn create(&self, owner: &str, payload: CreateXxx) -> Result<Xxx>;
    async fn update(&self, write_teams: Vec<Thing>, id: &str, payload: CreateXxx) -> Result<Xxx>;
    async fn delete(&self, write_teams: Vec<Thing>, id: &str) -> Result<Xxx>;
}
```

This file has **no** database dependency — it's a pure interface. This
enables unit-testing services with mock repositories.

---

### `surreal_repo.rs` — SurrealDB Implementation

Implements the repository trait against the `Database` connection. Contains
all SurrealQL queries.

Standard CRUD queries follow this convention:

| Op | SurrealQL pattern |
|---|---|
| List | `SELECT * FROM <table> WHERE owner IN $teams` |
| Get | `db.select(resource_id(...))` + Rust-side `belongs_to` check |
| Create | `db.create("<table>").content(Record::from_payload(...))` |
| Update | `UPDATE type::thing($tb, $sid) SET ... WHERE owner IN $teams RETURN AFTER` |
| Delete | `DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE` |

**Depends on:** `Database`, `model.rs` (record type), `repository.rs`
(trait), `common` (helpers like `resource_id`, `belongs_to`).

---

### `service.rs` — Service Layer

A generic struct parameterised over trait bounds. Orchestrates:

1. **ACL resolution** — uses `UserPermissions` to lazily resolve team lists via `TeamResolver`
2. **Delegation** — calls the repository with resolved teams
3. **Cross-resource logic** — any side effects involving other repositories

```rust
pub struct XxxService<R: XxxRepository, T: TeamResolver, ...> {
    pub repo: R,
    pub teams: T,          // TeamResolver implementation
    // ... additional trait-bounded dependencies ...
}
```

Service methods accept `&UserPermissions<'_, T>` instead of a bare `&User`,
giving them access to both the user and lazily-cached team lists:

```rust
pub async fn get_one_for_user(
    &self,
    perms: &UserPermissions<'_, T>,
    id: &str,
) -> Result<Xxx, AppError> {
    let read_teams = perms.read_teams().await?;   // lazy, cached
    self.repo.get_one(read_teams, id).await
}
```

A type alias wires concrete implementations for use in Actix `app_data`:

```rust
pub type XxxServiceHandle = XxxService<SurrealXxxRepo, SurrealTeamResolver, ...>;
```

A `build()` or `new()` constructor takes the concrete dependencies and
returns the handle.

**Depends on:** `repository.rs` (trait), `team::TeamResolver` (via
`UserPermissions`), and optionally other resource repository traits for
cross-resource operations.

---

### `rest.rs` — HTTP Handlers

Exposes a single `scope()` function returning an Actix `Scope`. Handlers
are intentionally thin: extract → construct `UserPermissions` → call service → return JSON.

```rust
pub fn scope() -> Scope {
    web::scope("/<name>s")
        .service(get_all)
        .service(get_one)
        .service(create)
        .service(update)
        .service(delete)
}
```

Each handler:

1. Extracts `Data<XxxServiceHandle>`, `ReqData<User>`, and optionally
   `Path`, `Query`, `Json`.
2. Constructs `UserPermissions::new(&user, &svc.teams)` — a zero-cost
   wrapper that lazily resolves and caches team lists on first access.
3. Calls exactly one service method, passing `&perms`.
4. Returns `Result<HttpResponse, AppError>`.
5. Is annotated with `#[utoipa::path(...)]` for OpenAPI generation.

```rust
async fn get_one(
    svc: Data<XxxServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    Ok(HttpResponse::Ok().json(svc.get_one_for_user(&perms, &id).await?))
}
```

Standard HTTP status mapping:

| Action | Success Status |
|---|---|
| List / Get | `200 OK` |
| Create | `201 Created` |
| Update | `200 OK` |
| Delete | `200 OK` (returns the deleted entity) |

**Depends on:** `XxxServiceHandle` (via `Data<>`), shared DTOs, `User`
(via `ReqData<>`), `team::UserPermissions`.

---

## Shared Helpers (`resources/common.rs`)

Helper functions and DB record types reused across multiple resources.

| Helper | Purpose |
|---|---|
| `resource_id(table, id)` | Parse and validate an ID string, accepting both plain IDs and `table:id` format |
| `belongs_to(owner, teams)` | Check if an `Option<Thing>` owner is within a team list |
| `*_thing(id)` helpers | Coerce a string into a typed `Thing` for a known table |
| `SongLinkRecord` | DB record shape for embedded song references |
| `FetchedSongRecord` | Fully-fetched song record (via SurrealDB `FETCH`) |

---

## Adding a New Resource — Checklist

1. **`shared/src/<name>/`** — Define `Xxx` and `CreateXxx` DTOs.
2. **`backend/src/resources/<name>/mod.rs`** — Re-export DTOs, declare sub-modules, re-export public types.
3. **`backend/src/resources/<name>/model.rs`** — Define `XxxRecord` with DB-to-DTO conversion methods.
4. **`backend/src/resources/<name>/repository.rs`** — Define `trait XxxRepository` with async CRUD methods.
5. **`backend/src/resources/<name>/surreal_repo.rs`** — `SurrealXxxRepo` implementing the repository trait.
6. **`backend/src/resources/<name>/service.rs`** — `XxxService<R, T, ...>` generic over repository + `TeamResolver`. Define `XxxServiceHandle` type alias.
7. **`backend/src/resources/<name>/rest.rs`** — `scope()` + CRUD handlers with utoipa annotations.
8. **`backend/src/resources/mod.rs`** — Add `pub mod <name>;` and re-exports.
9. **`backend/src/resources/rest.rs`** — Mount `.service(<name>::rest::scope())`.
10. **`backend/src/main.rs`** — Build service handle, register as `app_data`.
11. **Database migration** — Add SurrealQL `DEFINE TABLE` / `DEFINE FIELD` statements.
