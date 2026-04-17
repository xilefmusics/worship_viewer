# REST API Review — `worship_viewer` backend

Date: 2026-04-17
Scope: the HTTP surface exposed by the `backend` crate
(`/auth`, `/api/v1/*`, `/api/docs/*`, SPA fallback), as defined in:

- `backend/src/main.rs`
- `backend/src/resources/rest.rs`
- `backend/src/resources/{blob,collection,setlist,song,team,user}/rest.rs`
- `backend/src/resources/team/invitation/rest.rs`
- `backend/src/resources/user/session/rest.rs`
- `backend/src/auth/{rest.rs,middleware.rs,oidc/rest.rs,otp/rest.rs}`
- `backend/src/error.rs`
- `backend/src/docs.rs`
- `shared/src/api/list_query.rs`
- `shared/src/error/mod.rs`
- plus supporting services and BLCs in `docs/business-logic-constraints/*`

---

## 1. Executive summary

The API has a **coherent, layered architecture** (rest → service → repository →
Surreal) with good separation of concerns, consistent per-resource conventions,
good use of traits for testability, and OpenAPI coverage on every route.
Authentication is cleanly expressed as two tiers of middleware
(`RequireUser`, `RequireAdmin`) and the ACL model (team-based, with lazy,
cached permission resolution via `UserPermissions`) is elegant.

That said, the surface has **several concrete inconsistencies and a few real
correctness/security smells** that deserve attention. The most important ones:

| # | Severity | Issue | Location |
|---|---|---|---|
| S1 | **Security (IDOR)** | `DELETE /api/v1/users/me/sessions/{id}` does not scope by caller — any authenticated user can delete any other user's session by id. | `backend/src/resources/user/session/rest.rs:82` |
| S2 | Medium | Admin `GET /users/{user_id}/sessions/{id}` and `DELETE /users/{user_id}/sessions/{id}` ignore the `user_id` path segment. | same file, lines 168, 196 |
| S3 | Medium | 400 responses may leak SurrealDB-internal field/index/query strings to clients (`AppError::InvalidRequest(dberr.to_string())`). | `backend/src/error.rs:65–77` |
| S4 | Medium | `Authorization` header is accepted **both** as a raw session id and as `Bearer <id>` — ambiguous and non-standard. | `backend/src/auth/bearer.rs` (via `authorization_bearer`) |
| S5 | Medium | DELETE semantics are inconsistent: most resources return **200 + deleted body**, `DELETE /teams/{id}/invitations/{id}` returns **204 No Content**. | see §5.1 |
| S6 | Medium | `POST /auth/otp/request` returns `204` and `POST /auth/otp/verify` can therefore be used to probe whether an email exists (auto-creates on verify instead of request). Email enumeration surface. | `backend/src/auth/otp/rest.rs` |
| S7 | Low | PUT on resources is **upsert** (creates with caller's personal team as owner when the id is new). Never documented in OpenAPI, tested but surprising for clients. | `SongService::update_song_for_user`, tests `blc_song_018_*` |
| S8 | Low | `body = Vec<u8>, content_type = "application/octet-stream"` for `/songs|setlists|collections/{id}/export` is wrong for `pdf`/`zip`/`wp`/`cp`. | `*/rest.rs` export routes |
| S9 | Low | Likes resource is a URL smell: `PUT /songs/{id}/likes` on a singleton with a `{ liked: bool }` body. Should be `PUT`/`DELETE /songs/{id}/like` or a proper collection. | `song/rest.rs:310–352` |
| S10 | Low | `page_size=0` meaning "no cap" is non-standard and undiscoverable; `page` is 0-based but also undocumented outside of BLCs. | `shared/src/api/list_query.rs`, `BLC-LP-006` |

The rest of this document discusses what is good, then drills into each class
of issue with concrete references and recommended fixes.

---

## 2. API surface at a glance

```
/                                          (SPA + static assets)
/api/docs/{*}                              Swagger UI (BLC-DOCS-001; public)
/api/docs/openapi.json                     OpenAPI JSON (public)
/auth/login                 GET            OIDC start (redirects)
/auth/callback              GET            OIDC callback
/auth/otp/request           POST           Send one-time code
/auth/otp/verify            POST           Verify code → session
/auth/logout                POST           Clear session
/api/v1                                    (wrapped by RequireUser)
  /users
    GET "/me"                              Current user
    GET|DELETE "/me/sessions{/id?}"        Self-sessions
    (wrapped by RequireAdmin)
    GET|POST|DELETE "" and "/{id}"         Admin user CRUD
    GET|POST|DELETE "/{user_id}/sessions{/id?}"
  /songs                                   CRUD + /player, /export, /likes
  /collections                             CRUD + /player, /export, /songs
  /setlists                                CRUD + /player, /export, /songs
  /blobs                                   CRUD + /{id}/data (binary download)
  /teams                                   CRUD (+ /{team_id}/invitations)
  /invitations/{id}/accept  POST           Accept invite
```

Counts: **~55** documented operations across **7** resource tags.

---

## 3. What is good (best-practice)

### 3.1 Overall architecture

- **Clean layering.** Each resource has `rest.rs` → `service.rs` → `repository.rs`
  → `surreal_repo.rs`. Handlers are thin (extract, build `UserPermissions`,
  delegate, respond). This is one of the most pleasant layouts I've reviewed.
- **Traits on the seam.** `SongRepository`, `TeamResolver`,
  `BlobStorage`, `LikedSongIds`, `UserCollectionUpdater`, etc. make services
  unit-testable without Surreal; the `MockSessionRepo` / `MockUserRepo` in
  `session/service.rs` is the fruit of that.
- **Per-request permission caching.** `UserPermissions` wraps `&User` +
  `&TeamResolver` and lazily resolves `read_teams`/`write_teams`/
  `personal_team` via `OnceCell`, so multi-call handlers pay for ACL
  resolution at most once. This is a good pattern and worth keeping.
- **Middleware is composable.** `RequireUser` sits on `/api/v1` and
  `RequireAdmin` is scoped further on admin routes via a nested
  `web::scope("")`. Clear, no ambiguity.
- **Resource IDs are validated.** `resource_id(table, id)` both accepts plain
  ids and the Surreal `table:id` form while rejecting wrong-table prefixes
  (`resources/common.rs`). Good BLC-HTTP-001 enforcement.

### 3.2 HTTP semantics that are correct

- **Status codes** are mostly right:
  - `201 Created` for `POST` on collection resources.
  - `204 No Content` for `POST /auth/logout` and `POST /auth/otp/request`.
  - `302 Found` with `Location` for `GET /auth/login` and `/auth/callback`.
  - `401` before `403`/`404` for missing/invalid session (BLC-AUTH-002).
  - `409 Conflict` for dedicated business conflicts (sole-admin removal,
    duplicate email).
- **PATCH semantics are explicit.** `PatchSong`/`PatchSongData` use
  `#[serde(deny_unknown_fields)]` and a custom `Patch<T>` tri-state
  (`Missing`/`Null`/`Value`). This is better than typical `Option<Option<T>>`
  hacks. Covered by exhaustive tests (`patch_song_data_all_field_combinations`
  with 1024 masks).
- **SPA fallback** is correctly scoped so that unmatched frontend routes serve
  `index.html` without swallowing `/api/v1/*` 404s.
- **Session cookie** is `HttpOnly`, `SameSite=Lax`, `Secure` via config,
  `Path=/`, and emptied with `Max-Age=0` on logout. Correct defaults.
- **OIDC login** uses **PKCE + nonce + state**, stores them server-side with
  a TTL, cleans them up before read/write, and uses `sanitize_redirect` to
  prevent open-redirect via `redirect_to`. Solid.
- **User agnosticism.** `User::new` lowercases email; OTP trims/lowercases
  before use. Consistent identity normalization.

### 3.3 OpenAPI / Documentation

- **Every route is annotated** with `#[utoipa::path(...)]`, including path,
  query parameters, request body schema, response-per-status and the security
  requirement (`SessionCookie` / `SessionToken`).
- **Tags** group resources neatly (`Auth`, `Users`, `Songs`, `Collections`,
  `Blobs`, `Setlists`, `Teams`).
- **Security schemes are declared** via `Modify` impl (`SessionSecurity`).
- **Publicly available spec** at `/api/docs/openapi.json` (BLC-DOCS-001).
- The `docs/business-logic-constraints/*.md` files form a rigorous,
  referenceable contract with BLC-IDs; the API implementation generally
  matches them.

### 3.4 Error handling

- `AppError` is a tidy, exhaustive enum with a single `ResponseError` impl
  that renders `{"error": "..."}` — matching the documented `ErrorResponse`
  schema.
- `500`s are logged via `tracing::error!`; `4xx`s are not log-spammed. Good.
- `From<surrealdb::Error>` for `AppError` correctly maps well-known Surreal
  error variants to `409`/`400`/`404` rather than always `500`.

### 3.5 Shared DTO types

- The `shared` crate publishes canonical DTOs used by both backend and
  frontend/CLI clients; backend annotates them with `utoipa::ToSchema` under
  a feature flag. This keeps the wire format single-sourced.
- `ListQuery::to_offset_limit()` exposes the pagination math symmetrically to
  frontend and tests.

---

## 4. Inconsistencies

These are places where the same concept is expressed differently across the
API. Each one is individually small, but collectively they make the surface
harder to consume and document.

### 4.1 Response status + body on `DELETE`

| Endpoint | Status | Body |
|---|---|---|
| `DELETE /api/v1/songs/{id}` | `200` | deleted `Song` |
| `DELETE /api/v1/collections/{id}` | `200` | deleted `Collection` |
| `DELETE /api/v1/setlists/{id}` | `200` | deleted `Setlist` |
| `DELETE /api/v1/blobs/{id}` | `200` | deleted `Blob` |
| `DELETE /api/v1/teams/{id}` | `200` | deleted `Team` |
| `DELETE /api/v1/users/{id}` | `200` | deleted `User` |
| `DELETE /api/v1/users/me/sessions/{id}` | `200` | deleted `Session` |
| `DELETE /api/v1/users/{user_id}/sessions/{id}` | `200` | deleted `Session` |
| `DELETE /api/v1/teams/{team_id}/invitations/{inv_id}` | **`204`** | — |

**Pick one.** Either everything returns `204 No Content`, or everything
returns `200` with the entity. Mixing both forces every client to branch per
resource. Recommended: `204` everywhere (the "deleted" body is almost never
consumed, and `200`+body muddles the semantics of "gone").

### 4.2 `POST /auth/logout` vs `DELETE` session resources

Logout is modelled as **`POST /auth/logout`** (correctly deletes the session
by reading it from cookie or bearer), while session deletion as a user-owned
resource is **`DELETE /api/v1/users/me/sessions/{id}`**. Not wrong, but note
they both ultimately hit `SessionService::delete_session`. At minimum, the
two code paths should share a helper and agree on cookie handling
(`/me/sessions/{id}` does **not** clear the cookie if you delete your own
current session — so the browser keeps a zombie cookie).

### 4.3 `PATCH` documented status codes

Several `PATCH` routes declare **400** but not **422**; several declare
**404**. A few are missing **403** on resources that can be writable to
non-admins but readable to admins (e.g. teams).

Notable specific gaps:
- `PATCH /api/v1/setlists/{id}` declares `400/404/500` but **not `403`**
  — though `update_setlist_for_user` can reject content-writability.
- `PUT /api/v1/setlists/{id}` does **not** declare `404` even though `PUT`
  elsewhere can produce it.

Harmonize the response matrix per method across resources (see §11).

### 4.4 Query parameter naming

- `/songs`, `/collections`, `/setlists` accept `q` for full-text search.
- `/users` and `/blobs` do **not** accept `q`.
- `/teams` has no listing filter at all.

Either document the "no-`q`" lists explicitly (current `BLC-LP-003` does, but
only BLC-readers will see it), or add `q` everywhere (e.g. search by user
email, blob OCR text — which is already stored — or team name).

### 4.5 Sub-resource vs. verb vs. representation

The project uses three different styles for "give me a different view of X":

- `GET /songs/{id}/player` — a shape transform (returns a `Player`).
- `GET /songs/{id}/export?format=pdf` — action with a format param.
- `GET /blobs/{id}/data` — a binary view of the same entity.

A cleaner convention would be:
- `?view=player` or **content negotiation** (`Accept: application/vnd.worship.player+json`)
  for representation variations.
- Reserve sub-resources for things that really are sub-resources
  (`/songs/{id}/likes` arguably is one, with a twist — see §5.4).

At minimum, pick the same style for **every** export: either `GET
/<resource>/{id}?format=pdf` or `GET /<resource>/{id}/export?format=pdf`
(you already do this — good; but the `player` route doesn't follow the
pattern it'd be naturally grouped with).

### 4.6 `ListQuery` ergonomics

- `page` is **0-based** (BLC-LP-001). The world is split on this — fine, but
  you should say this in **every** list operation's OpenAPI
  `param.description`. Currently only some do ("zero-based"), others just say
  "page index".
- `page_size=0` means **no cap** (BLC-LP-006). This is unusual; conventionally
  `page_size=0` either returns an empty array or is rejected. Clients that
  default `page_size` to `0` (e.g. "not set") will accidentally ask for the
  full table — a DoS footgun. Recommend:
  - treat absent as "server default",
  - reject `page_size=0` with `400`,
  - cap `page_size` at some large-but-finite server maximum (e.g. 500).
- There is **no pagination envelope**: no `total`, no `next`/`prev` links, no
  `X-Total-Count` header. Clients can only guess "end of list" from an
  undersized or empty page. This is awkward for any non-trivial UI.

### 4.7 Resource identifiers

- `resource_id(table, id)` accepts **both** `"abc"` and `"setlist:abc"` in a
  URL path. That flexibility is handy for internal tooling but problematic
  externally: it means two different URLs map to the same resource, which
  interferes with caching, logging, and client-side URL hygiene. Pick one
  external form (plain id) and enforce it; reject `table:id` at the edge with
  `400`.

### 4.8 Path-parameter semantics are not enforced

- `GET /api/v1/users/{user_id}/sessions/{id}` is documented as user-scoped
  but the handler calls `svc.get_session(&path.id)` — `user_id` is silently
  ignored (§S2).
- Same story for the admin-scoped `DELETE`. This means the URL lies: an
  admin can pass `user_id=A, id=<B's session>` and get/delete a session
  belonging to user B through a URL that says A. For logs and audit trails
  this is misleading.

### 4.9 Pagination support for nested collections

`/songs`, `/collections`, `/setlists`, `/blobs`, `/users` support pagination;
`/teams`, `/setlists/{id}/songs`, `/collections/{id}/songs`,
`/teams/{id}/invitations`, and `/users/me/sessions` do **not**. A team could
theoretically have hundreds of invitations or a collection thousands of
songs; those endpoints will break silently at some size.

### 4.10 Auth route versioning

`/auth/*` lives **outside** `/api/v1/*`, with no version prefix. Versioning
`/api/v1` but not `/auth` means a v2 roll-out cannot change login payload
shapes without a separate scheme. Consider moving to `/api/v1/auth/*`
(possible without breaking current clients via a transitional alias).

---

## 5. Smells (design / correctness)

### 5.1 `DELETE` returns the deleted entity (see also §4.1)

Even after you pick one style, returning the deleted entity on `DELETE` is
atypical enough to warrant `204` with no body, or to be renamed as `POST
/api/v1/.../{id}:delete` if you really want a body back. BLC-HTTP-002 says
a second DELETE returns 404 — that is orthogonal and good.

### 5.2 `DELETE /api/v1/users/me/sessions/{id}` is an IDOR (S1)

```82:88:backend/src/resources/user/session/rest.rs
#[delete("/me/sessions/{id}")]
pub async fn delete_session_for_current_user(
    svc: Data<SessionServiceHandle>,
    path: Path<SessionPath>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.delete_session(&path.id).await?))
}
```

There is no `ReqData<User>` parameter and no ownership check. Any
authenticated user can call `DELETE /api/v1/users/me/sessions/<any-session-id>`
and terminate another user's session — the only thing stopping total
account take-over is that session ids are unguessable. **This should be
`svc.delete_session_for_user(&path.id, &user.id)`** (mirror
`get_session_for_user`, which is correct), and return `404` if the session
does not belong to the caller.

Compare with the correct sibling directly above it:

```55:62:backend/src/resources/user/session/rest.rs
pub async fn get_session_for_current_user(
    svc: Data<SessionServiceHandle>,
    user: ReqData<User>,
    path: Path<SessionPath>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_session_for_user(&path.id, &user.id).await?))
}
```

That is doing the right thing. The delete sibling is not.

### 5.3 Admin session routes ignore `user_id` (S2)

```167:173:backend/src/resources/user/session/rest.rs
#[get("/{user_id}/sessions/{id}")]
pub async fn get_session_for_user(
    svc: Data<SessionServiceHandle>,
    path: Path<UserSessionPath>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(svc.get_session(&path.id).await?))
}
```

`UserSessionPath.user_id` is extracted and then `#[allow(dead_code)]`-ed. The
`DELETE` sibling is the same. Either honor `user_id` (check that the returned
session's `user.id` matches, else `404`), or drop the `user_id` segment and
use `/api/v1/sessions/{id}` for admins — but don't pretend to scope.

### 5.4 `/songs/{id}/likes` is not a collection

```310:352:backend/src/resources/song/rest.rs
#[get("/{id}/likes")]    // body: LikeStatus
#[put("/{id}/likes")]    // body: LikeStatus { liked: bool }
```

Semantically this is a **flag** ("does the caller like this song?"), not a
collection. Two cleaner options:

- **Toggle via PUT/DELETE on a singleton:**
  - `PUT  /api/v1/songs/{id}/like` → 204 (liked)
  - `DELETE /api/v1/songs/{id}/like` → 204 (unliked)
  - `GET  /api/v1/songs/{id}/like` → 200 or 404
- **Promote `likes` to a real collection:**
  - `GET  /api/v1/songs/{id}/likes` → who liked it (admin/self)
  - `PUT  /api/v1/songs/{id}/likes/me` / `DELETE .../likes/me`

The current shape is neither — it's `/likes` (plural, which suggests a
collection) with a body `{ liked: bool }` (which suggests a singleton). Pick
one.

### 5.5 Export endpoint OpenAPI content-type

```132:139:backend/src/resources/song/rest.rs
(
    status = 200,
    description = "Download exported song file",
    body = Vec<u8>,
    content_type = "application/octet-stream"
)
```

Same block in `collection/rest.rs` and `setlist/rest.rs`. In reality
`format=pdf` returns `application/pdf`, `format=zip` `application/zip`, and
`wp`/`cp` are text. `application/octet-stream` is misleading in the spec and
forces clients to ignore the declared content-type and sniff their own. Use
`utoipa` content-per-format or a union of `responses((status=200, ...,
content_type = ...))`. Also document the `Content-Disposition: attachment;
filename="..."` header if the server sets it (and if it doesn't, it should,
for any `format != wp|cp`).

### 5.6 `PUT` is upsert, secretly (S7)

`update_song_for_user` will **create** a song when the id doesn't exist,
attributing it to the caller's personal team (see
`blc_song_018_put_new_id_as_guest_creates_on_own_team`). For most resources
the OpenAPI says `(status = 200, description = "Update an existing X")` and
omits `201`. Clients cannot rely on a 200/201 distinction to know whether
they created vs updated. Recommended:

- Add `(status = 201, description = "Created via upsert", body = X)` to every
  `PUT`, and return 201 when the service created instead of updated.
- Alternatively, make `PUT` update-only (`404` if the id is unknown) and
  require `POST` for creation. "PUT-creates" is valid HTTP but a
  least-surprise violation when the collection endpoint already does
  something richer on create (e.g. appending to default collection).

### 5.7 Auto-creation side-effects of `POST /songs`

`create_song_for_user` silently creates a "Default" collection and sets
`user.default_collection` when absent (`BLC-SONG-010`). This is convenient
but:

- It's not reflected in `POST /songs` OpenAPI responses (no hint that a
  collection might be created, no `201` for the collection, no header
  pointing at it).
- A concurrent `POST /songs` by a fresh user could, in principle, create two
  "Default" collections (race condition between `create_collection` and the
  later `set_default_collection`). Consider an idempotency key or a DB-level
  unique constraint on `(owner, title='Default', is_system=true)` keyed on
  the user.

### 5.8 `Authorization` header semantics (S4)

From `SessionSecurity` in `docs.rs`:

> `Optional session override using "Authorization" header (raw value or "Bearer <session>")`

Accepting raw session ids as `Authorization:` values is non-standard. It also
means a malicious Referer/log redactor won't strip them (the usual list is
`Authorization: Bearer ...`). Require `Bearer ` prefix and reject otherwise.

### 5.9 OTP & email enumeration (S6)

```35:59:backend/src/auth/otp/rest.rs
#[post("/otp/request")]
// 400 if email missing; always 204 on success (even if no user yet)
```

`otp_request` creates an OTP regardless of whether the email belongs to a
user — that is fine for "login-via-OTP" and also avoids enumeration. But:

- `otp_verify` calls `user_svc.get_user_by_email_or_create(&email)` — it will
  **create** the user on first correct code. That's a silent self-signup
  flow via OTP, which may or may not be intentional but is not documented.
  If self-signup is not desired, an attacker can register arbitrary emails
  they control by going through `/otp/request` + `/otp/verify`.
- The OTP pepper is read from env and never rotated. Consider a key-rotation
  story.
- The OTP is a 6-digit numeric code. With the current TTL (5 min) and no
  attempt-limit visible in `rest.rs`, brute-force of `1_000_000` is in reach
  in 5 minutes with a good pipe. Verify that `db.validate_otp` enforces a
  per-email attempt counter + lockout — if not, **this is a security gap**.

### 5.10 Error messages leak internal detail (S3)

```65:77:backend/src/error.rs
surrealdb::error::Db::FieldCheck { .. }
| surrealdb::error::Db::FieldValue { .. }
| surrealdb::error::Db::InvalidField { .. }
| surrealdb::error::Db::InvalidArguments { .. }
| surrealdb::error::Db::InvalidParam { .. }
| surrealdb::error::Db::InvalidPatch { .. }
| surrealdb::error::Db::InvalidQuery { .. }
| surrealdb::error::Db::IdInvalid { .. }
| surrealdb::error::Db::InvalidUrl { .. }
| surrealdb::error::Db::SetCheck { .. }
| surrealdb::error::Db::TableCheck { .. } => {
    AppError::invalid_request(dberr.to_string())
}
```

The 400 body is `{"error": "<raw SurrealDB error>"}`. Depending on the
message that can leak table/field/index names and query fragments. For
authenticated callers this is mostly informational, but for public-ish
routes (`/auth/*`) it is a minor information-disclosure smell. Consider:

- translating to a **sanitized** 400 message plus a stable error code
  (`"bad_request"`, `"invalid_field"`), and
- putting the raw text into the server log (not the response body).

Same for `reqwest::Error` in the OIDC flow — the upstream error body may
contain provider-specific detail and is echoed as a 500 string.

### 5.11 Error envelope is ad-hoc (not Problem Details)

`ErrorResponse { error: String }` is fine but limits clients. A canonical
form would be RFC 7807 `application/problem+json`:

```json
{ "type": "about:blank",
  "title": "Bad request",
  "status": 400,
  "code": "invalid_song_id",
  "detail": "expected song:<id>",
  "instance": "/api/v1/songs/set%3Afoo",
  "request_id": "01HZX..." }
```

Even without going full 7807, adding:
- a stable, machine-readable `code` ("unauthorized", "not_found",
  "song_id_invalid", "sole_admin_cannot_remove_admins", …) and
- a `request_id` (use `tracing`'s span id or a UUID set by a middleware)

would make client-side error handling far nicer. Today clients must pattern-
match the human English string.

### 5.12 Per-response `utoipa::path(...)` boilerplate

Every handler repeats:

```rust
security(
    ("SessionCookie" = []),
    ("SessionToken" = [])
),
responses(
    (status = 401, description = "Authentication required", body = ErrorResponse),
    (status = 500, description = "...", body = ErrorResponse),
    ...
)
```

This is error-prone (see §4.3, where the matrix is inconsistent across
endpoints), and a large part of why minor inconsistencies exist. Consider
one of:

- A macro (`#[authenticated_path(...)]`) that injects 401/403/500 and the
  security block.
- A custom `utoipa::Modify` that appends common responses to all routes
  with a `Security` requirement.

### 5.13 Handlers duplicate `UserPermissions::new(&user, &svc.teams)`

Every handler does:

```rust
let perms = UserPermissions::new(&user, &svc.teams);
Ok(HttpResponse::Ok().json(svc.xxx_for_user(&perms, ...).await?))
```

A thin `FromRequest` extractor (`Perms<T>` tuple of `(User, UserPermissions)`)
would remove 50+ lines of boilerplate and is a cheaper change than it looks.

### 5.14 No uniform input validation layer

- `CreateSong { blobs: Vec<String> }` — no limit, no id-format check until
  Surreal complains.
- `CreateTeam { name: String }` — no length, no whitespace rules.
- `UpdateTeam.members` — no max cardinality; a malicious client could
  attempt very large member lists.

A `validator`-derived pass (or hand-written `TryFrom<CreateX> for ValidX`)
on all inbound DTOs would catch these at 400 with precise messages rather
than surfacing as DB errors.

### 5.15 Missing caching headers

None of the `GET` endpoints emit `ETag`, `Last-Modified`, or `Cache-Control`.
For `GET /api/v1/songs/{id}` specifically, a strong ETag and `If-None-Match`
support would save real bandwidth because song data is chunky. For
`GET /api/v1/blobs/{id}/data` (binary), `ETag` + `Cache-Control:
private, max-age=..., immutable` is trivial (blobs are content-addressed)
and a huge win over the wire.

### 5.16 No rate-limiting / 429s

No routes document or enforce rate limits. At minimum the login surface
(`/auth/otp/request`, `/auth/otp/verify`, `/auth/login`) should be rate-
limited per IP and per email. Document `429` in the OpenAPI responses for
those three, and wire up a middleware (e.g. `actix-governor`).

### 5.17 No request body size limit

Actix has a default but it is not set explicitly. For `/songs` (arbitrarily
deep `SongData` via `value_type = Object, additional_properties = true`) and
`/blobs` there is no declared ceiling in the spec. Declare and enforce one
(e.g. 1 MB for JSON, explicit route for binary blob upload — see §5.18).

### 5.18 How do blobs actually get content?

`POST /api/v1/blobs` takes `CreateBlob` with `file_type`, `width`, `height`,
`ocr` — but **no binary payload**. Yet `GET /api/v1/blobs/{id}/data` serves
the binary via `NamedFile`. There is no documented endpoint to **upload**
blob content. Either:

- the upload is happening out-of-band (migrations? CLI? Seed script?), in
  which case this should be documented, or
- there is an undocumented (or missing) `PUT /api/v1/blobs/{id}/data`.

This is the biggest "missing verb" in the API today and should be closed.

### 5.19 `NamedFile` exposes `Content-Disposition: inline`

For `GET /api/v1/blobs/{id}/data` and the export routes, the default
`NamedFile`/`HttpResponse` will serve `inline` unless explicitly set.
Consider `Content-Disposition: attachment; filename="<Blob::file_name>"`
(the helper already exists on `Blob`), especially for exports where it's
expected.

### 5.20 DELETE idempotency wording

BLC-HTTP-002 says the second `DELETE` returns `404`. That *is* idempotent
in effect (the resource remains absent), but many HTTP consumers expect
`204` + `204` for both calls. Either rephrase the BLC to
"subsequent DELETE on the same id returns 404, which clients MUST treat as
success" — or (better) return `204` on the second call as well. The cost of
doing so is small and it aligns with RFC 9110 §9.3.5.

### 5.21 Deserialization is not strict everywhere

`CreateSong`, `CreateCollection`, `CreateBlob`, `CreateSetlist`,
`CreateUserRequest`, `CreateTeam`, `UpdateTeam` do **not** have
`#[serde(deny_unknown_fields)]` (only `Patch*` types do, plus `TeamMemberInput`,
`TeamUserRef`). Unknown fields are silently dropped, which makes typos go
unnoticed. Add it to all request DTOs for consistency (the `Patch*` types
already set the precedent).

### 5.22 SPA fallback is a `404` leak

The fallback returns `index.html` for any unknown GET, including `/api/v2/*`,
`/api/v1/nonexistent/resource`, etc. Actix normally routes `/api/v1/*` to the
scope (where 404s are produced), but if a path segment escapes the scope
(e.g. `/api/`, `/api/v1`, `/api/v1/`, depending on routing order), the SPA
swallows it. Make the fallback refuse to serve on any path starting with
`/api/` or `/auth/` and return a plain JSON `404` there.

### 5.23 `TeamService::delete_team_for_user` takes `&perms`, others take `&user`

Compare `delete_team` (uses `UserPermissions`) with `update_team`,
`patch_team`, `create_team`, `get_team` (all use bare `&user`):

```127:172:backend/src/resources/team/rest.rs
async fn update_team(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    ...
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(
        svc.update_team_for_user(&user, &id, payload.into_inner())
            .await?,
    ))
}
```

vs.:

```194:202:backend/src/resources/team/rest.rs
async fn delete_team(
    svc: Data<TeamServiceHandle>,
    user: ReqData<User>,
    id: Path<String>,
) -> Result<HttpResponse, AppError> {
    let perms = UserPermissions::new(&user, &svc.resolver);
    Ok(HttpResponse::Ok().json(svc.delete_team_for_user(&perms, &id).await?))
}
```

The team service inconsistently mixes `&User` and `UserPermissions`. Pick
one (`UserPermissions` is the pattern elsewhere).

### 5.24 `default_collection` is never revalidated

If a user's `default_collection` points at a collection that has been
deleted, `create_song_for_user` will call
`collections.add_song_to_collection(...)` and the DB call will fail with a
generic error. Guard with a "does-this-collection-exist-and-is-writable?"
check, or clear the stale reference when creating the next default.

---

## 6. Security review summary

Grouped and ranked:

1. **[High] IDOR in `DELETE /api/v1/users/me/sessions/{id}`** (§5.2). Fix: pass
   the current user and scope the delete, or 404 on mismatch.
2. **[Medium] Path-lying admin session routes** (§5.3). Fix: enforce
   `path.user_id` in the query or drop it.
3. **[Medium] Error-string leakage** (§5.10). Fix: sanitize 400 messages,
   keep raw text in logs only.
4. **[Medium] Bearer header ambiguity** (§5.8). Fix: require `Bearer ` prefix.
5. **[Medium] OTP brute-force / signup-via-OTP** (§5.9). Fix: attempt counter
   + lockout; document (or disable) auto-signup.
6. **[Medium] Missing rate-limit** (§5.16). Fix: add middleware for
   `/auth/otp/*` and `/auth/login`.
7. **[Low] CSRF on cookie-authenticated mutations**. `SameSite=Lax` blocks
   cross-site form POSTs but **not** cross-site `GET` side-effects. The
   `/auth/logout` is a `POST` (good) and all mutations are `POST/PUT/PATCH/
   DELETE`. No CSRF hardening beyond `SameSite`. Document this posture and
   consider a custom header (`X-Requested-With`) that must be present on
   cookie-authenticated writes.
8. **[Low] Open redirect surface.** `sanitize_redirect` accepts any path that
   starts with `/` and is not `//` or `/http`. `/\\evil` and various other
   browser-specific quirks may still coerce navigation. Consider an allow-
   list of paths rather than a deny-list. Also pass the candidate through a
   URL parser and verify no scheme/host is present.
9. **[Low] No `Set-Cookie` on logout from bearer path.** `POST /auth/logout`
   only sets the empty cookie when a cookie was present in the request. If a
   browser logs out via `Authorization: Bearer ...` after the cookie expired,
   the zombie cookie remains. Safer: always send the clearing cookie.

---

## 7. Error-handling review

**Good:**

- `AppError` is exhaustive and self-documenting; `status_code()` is
  single-sourced.
- 500s log; 4xx do not.
- SurrealDB errors are mapped to specific 4xx where possible.

**Improvements:**

- Add a `Code` (stable string enum) alongside the human message so clients
  can branch deterministically (§5.11).
- Include `request_id` in the response; set `X-Request-Id` from a middleware.
- Sanitize 400 bodies (§5.10).
- `Internal` variant currently carries the raw string. Log it, never return
  it to the client. Currently `error_response()` does
  `self.to_string()` which includes the internal detail (`"internal error:
  <leakage>"`). Strip the tail for Internal before JSON-serialising.
- The `LoginQuery` uses `sanitize_redirect(...)` silently — if the input is
  rejected it silently falls back. Prefer to 400 obviously-bad values (so
  that callers notice) while still preserving a safe fallback for nulls.

---

## 8. Pagination & list semantics review

**Good:**

- `ListQuery` is simple, typed, and shared with the frontend/CLI.
- Behavior rules are specified as BLCs.

**Issues:**

- Unusual `page_size=0` semantics (§4.6).
- No total / link envelope.
- Trim-to-empty `q` semantics (BLC-LP-005) are not mentioned in the
  OpenAPI description of `q`.
- Several sub-collection GETs are unbounded (§4.9).
- Sorting is nowhere in the spec; the client cannot ask for a specific sort
  order. Even documenting the default (creation order? alphabetic?) would
  help.
- No filtering beyond `q`. Consider a small structured filter grammar
  (`owner=`, `tag=`, `lang=`) for `/songs` in particular; the data model
  supports it (`SongData.tags`, `languages`).

---

## 9. OpenAPI / Documentation review

**Good:**

- Every route is annotated. `utoipa::IntoParams` on query structs.
- Security schemes declared; tags applied.
- Shared schemas declared in one list (`docs.rs`) — easy audit.

**Issues:**

- Inconsistent response matrices per method (§4.3).
- `body = Vec<u8>, content_type = "application/octet-stream"` for exports is
  inaccurate (§5.5).
- Some descriptions are very terse ("Invalid request" — invalid how?).
  Prefer short, example-flavored descriptions.
- `PatchSongData` uses a custom tri-state with `value_type = Option<String>`
  workaround; fine, but consider emitting an OpenAPI `oneOf` to expose
  "absent vs null vs value" to code-generators properly.
- Versioning: the OpenAPI document has no `info.version` bump strategy
  attached to the `/api/v1` prefix. Decide whether the `v1` lives in the
  URL or the `info.version` (or both, with rules about when which changes).
- `info.license`, `info.contact`, `info.termsOfService` are not set.
- No examples on any endpoint. Even one example per resource (request body
  + response body) would make the Swagger UI self-teaching.
- No `servers:` entry in the spec.

---

## 10. Miscellaneous

- `Logger::default()` logs the session cookie as part of headers unless
  configured otherwise. Double-check the default format string; if headers
  are logged verbatim, you need a custom Logger format that redacts
  `Cookie:` and `Authorization:`.
- `PrinterConfig` is a server-side config passed into handlers as `Data`.
  That is fine, but the `Data<PrinterConfig>` parameter ordering in handler
  signatures varies per file; harmless, yet one more thing to clean up.
- `actix_files::Files::new("/", static_dir)` — if `static_dir` is a
  relative path the behavior depends on CWD. Prefer an absolute canonical
  path resolved at startup (log it).
- `initial_admin_user_test_session` creates a non-expiring admin session if
  enabled — good that the log says "DO NOT USE THIS IN PRODUCTION", but it
  would be better to `panic!` (or at least fail loudly) if the flag is set
  while the settings indicate a production profile.

---

## 11. Canonical response matrix (proposed)

For every `/api/v1` route (authenticated):

| Outcome | GET list | GET one | POST | PUT | PATCH | DELETE |
|---|---|---|---|---|---|---|
| Success | 200 + `[T]` | 200 + `T` | 201 + `T` + `Location` | 200 + `T` (or 201 if upsert created) | 200 + `T` | 204 |
| Missing auth | 401 | 401 | 401 | 401 | 401 | 401 |
| Insufficient role | 403 | 403 | 403 | 403 | 403 | 403 |
| Resource absent | — | 404 | — | 404 (if update-only) | 404 | 404 |
| Bad id | 400 | 400 | — | 400 | 400 | 400 |
| Bad body | — | — | 400 | 400 | 400 | — |
| Business conflict | — | — | 409 | 409 | 409 | 409 |
| Internal | 500 | 500 | 500 | 500 | 500 | 500 |
| Rate-limited | 429 (auth) | 429 (auth) | 429 (auth) | 429 (auth) | 429 (auth) | 429 (auth) |

Use this matrix as the single source of truth for `utoipa::responses(...)`
blocks, ideally driven by a macro (§5.12).

---

## 12. Recommendations — prioritized

### P0 (do first)

1. Fix §S1 IDOR on `DELETE /api/v1/users/me/sessions/{id}` — add ownership
   check.
2. Honor (or drop) `user_id` on admin session routes (§S2).
3. Verify OTP attempt-rate limiting and enforce a lockout; add rate-limit
   middleware to `/auth/otp/*` and `/auth/login` (§S6, §5.16).
4. Add blob content upload endpoint — or document the out-of-band path
   (§5.18).

### P1 (near term)

5. Pick DELETE semantics (204 without body) and apply everywhere (§4.1,
   §5.1).
6. Add stable error codes + `request_id` + sanitized 400 bodies (§5.10,
   §5.11).
7. `deny_unknown_fields` on all request DTOs (§5.21).
8. Cap `page_size`, reject `page_size=0`, add response envelope or
   `X-Total-Count` header (§4.6).
9. Tighten resource id acceptance: plain id only at the edge (§4.7).
10. Require `Bearer ` prefix on `Authorization` (§5.8).
11. Correct export/download content types in OpenAPI (§5.5).

### P2 (quality)

12. Rename likes to either `PUT/DELETE /songs/{id}/like` or a proper
    collection (§5.4).
13. Add `ETag`/`If-None-Match` for GETs, `Cache-Control` + `immutable` for
    blob data (§5.15).
14. Add caching/pagination to nested collections (`/teams`, `/collections/
    {id}/songs`, invitations, `/users/me/sessions`) (§4.9).
15. Harmonize team service signatures (§5.23).
16. Reduce `utoipa` boilerplate via macro or Modify (§5.12).
17. Extract `Perms<T>` as a `FromRequest` extractor (§5.13).
18. Validate request bodies at the edge (§5.14).
19. Document PUT upsert behaviour (or remove it) (§5.6).
20. Document the auto-Default-collection side effect on `POST /songs`
    (§5.7).
21. Add examples to OpenAPI (at least one per resource) (§9).
22. Clean up `error.rs` tail-leak on `Internal` (§7).
23. Decide and document CSRF posture (§6.7).
24. Move auth under a version prefix or clearly document the split (§4.10).

### P3 (nice to have)

25. RFC 7807 Problem Details.
26. Sort + structured filters on `/songs`.
27. Content negotiation for player/export.
28. Explicit `servers:`, `info.version`, `info.license` in the OpenAPI doc.
29. Add `Content-Disposition: attachment` to downloads.
30. Wire OpenTelemetry span ids through `request_id`.

---

## 13. Closing note

The codebase is cleanly structured, and the BLC docs in
`docs/business-logic-constraints/` are a genuine asset — it is rare to see
this level of specification rigor on a greenfield REST backend. The issues
above are almost all at the edges: naming, response shape, auth-route
hygiene, and a handful of real correctness/security bugs that a single
careful pass can close. None of them require architectural change.

Fixing P0 + P1 alone would put this API in a very strong position for
external consumers, and the architecture you already have makes those fixes
cheap and mechanical.
