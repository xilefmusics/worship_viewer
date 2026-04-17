# Backend Request Flow

End-to-end path of an HTTP request through the system — from Actix
entry point to SurrealDB and back.

---

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│  HTTP Request                                                                   │
└──────────┬──────────────────────────────────────────────────────────────────────┘
           │
           ▼
┌──────────────────────┐
│  RequireUser          │  auth middleware — validates session, injects ReqData<User>
│  middleware            │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│  resources/rest.rs  →  /api/v1                                                   │
│                                                                                  │
│  Mounts per-resource scopes:                                                     │
│  ┌──────────┐ ┌──────────────┐ ┌───────────┐ ┌──────┐ ┌──────┐ ┌──────┐        │
│  │ /blobs   │ │ /collections │ │ /setlists │ │/songs│ │/teams│ │/users│        │
│  │ blob::   │ │ collection:: │ │ setlist:: │ │song::│ │team::│ │user::│        │
│  │ rest     │ │ rest         │ │ rest      │ │rest  │ │rest  │ │rest  │        │
│  └────┬─────┘ └──────┬───────┘ └─────┬─────┘ └──┬───┘ └──┬───┘ └──┬───┘        │
└───────┼──────────────┼───────────────┼──────────┼────────┼────────┼─────────────┘
        │              │               │          │        │        │
        ▼              ▼               ▼          ▼        ▼        ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│  Service Layer  (generic structs, e.g. SongService<R, T, L, C, U>)               │
│                                                                                  │
│  ┌───────────┐ ┌─────────────────┐ ┌────────────────┐ ┌───────────┐             │
│  │BlobService│ │CollectionService│ │SetlistService  │ │SongService│  ...         │
│  │ <R, T, S> │ │ <R, T, L>      │ │ <R, T, L>      │ │<R,T,L,C,U>│             │
│  └──┬──┬──┬──┘ └──┬──┬──┬───────┘ └──┬──┬──┬───────┘ └┬──┬──┬─┬─┘             │
│     │  │  │       │  │  │            │  │  │          │  │  │ │               │
└─────┼──┼──┼───────┼──┼──┼────────────┼──┼──┼──────────┼──┼──┼─┼───────────────┘
      │  │  │       │  │  │            │  │  │          │  │  │ │
      │  │  │       │  │  │            │  │  │          │  │  │ └─► UserRepository
      │  │  │       │  │  │            │  │  │          │  │  └──► CollectionRepository
      │  │  │       │  │  └────────────┼──┼──┘          │  │
      │  │  │       │  │               │  │             │  └──────► LikedSongIds
      │  │  │       │  └───────────────┼──┘             │
      │  │  │       │                  │                └─────────► *Repository trait
      │  │  └───────┼──────────────────┼──► BlobStorage trait
      │  └──────────┴──────────────────┴──► TeamResolver trait
      └─────────────────────────────────► *Repository trait
                                          │
                                          ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│  Repository Layer  (trait objects + SurrealDB implementations)                    │
│                                                                                  │
│  SurrealBlobRepo   SurrealCollectionRepo   SurrealSetlistRepo   SurrealSongRepo  │
│  SurrealTeamRepo   SurrealTeamInvitationRepo   SurrealUserRepo  SurrealSessionRepo│
│  SurrealTeamResolver   FsBlobStorage                                             │
└──────────┬───────────────────────────────────────────────────────────────────────┘
           │
           ▼
┌──────────────────────┐     ┌──────────────────────┐
│  SurrealDB (Database)│     │  Filesystem (blobs)   │
└──────────────────────┘     └──────────────────────┘
```

---

## Step-by-Step Request Lifecycle

### 1. Actix receives the HTTP request

`main.rs` builds the `App`, registers all `ServiceHandle`s as `app_data`,
and mounts `resources::rest::scope()` under `/api/v1`.

### 2. `RequireUser` middleware

Validates the session cookie/token and injects `ReqData<User>` into the
request extensions. If validation fails, the request is rejected before
reaching any handler.

### 3. `resources/rest.rs` routes to a resource scope

All resource scopes are mounted here:

```rust
web::scope("/api/v1")
    .wrap(RequireUser)
    .service(blob::rest::scope())       // /api/v1/blobs
    .service(collection::rest::scope()) // /api/v1/collections
    .service(setlist::rest::scope())    // /api/v1/setlists
    .service(song::rest::scope())       // /api/v1/songs
    .service(team::rest::scope())       // /api/v1/teams
    .service(team::invitations_accept_scope()) // /api/v1/invitations
    .service(user::rest::scope())       // /api/v1/users (nests session routes)
```

### 4. Resource `rest.rs` handler

The handler extracts typed data from the request, constructs a
`UserPermissions` wrapper, and makes a single service call:

```rust
#[get("/{id}")]
async fn get_one(
    svc: Data<XxxServiceHandle>,
    user: ReqData<User>,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.teams);
    let result = svc.get_one(&perms, &path).await?;
    Ok(HttpResponse::Ok().json(result))
}
```

`UserPermissions` is a lightweight struct defined in `team/resolver.rs`
that wraps `&User` and `&T: TeamResolver` with three `tokio::sync::OnceCell`
fields. Team lists are resolved lazily on first access and cached for the
lifetime of the struct (one per HTTP request).

### 5. `UserPermissions` resolves ACL via `TeamResolver`

The service receives `&UserPermissions<T>` and calls its lazy accessor
methods to get the teams the user is allowed to read from or write to:

| Operation | `UserPermissions` method | Underlying `TeamResolver` call | Returns |
|---|---|---|---|
| List / Get | `perms.read_teams().await?` | `content_read_teams(&user)` | `[team:public, team:<personal>, team:<memberships...>]` |
| Create | `perms.user().id` | — (no team resolution needed) | User's ID string |
| Update / Delete | `perms.write_teams().await?` | `content_write_teams(&user)` | `[team:<personal>, team:<admin/maintainer memberships>]` |
| Delete personal team | `perms.personal_team().await?` | `personal_team(&user_id)` | Single `Thing` for the user's personal team |

Each method resolves the value on first call and returns a cached clone
on subsequent calls within the same request, eliminating duplicate
database round-trips.

### 6. Repository executes the query

The service delegates to the repository trait method with the resolved
teams. The SurrealDB implementation runs a query like:

```sql
SELECT * FROM xxx WHERE owner IN $teams
```

For single-record lookups without a `WHERE` clause, the Rust-side
`belongs_to` helper performs the ownership check after the `SELECT`.

### 7. Record → DTO conversion

The `SurrealXxxRepo` receives a `XxxRecord` from SurrealDB and converts
it into the shared `Xxx` DTO via `into_xxx()`.

### 8. Response

The handler wraps the DTO in an `HttpResponse` with the appropriate
status code:

| Action | Status |
|---|---|
| List / Get | `200 OK` |
| Create | `201 Created` |
| Update | `200 OK` |
| Delete | `200 OK` (returns the deleted entity) |

---

## Authorization Flow (detailed)

```
HTTP request
  │
  ▼
RequireUser middleware ─── validates session cookie/token, injects ReqData<User>
  │
  ▼
rest.rs handler
  │  constructs UserPermissions::new(&user, &svc.teams)
  │
  ▼
UserPermissions<T>  (per-request caching wrapper in team/resolver.rs)
  │
  ├── .read_teams()  ─── lazy, cached via OnceCell ──► TeamResolver.content_read_teams(&user)
  │                      first call: DB query          returns Vec<Thing>:
  │                      subsequent: cached clone      [team:public, team:<personal>, team:<memberships...>]
  │
  ├── .write_teams() ─── lazy, cached via OnceCell ──► TeamResolver.content_write_teams(&user)
  │                      first call: DB query          returns Vec<Thing>:
  │                      subsequent: cached clone      [team:<personal>, team:<admin/maintainer memberships>]
  │
  └── .personal_team() ─ lazy, cached via OnceCell ──► TeamResolver.personal_team(&user_id)
                         first call: DB query          returns single Thing
                         subsequent: cached clone
  │
  ▼
Service method(perms, ...)
  │
  └── Repository method(teams, ...)
      └─ SurrealQL: WHERE owner IN $teams
```

`TeamResolver` is implemented by `SurrealTeamResolver` in
`team/resolver.rs`. The free functions `content_read_team_things()` and
`content_write_team_things()` are convenience wrappers used internally.
`UserPermissions` is the public API that handlers and services use — it
provides lazy caching so a team-list DB query is only executed once per
request even if multiple service methods call the same accessor.

---

## Top-Level Wiring Files

### `resources/mod.rs`

Declares every resource as a public sub-module and re-exports their shared
DTOs for ergonomic access from the rest of the crate.

| Declaration | Purpose |
|---|---|
| `pub mod rest` | Top-level API scope aggregator |
| `pub(crate) mod common` | Shared helpers used across resources |
| `pub mod blob`, `collection`, `setlist`, `song`, `team`, `user` | Resource sub-modules |
| `pub use blob::{Blob, CreateBlob}`, ... | Re-exports shared DTOs |

### `resources/rest.rs`

Creates the `/api/v1` scope, wraps it with `RequireUser`, and mounts
every resource's `rest::scope()`. Team invitation accept is mounted
separately at `/api/v1/invitations/...`.

### `resources/common.rs`

Shared helper functions and DB record types used by multiple resources.

| Helper | Purpose |
|---|---|
| `resource_id(table, id)` | Parse/validate a SurrealDB record ID string |
| `belongs_to(owner, teams)` | Rust-side ownership check for single-record SELECTs |
| `song_thing(id)` / `blob_thing(id)` | Coerce a string into a typed `Thing` |
| `player_from_song_links(liked, links)` | Build a `Player` from fetched song links |
| `SongLinkRecord` | DB record shape for embedded song references |
| `FetchedSongRecord` | Fully-fetched song record (via SurrealDB `FETCH`) |

---

## Per-Resource Specifics

### `blob/` — Binary file storage

| File | Key Types |
|---|---|
| `model.rs` | `BlobRecord` |
| `repository.rs` | `trait BlobRepository` |
| `surreal_repo.rs` | `SurrealBlobRepo` |
| `storage.rs` | `trait BlobStorage`, `FsBlobStorage` |
| `service.rs` | `BlobService<R, T, S>` where `S: BlobStorage` |
| `rest.rs` | `/blobs` scope; file download returns `NamedFile` |

Unique: takes a `BlobStorage` backend in addition to the standard repo +
team resolver. The download handler returns `actix_files::NamedFile`
rather than JSON.

### `song/` — Song sheets with chords/lyrics

| File | Key Types |
|---|---|
| `model.rs` | `SongRecord`, `LikeRecord` |
| `repository.rs` | `trait SongRepository` |
| `surreal_repo.rs` | `SurrealSongRepo` |
| `liked.rs` | `trait LikedSongIds` |
| `service.rs` | `SongService<R, T, L, C, U>` |
| `rest.rs` | `/songs` scope |

Most interconnected service: creating a song auto-adds it to the user's
default collection via `CollectionRepository` and `UserCollectionUpdater`.

### `collection/` — Groupings of songs

| File | Key Types |
|---|---|
| `model.rs` | `CollectionRecord`, `CollectionSongsRecord` |
| `repository.rs` | `trait CollectionRepository` (includes `add_song_to_collection`) |
| `surreal_repo.rs` | `SurrealCollectionRepo` |
| `service.rs` | `CollectionService<R, T, L>` |
| `rest.rs` | `/collections` scope |

`CollectionRepository` is also a dependency of `SongService` for the
default-collection auto-add. Uses `LikedSongIds`.

### `setlist/` — Ordered song lists for worship sessions

| File | Key Types |
|---|---|
| `model.rs` | `SetlistRecord`, `SetlistSongsRecord` |
| `repository.rs` | `trait SetlistRepository` |
| `surreal_repo.rs` | `SurrealSetlistRepo` |
| `service.rs` | `SetlistService<R, T, L>` |
| `rest.rs` | `/setlists` scope |

Structurally identical to collection. Depends on `SetlistRepository`,
`TeamResolver`, and `LikedSongIds`.

### `team/` — Teams, membership, and ACL

| File | Key Types |
|---|---|
| `model.rs` | `TeamCreatePayload`, `DbTeamMember`, `TeamFetched`, ACL helpers |
| `repository.rs` | `trait TeamRepository` |
| `surreal_repo.rs` | `SurrealTeamRepo` |
| `resolver.rs` | `trait TeamResolver`, `SurrealTeamResolver`, `UserPermissions`, `content_read_team_things()`, `content_write_team_things()` |
| `invitation_model.rs` | `InvitationRow`, `InvitationAcceptRow`, helpers |
| `invitation_repository.rs` | `trait TeamInvitationRepository` |
| `invitation_surreal_repo.rs` | `SurrealTeamInvitationRepo` |
| `service.rs` | `TeamService<R, IR, TR>` |
| `rest.rs` | `/teams` scope + `/invitations/{id}/accept` |

`TeamResolver` is the most depended-upon trait — every content service
requires it. `UserService` depends on `TeamRepository` to create personal
teams on user registration.

### `user/` — User accounts (admin-only CRUD)

| File | Key Types |
|---|---|
| `model.rs` | `UserRecord` |
| `repository.rs` | `trait UserRepository` |
| `surreal_repo.rs` | `SurrealUserRepo` |
| `service.rs` | `UserService<R, T>` where `T: TeamRepository` |
| `rest.rs` | `/users` scope, `/users/me`; nests session routes; admin routes use `RequireAdmin` |

### `user/session/` — Login sessions

| File | Key Types |
|---|---|
| `model.rs` | `SessionRecord`, `SessionCreateRecord` |
| `repository.rs` | `trait SessionRepository` |
| `surreal_repo.rs` | `SurrealSessionRepo` |
| `service.rs` | `SessionService<S, U>` where `U: UserRepository` |
| `rest.rs` | Session routes (mounted inside `user/rest.rs`) |

`SessionServiceHandle` is also used by `auth/rest.rs` for the logout flow.

---

## Cross-Resource Dependency Graph

```
                    ┌──────────────┐
                    │ TeamResolver │ (resolver.rs)
                    └──────┬───────┘
          ┌────────────────┼────────────────┬────────────────┐
          ▼                ▼                ▼                ▼
   ┌─────────────┐  ┌───────────┐  ┌──────────────┐  ┌───────────┐
   │ BlobService │  │SongService│  │CollectionSvc │  │SetlistSvc │
   └──────┬──────┘  └─────┬─────┘  └──────┬───────┘  └─────┬─────┘
          │               │               │                │
          ▼               │               ▼                ▼
   ┌─────────────┐        │        ┌──────────────┐  ┌───────────┐
   │ BlobStorage │        │        │CollectionRepo│  │SetlistRepo│
   └─────────────┘        │        └──────────────┘  └───────────┘
                          │               ▲
                          │               │ (auto-add song to default collection)
                          ├───────────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │LikedSongIds │
                   └──────┬──────┘
                          │
          ┌───────────────┤
          ▼               ▼
   CollectionService  SetlistService


   ┌─────────────┐          ┌─────────────────┐
   │ UserService │───────►  │ TeamRepository  │  (creates personal team)
   └──────┬──────┘          └─────────────────┘
          │
          ▼
   ┌──────────────┐         ┌─────────────────┐
   │SessionService│────────►│ UserRepository  │  (lookup user by ID)
   └──────────────┘         └─────────────────┘
```

---

## Resource Comparison Matrix

| | blob | song | collection | setlist | team | user | session |
|---|---|---|---|---|---|---|---|
| **Shared DTO** | yes | yes | yes | yes | yes | yes | yes |
| **Repository trait** | `BlobRepository` | `SongRepository` | `CollectionRepository` | `SetlistRepository` | `TeamRepository` + `TeamInvitationRepository` | `UserRepository` | `SessionRepository` |
| **Surreal impl** | `SurrealBlobRepo` | `SurrealSongRepo` | `SurrealCollectionRepo` | `SurrealSetlistRepo` | `SurrealTeamRepo` + `SurrealTeamInvitationRepo` | `SurrealUserRepo` | `SurrealSessionRepo` |
| **Service** | `BlobService<R,T,S>` | `SongService<R,T,L,C,U>` | `CollectionService<R,T,L>` | `SetlistService<R,T,L>` | `TeamService<R,IR,TR>` | `UserService<R,T>` | `SessionService<S,U>` |
| **Team-scoped** | yes | yes | yes | yes | own ACL | no | no |
| **Extra files** | `storage.rs` | `liked.rs` | — | — | `resolver.rs`, `invitation_model.rs`, `invitation_repository.rs`, `invitation_surreal_repo.rs` | — | — |
| **Extra dependencies** | `BlobStorage` | `LikedSongIds`, `CollectionRepo`, `UserCollectionUpdater` | `LikedSongIds` | `LikedSongIds` | `TeamResolver` | `TeamRepository` | `UserRepository` |
