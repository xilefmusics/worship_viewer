# Test coverage improvement plan for business logic constraints

This document maps every business logic constraint (BLC) to its current test status, identifies testability gaps, and proposes concrete test cases covering edge cases and representative middle cases on both positive and negative sides.

## Table of contents

- [Current state summary](#current-state-summary)
- [Architectural improvements for testability](#architectural-improvements-for-testability)
- [Constraint coverage by resource](#constraint-coverage-by-resource)
  - [API documentation (BLC-DOCS)](#api-documentation-blc-docs)
  - [Authentication (BLC-AUTH)](#authentication-blc-auth)
  - [HTTP contract (BLC-HTTP)](#http-contract-blc-http)
  - [List pagination (BLC-LP)](#list-pagination-blc-lp)
  - [User (BLC-USER)](#user-blc-user)
  - [Session (BLC-SESS)](#session-blc-sess)
  - [Team (BLC-TEAM)](#team-blc-team)
  - [Team invitation (BLC-TINV)](#team-invitation-blc-tinv)
  - [Blob (BLC-BLOB)](#blob-blc-blob)
  - [Collection (BLC-COLL)](#collection-blc-coll)
  - [Setlist (BLC-SETL)](#setlist-blc-setl)
  - [Song (BLC-SONG)](#song-blc-song)
- [Implementation roadmap](#implementation-roadmap)

---

## Current state summary

| Resource | BLC count | Tested | Partially tested | Untested | Test style |
|----------|-----------|--------|------------------|----------|------------|
| API docs | 1 | 0 | 0 | 1 | — |
| Authentication | 2 | 0 | 0 | 2 | — |
| HTTP contract | 2 | 0 | 0 | 2 | — |
| List pagination | 9 | 2 | 3 | 4 | integration (setlist, song) |
| User | 14 | 0 | 1 | 13 | — |
| Session | 9 | 0 | 0 | 9 | — |
| Team | 18 | 3 | 2 | 13 | integration |
| Team invitation | 14 | 0 | 0 | 14 | — |
| Blob | 15 | 3 | 2 | 10 | mock + integration |
| Collection | 19 | 3 | 2 | 14 | integration |
| Setlist | 14 | 10 | 2 | 2 | mock + integration |
| Song | 18 | 5 | 3 | 10 | integration |
| **Total** | **135** | **~26** | **~15** | **~94** | |

### Naming convention for new tests

All new tests should use the pattern `blc_{resource}_{nnn}_{short_description}` and include a doc comment referencing the BLC ID, e.g.:

```rust
/// BLC-TINV-011: accepting when already content_maintainer must not downgrade role.
#[tokio::test]
async fn blc_tinv_011_accept_does_not_downgrade_content_maintainer() { ... }
```

---

## Architectural improvements for testability

### 1. Extract pure validation functions from service methods

Several constraints are enforced inline inside service methods that need a database. Extract them into standalone `pub(crate)` functions that can be tested with unit tests (no DB).

**Where this applies:**
- `team/model.rs` — functions like `member_self_leave_payload`, `validate_shared_has_admin`, `validate_personal_members_not_owner` already exist but have **zero** unit tests. These are pure functions and trivially testable.
- `team/invitation/model.rs` — `invitation_thing` and `team_things_match` are pure but untested.
- `user/model.rs` — `user_resource` is pure but untested.
- Setlist/song/collection — `from_payload` conversions have tests, but validation of required fields (e.g. `BLC-SETL-004` non-empty title) is enforced at the serde level or not at all; these need explicit test cases.

### 2. Add mock implementations for remaining repository traits

Currently `blob/service.rs` and `setlist/service.rs` use mock repos. The following services have **no** mock-based unit tests:

| Service | Repository trait | Mock exists? |
|---------|------------------|-------------|
| `InvitationService` | `TeamInvitationRepository` + `TeamRepository` | No |
| `UserService` | `UserRepository` + `TeamRepository` | No |
| `SessionService` | `SessionRepository` + `UserRepository` | No |
| `CollectionService` | `CollectionRepository` + `TeamResolver` | No |
| `SongService` | `SongRepository` + `TeamResolver` + `UserCollectionUpdater` | No |
| `TeamService` | `TeamRepository` | No |

For each, create a `MockXxxRepo` (or use `mockall`) implementing the trait to enable fast, deterministic unit tests that do not require SurrealDB.

### 3. HTTP-layer test harness

BLC-AUTH, BLC-DOCS, BLC-HTTP, and some BLC-USER/BLC-SESS constraints describe HTTP-level behavior (status codes, header handling). These are currently **completely untested**. Introduce an `actix_web::test` harness:

```rust
// backend/src/resources/rest_tests.rs (new file)
#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    // Build test app with real or mock services, call endpoints, assert status codes.
}
```

This enables testing:
- Missing/invalid `Authorization` header → 401
- Non-admin calling admin-only routes → 403
- Invalid path param format → 400
- OpenAPI doc endpoint (unauthenticated) → 200

### 4. Test-fixture builder for multi-role scenarios

Many BLC constraints require testing the same operation from different roles (owner, admin, content_maintainer, guest, non-member, platform admin). Create a builder:

```rust
struct TeamFixture {
    db: Arc<Database>,
    owner: User,           // personal team owner
    team_admin: User,      // shared team admin
    content_maintainer: User,
    guest: User,
    non_member: User,
    platform_admin: User,
    personal_team_id: String,
    shared_team_id: String,
}
```

This avoids repeating the same 30 lines of setup in every test and ensures all role permutations are covered.

---

## Constraint coverage by resource

### API documentation (BLC-DOCS)

> Source: [api-documentation.md](./business-logic-constraints/api-documentation.md)

#### BLC-DOCS-001 — OpenAPI endpoint unauthenticated

**Current coverage:** None.

**Testability:** HTTP-layer only (outside `/api/v1` scope).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /api/docs/openapi.json` without auth header → 200, body is valid JSON | positive middle | HTTP |
| 2 | `GET /api/docs/openapi.json` with an auth header present → still 200 (not gated) | positive edge | HTTP |
| 3 | `GET /api/v1/docs/openapi.json` (wrong path) → 404 | negative edge | HTTP |

---

### Authentication (BLC-AUTH)

> Source: [authentication.md](./business-logic-constraints/authentication.md)

#### BLC-AUTH-001 — Missing or malformed Authorization header → 401

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Request to `/api/v1/songs` with no `Authorization` header → 401 | negative middle | HTTP |
| 2 | Request with `Authorization: Basic abc` (not Bearer) → 401 | negative edge | HTTP |
| 3 | Request with empty `Authorization:` header → 401 | negative edge | HTTP |
| 4 | Request with valid `Authorization: Bearer <token>` → not 401 (proceeds to resource logic) | positive middle | HTTP |

#### BLC-AUTH-002 — Invalid/expired token → 401

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `Authorization: Bearer totallyinvalidtoken` → 401 | negative middle | HTTP |
| 2 | `Authorization: Bearer <deleted-session-id>` → 401 | negative edge | HTTP |
| 3 | Valid session token → passes auth middleware | positive middle | HTTP |

---

### HTTP contract (BLC-HTTP)

> Source: [http-contract.md](./business-logic-constraints/http-contract.md)

#### BLC-HTTP-001 — Syntactically invalid resource ID in path → 400

**Current coverage:** Partially covered by `common::resource_id` unit tests, but no HTTP-layer tests.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /api/v1/songs/song:validid` → 200 or 404 (valid format) | positive middle | HTTP |
| 2 | `GET /api/v1/songs/blob:wrongtable` → 400 | negative middle | HTTP |
| 3 | `GET /api/v1/songs/plainid` → 200 or 404 (valid format) | positive edge | HTTP |
| 4 | `DELETE /api/v1/setlists/collection:x` → 400 | negative edge | HTTP |

#### BLC-HTTP-002 — Idempotent DELETE → 404 on repeat

**Current coverage:** None at HTTP layer.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | DELETE resource, DELETE same ID again → 404 | negative middle | integration |
| 2 | DELETE non-existent ID → 404 | negative edge | integration |
| 3 | DELETE existing resource → success (200/204) | positive middle | integration |

---

### List pagination (BLC-LP)

> Source: [list-pagination.md](./business-logic-constraints/list-pagination.md)

#### BLC-LP-001 / BLC-LP-002 — `page` and `page_size` parameters

**Current coverage:** Partially covered in setlist integration tests (`blc_setl_list_and_pagination`).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `page=0&page_size=2` with 5 items → returns 2 items | positive middle | integration |
| 2 | `page=0` only (no `page_size`) → uses server default, returns 200 | positive edge (BLC-LP-007) | integration |
| 3 | `page_size=1` only (no `page`) → uses server default page, returns 200 | positive edge (BLC-LP-007) | integration |
| 4 | Neither param → returns all with defaults, 200 | positive edge | integration |

#### BLC-LP-003 — `q` parameter scope

**Current coverage:** Partially in setlist (`blc_setl_search`) and song (`blc_song_crud_search_likes`).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /songs?q=partial_title` → matches songs by title | positive middle | integration |
| 2 | `GET /songs?q=artist_name` → matches songs by artist | positive middle | integration |
| 3 | `GET /collections?q=coll_title` → matches collections by title | positive middle | integration |
| 4 | `GET /blobs?q=anything` → `q` is ignored or rejected (blobs don't support `q`) | negative edge | integration/HTTP |
| 5 | `GET /users?q=anything` → `q` is ignored (users don't support `q`) | negative edge | integration/HTTP |

#### BLC-LP-004 — Non-integer `page`/`page_size` → 400

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /songs?page=abc` → 400 | negative middle | HTTP |
| 2 | `GET /songs?page_size=1.5` → 400 | negative edge | HTTP |
| 3 | `GET /songs?page=-1` → either 400 or 200 with empty (document behavior) | negative edge | HTTP |
| 4 | `GET /songs?page=0&page_size=10` → 200 | positive middle | HTTP |

#### BLC-LP-005 — Whitespace-only `q` treated as absent

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /songs?q=%20%20` → same result as no `q` | positive middle (treated as no filter) | integration |
| 2 | `GET /songs?q=` (empty string) → same result as no `q` | positive edge | integration |
| 3 | `GET /songs?q=%09` (tab only) → same result as no `q` | negative edge | integration |

#### BLC-LP-006 — `page_size=0` returns all items

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /songs?page_size=0` with 10 songs → returns all 10 | positive middle | integration |
| 2 | `GET /songs?page_size=0&page=0` → returns all | positive edge | integration |
| 3 | `GET /songs?page_size=0&page=1` → returns empty (all items on page 0) | negative edge | integration |

#### BLC-LP-007 — Omitted param uses server default

**Current coverage:** Partial (setlist tests supply both params).

Covered above in BLC-LP-001/002 tests.

#### BLC-LP-008 — Page beyond last → 200 with empty array

**Current coverage:** None explicit.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /songs?page=999&page_size=10` with 3 songs → 200, empty array | positive middle | integration |
| 2 | `GET /songs?page=1&page_size=100` with 3 songs → 200, empty array | positive edge | integration |
| 3 | `GET /songs?page=0&page_size=10` with 0 songs → 200, empty array | negative edge | integration |

#### BLC-LP-009 — `q` filter applies before pagination

**Current coverage:** None explicit.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | 10 songs, 3 match `q`, `page_size=2&page=0` → returns 2 matching | positive middle | integration |
| 2 | 10 songs, 3 match `q`, `page_size=2&page=1` → returns 1 matching | positive edge | integration |
| 3 | 10 songs, 0 match `q`, `page=0` → returns empty | negative middle | integration |

---

### User (BLC-USER)

> Source: [user.md](./business-logic-constraints/user.md)

#### BLC-USER-001 — Email unique after normalization

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create user with `test@example.com`, create another with `TEST@EXAMPLE.COM` → 409 | negative middle | integration |
| 2 | Create user with `  test@example.com  ` (whitespace), then `test@example.com` → 409 | negative edge | integration |
| 3 | Create two users with distinct emails → both succeed | positive middle | integration |

#### BLC-USER-002 — Role is `default` or `admin`

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | New user has role `default` | positive middle | unit/integration |
| 2 | Admin-created user can have role `admin` | positive edge | integration |

#### BLC-USER-003 — User creation paired with personal team

**Current coverage:** Implicitly tested in `test_helpers::seed_user` + `personal_team_id`, but no explicit assertion.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create user → personal team exists with user as owner | positive middle | integration |
| 2 | Create user → personal team has no members | positive edge | integration |
| 3 | Delete personal team independently → should fail per BLC-TEAM-006 | negative edge | integration |

#### BLC-USER-004 — `default_collection` stored as-is (no existence check)

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create user with `default_collection` set to non-existent ID → 201 | positive middle | integration |
| 2 | Create user with `default_collection` set to valid ID → 201 | positive edge | integration |

#### BLC-USER-005 — `GET /users/me` returns current user

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Authenticated user calls `GET /users/me` → 200, body matches user record | positive middle | HTTP |
| 2 | Different users each get their own record | positive edge | HTTP |

#### BLC-USER-006 — `GET /users/me` accepts raw token without `Bearer` prefix

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `Authorization: <raw-token>` (no Bearer) on `GET /users/me` → 200 | positive middle | HTTP |
| 2 | `Authorization: <raw-token>` on other endpoints → 401 | negative edge | HTTP |

#### BLC-USER-007 — Non-admin on admin-only user routes → 403

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Non-admin `GET /users` → 403 | negative middle | HTTP |
| 2 | Non-admin `POST /users` → 403 | negative middle | HTTP |
| 3 | Non-admin `DELETE /users/{id}` → 403 | negative middle | HTTP |
| 4 | Non-admin `GET /users/{id}` → 403 | negative middle | HTTP |
| 5 | Admin `GET /users` → 200 | positive middle | HTTP |

#### BLC-USER-008 — Duplicate email → 409, invalid/missing → 400

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `POST /users` with existing email → 409 | negative middle | integration |
| 2 | `POST /users` with missing email → 400 | negative edge | HTTP |
| 3 | `POST /users` with empty email → 400 | negative edge | HTTP |
| 4 | `POST /users` with valid new email → 201 | positive middle | integration |

#### BLC-USER-009 — `GET /users/{id}` requires admin or self-read

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Admin calls `GET /users/{id}` → 200 | positive middle | HTTP |
| 2 | Non-admin guest on personal team calls `GET /users/{owner_id}` → 403 | negative edge | HTTP |
| 3 | Non-admin calls `GET /users/{other_id}` → 403 | negative middle | HTTP |

#### BLC-USER-010 — `/users/me/sessions` scoped to current user

**Current coverage:** None. See BLC-SESS-003/004.

#### BLC-USER-011 — Admin session management for other users

**Current coverage:** None. See BLC-SESS-005/006.

#### BLC-USER-012 — Delete user → sessions stop working → 401

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create user + session, delete user, use session token → 401 | negative middle | integration |
| 2 | Create user + 2 sessions, delete user, both tokens → 401 | negative edge | integration |

#### BLC-USER-013 — Delete user → personal team + all owned resources removed

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create user with songs/collections/blobs, delete user → all 404 | negative middle | integration |
| 2 | Guest on deleted user's personal team → resources 404 | negative edge | integration |

#### BLC-USER-014 — Repeated DELETE → 404

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Delete user, delete same ID again → 404 | negative middle | integration |
| 2 | Delete non-existent user ID → 404 | negative edge | integration |

---

### Session (BLC-SESS)

> Source: [session.md](./business-logic-constraints/session.md)

#### BLC-SESS-001 — Session bound to one user (static)

**Current coverage:** None (structural — tested implicitly if create returns correct `user_id`).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create session → session `user_id` matches creating user | positive middle | integration |

#### BLC-SESS-002 — Session token is opaque (static)

No direct test needed; tested indirectly by auth middleware.

#### BLC-SESS-003 — `GET /users/me/sessions` returns only own sessions

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User A has 2 sessions, User B has 1 → A sees 2, B sees 1 | positive middle | integration |
| 2 | User with 0 sessions → 200, empty list | positive edge | integration |

#### BLC-SESS-004 — `GET/DELETE /users/me/sessions/{id}` scoped to own sessions

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User A deletes own session → 200/204 | positive middle | integration |
| 2 | User A tries to get User B's session via `/me/sessions/{b_session}` → 404 | negative middle | integration |
| 3 | User A tries to delete User B's session → 404 | negative edge | integration |

#### BLC-SESS-005 — Non-admin on `/users/{user_id}/sessions` → 403

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Non-admin `POST /users/{other}/sessions` → 403 | negative middle | HTTP |
| 2 | Non-admin `GET /users/{other}/sessions` → 403 | negative middle | HTTP |
| 3 | Admin `GET /users/{other}/sessions` → 200 | positive middle | HTTP |

#### BLC-SESS-006 — Admin manages other user's sessions

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Admin creates session for user → 201 | positive middle | integration |
| 2 | Admin lists sessions for user → 200 | positive middle | integration |
| 3 | Admin deletes session for user → 204 | positive middle | integration |
| 4 | Admin uses invalid `user_id` → 404 | negative edge | integration |
| 5 | Admin deletes already-deleted session → 404 | negative edge | integration |

#### BLC-SESS-007 — Non-admin `POST /users/{other}/sessions` → 403

**Current coverage:** None. Covered by BLC-SESS-005 tests.

#### BLC-SESS-008 — User deleted → all sessions invalidated

**Current coverage:** None. See BLC-USER-012.

#### BLC-SESS-009 — Deleted session must not authenticate again

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Delete session, then `validate_session_and_update_metrics` → None | negative middle | integration |
| 2 | Delete session, use token on authenticated route → 401 | negative edge | HTTP |

---

### Team (BLC-TEAM)

> Source: [team.md](./business-logic-constraints/team.md)

#### BLC-TEAM-001 — Personal team 1:1 with user, owner not in members

**Current coverage:** Implicit in `personal_team_id` helper.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create user → personal team has `owner` set to that user | positive middle | integration |
| 2 | `validate_personal_members_not_owner` with owner in members → error | negative middle | unit |
| 3 | `validate_personal_members_not_owner` with owner absent → ok | positive middle | unit |

#### BLC-TEAM-002 — Shared team has no owner, creator is admin

**Current coverage:** `blc_team_shared_create_and_list` checks creator is admin.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create shared team → `owner` is None, creator in members as admin | positive middle | integration |
| 2 | `build_create_shared_members` with creator ID → creator is admin regardless of extra input | positive edge | unit |
| 3 | `build_create_shared_members` with creator also in `extra` as guest → still admin | positive edge | unit |

#### BLC-TEAM-003 — Personal team owner has admin-level control

**Current coverage:** Implicitly via `effective_admin`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `effective_admin(owner_id, stored_with_owner)` → true | positive middle | unit |
| 2 | `effective_admin(non_owner_id, stored_with_owner)` → false | negative middle | unit |

#### BLC-TEAM-004 — Team names need not be unique

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create two shared teams with same name → both succeed | positive middle | integration |

#### BLC-TEAM-005 — GET responses expose owner/members as `{ id, email }` only

**Current coverage:** None (structural). Best tested at HTTP layer by asserting response shape.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /teams/{id}` → owner/members have only `id` and `email` fields | positive middle | HTTP |

#### BLC-TEAM-006 — Personal teams must not be deleted via API

**Current coverage:** `blc_team_personal_cannot_delete`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `DELETE /teams/{personal_team_id}` → rejected (400 or 403) | negative middle | integration (exists) |

#### BLC-TEAM-007 — Team visibility rules (member/owner/admin, catalog hidden)

**Current coverage:** `blc_team_shared_create_and_list` partially.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Member lists teams → sees teams they belong to | positive middle | integration |
| 2 | Non-member lists teams → does not see team | negative middle | integration |
| 3 | Platform admin lists teams → sees all except `team:public` | positive edge | integration |
| 4 | Anyone tries `GET /teams/public` → 404 | negative edge | integration |
| 5 | `can_read_team` with app_admin=true → true (unit) | positive edge | unit |
| 6 | `can_read_team` with non-member, app_admin=false → false | negative middle | unit |

#### BLC-TEAM-008 — POST creates shared team, creator is admin, optional members

**Current coverage:** `blc_team_shared_create_and_list` covers basic case.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | POST with no extra members → creator only, as admin | positive middle | integration (exists) |
| 2 | POST with extra members → creator is admin, extras present | positive middle | integration |
| 3 | POST with creator duplicated in members list as guest → creator stays admin | positive edge | unit + integration |

#### BLC-TEAM-009 — POST must not create personal team

**Current coverage:** None (enforced by lack of `owner` param in API; test at HTTP layer).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | POST with an `owner` field in body → field ignored or rejected | negative edge | HTTP |

#### BLC-TEAM-010 — Guest or stronger member can read team

**Current coverage:** None explicit.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest reads team → 200 | positive middle | integration |
| 2 | Content maintainer reads team → 200 | positive middle | integration |
| 3 | Non-member reads team → 404 | negative middle | integration |

#### BLC-TEAM-011 — PUT replaces members, must keep ≥1 admin

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT with valid members including an admin → 200 | positive middle | integration |
| 2 | PUT removing all admins → 409 | negative middle | integration |
| 3 | PUT with owner in members on personal team → rejected | negative edge | integration |
| 4 | `ensure_shared_team_has_admin_after_update` with 0 admins → conflict | negative middle | unit |
| 5 | `ensure_shared_team_has_admin_after_update` with 1 admin → ok | positive middle | unit |
| 6 | `ensure_shared_team_has_admin_after_update` with 2 admins → ok | positive edge | unit |

#### BLC-TEAM-012 — Admin/owner may change name and members

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Admin changes name → 200 | positive middle | integration |
| 2 | Personal owner changes name → 200 | positive edge | integration |
| 3 | Admin changes members → 200 | positive middle | integration |

#### BLC-TEAM-013 — Content maintainer / guest may only self-leave

**Current coverage:** None (logic exists in `member_self_leave_payload` but untested).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest PUTs with name unchanged, members = current minus self → 200 (self-leave) | positive middle | integration |
| 2 | Guest PUTs changing name → rejected | negative middle | integration |
| 3 | Guest PUTs removing another member → rejected | negative edge | integration |
| 4 | Content maintainer self-leaves → 200 | positive edge | integration |
| 5 | `member_self_leave_payload` with correct removal → true | positive middle | unit |
| 6 | `member_self_leave_payload` with name changed → false | negative middle | unit |
| 7 | `member_self_leave_payload` with extra member removed → false | negative edge | unit |
| 8 | `member_self_leave_payload` with user not in members → false | negative edge | unit |

#### BLC-TEAM-014 — Cannot reassign personal team owner

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT personal team with different owner → rejected | negative middle | integration |

#### BLC-TEAM-015 — Removing last admin → 409

**Current coverage:** None. See BLC-TEAM-011 tests 2, 4.

#### BLC-TEAM-016 — DELETE shared team → resources reassigned to deleter's personal team

**Current coverage:** `blc_team_delete_shared_empty_team` (but does not test resource reassignment).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Shared team with songs/blobs, admin deletes → resources now on admin's personal team | positive middle | integration |
| 2 | Shared team with collections, admin deletes → collections now on personal team | positive edge | integration |
| 3 | Non-admin tries to delete shared team → rejected | negative middle | integration |

#### BLC-TEAM-017 — User deleted → personal team + items removed

**Current coverage:** None. See BLC-USER-013.

#### BLC-TEAM-018 — Shared team deleted → members lose access, items survive

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | After shared team delete, former member lists teams → team absent | positive middle | integration |
| 2 | After shared team delete, items findable on admin's personal team | positive middle | integration |

---

### Team invitation (BLC-TINV)

> Source: [team-invitation.md](./business-logic-constraints/team-invitation.md)

**Current coverage:** Zero tests in `invitation/service.rs`.

#### BLC-TINV-001 — Invitation for shared team only

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create invitation for shared team → success | positive middle | integration |
| 2 | Create invitation for personal team → rejected (400 or 404) | negative middle | integration |
| 3 | Create invitation for `team:public` → rejected | negative edge | integration |

#### BLC-TINV-002 — Only team admin may CRUD invitations

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Team admin creates invitation → success | positive middle | integration |
| 2 | Content maintainer creates invitation → 403 | negative middle | integration |
| 3 | Guest creates invitation → 403 or 404 | negative edge | integration |
| 4 | Non-member creates invitation → 404 | negative edge | integration |
| 5 | Team admin lists invitations → success | positive middle | integration |
| 6 | Guest lists invitations → 403 or 404 | negative middle | integration |

#### BLC-TINV-003 — No expiry, no max uses, no use counter (static)

Not directly testable as a constraint; accept twice to confirm reusability (BLC-TINV-013).

#### BLC-TINV-004 — DELETE permanently removes invitation

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Delete invitation, then GET → 404 | negative middle | integration |
| 2 | Delete invitation, then accept → 404 | negative edge | integration |

#### BLC-TINV-005 — Invitation survives after accept

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Accept invitation, admin GETs invitation → still exists | positive middle | integration |

#### BLC-TINV-006 — Invitation ID is unguessable (long random)

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create invitation → ID length ≥ 32 chars (UUID) | positive middle | integration |
| 2 | Two invitations for same team → different IDs | positive edge | integration |

#### BLC-TINV-007 — POST: only admin, must be valid shared team

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Admin POST for valid shared team → 201 | positive middle | integration |
| 2 | POST for non-existent team → 404 | negative middle | integration |
| 3 | POST for personal team → 400 | negative edge | integration |

#### BLC-TINV-008 — GET: only admin; wrong team/id → 404

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Admin GET own team's invitation → 200 | positive middle | integration |
| 2 | Admin GET invitation belonging to different team → 404 | negative middle | integration |
| 3 | Admin GET non-existent invitation → 404 | negative edge | integration |
| 4 | Non-admin GET invitation → 403/404 | negative edge | integration |

#### BLC-TINV-009 — DELETE: only admin; missing → 404

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Admin DELETE valid invitation → 204 | positive middle | integration |
| 2 | Admin DELETE non-existent invitation → 404 | negative middle | integration |
| 3 | Non-admin DELETE → 403/404 | negative edge | integration |

#### BLC-TINV-010 — Accept: authenticated user added as guest

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | New user accepts → becomes guest member | positive middle | integration |
| 2 | Verify team member list includes the accepter as guest | positive middle | integration |

#### BLC-TINV-011 — Accept: does not downgrade existing higher role

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User is content_maintainer, accepts invitation → role stays content_maintainer | positive middle | integration |
| 2 | User is admin, accepts invitation → role stays admin | positive edge | integration |

#### BLC-TINV-012 — Accept: no duplicate members entries

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest accepts same invitation twice → still exactly one entry | positive middle | integration |
| 2 | Check member count before and after re-accept → unchanged | positive edge | integration |

#### BLC-TINV-013 — Same invitation reusable until admin deletes

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User A accepts, User B accepts same invitation → both are members | positive middle | integration |
| 2 | After admin deletes invitation, User C tries → 404 | negative middle | integration |

#### BLC-TINV-014 — Non-admin with wrong/foreign invitation → 404

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Non-admin GETs random invitation ID → 404 | negative middle | integration |
| 2 | Non-admin accepts non-existent invitation → 404 | negative edge | integration |

---

### Blob (BLC-BLOB)

> Source: [blob.md](./business-logic-constraints/blob.md)

#### BLC-BLOB-001 — Blob belongs to one owning team (static)

**Current coverage:** Implicitly in `blc_blob_crud`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create blob → response has `owner` matching personal team | positive middle | integration (exists) |

#### BLC-BLOB-002 — Read requires team read; mutate requires library edit; admin no special edit

**Current coverage:** Partial (`blc_blob_crud` tests guest read and missing id).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Team member (guest) reads blob → 200 | positive middle | integration |
| 2 | Non-member reads blob → 404 | negative middle | integration |
| 3 | Content maintainer updates blob → 200 | positive middle | integration |
| 4 | Guest updates blob → 404 | negative middle | integration |
| 5 | Platform admin (not team member) updates blob → 404 | negative edge | integration |

#### BLC-BLOB-003 — PUT must not change owner

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT blob with different `owner` in body → owner unchanged in response | negative middle | integration |
| 2 | PUT blob with same owner → success | positive middle | integration |

#### BLC-BLOB-004 — New blobs are metadata-only

**Current coverage:** Partially in `create_calls_storage_write` mock test.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | POST blob → GET .../data returns empty/placeholder bytes | positive middle | integration |

#### BLC-BLOB-005 — `file_type` must be accepted image type

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create blob with `file_type: "image/png"` → 201 | positive middle | integration |
| 2 | Create blob with `file_type: "image/jpeg"` → 201 | positive edge | integration |
| 3 | Create blob with `file_type: "application/pdf"` → 400 | negative middle | integration |
| 4 | Create blob with `file_type: ""` → 400 | negative edge | integration |

#### BLC-BLOB-006 — Cannot read owning team's library → 404

**Current coverage:** Partially in `blc_blob_crud`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Non-member GETs blob → 404 (not 403) | negative middle | integration |
| 2 | Non-member GETs blob list → only accessible blobs returned | negative middle | integration |

#### BLC-BLOB-007 — Guest attempts PUT/DELETE → 404

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest PUTs blob → 404 | negative middle | integration |
| 2 | Guest DELETEs blob → 404 | negative middle | integration |

#### BLC-BLOB-008 — Owner/admin/content_maintainer may PUT/DELETE

**Current coverage:** Partial (delete tested in `blc_blob_crud`).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Personal owner PUTs → 200 | positive middle | integration |
| 2 | Content maintainer DELETEs → 204 | positive edge | integration |
| 3 | Team admin PUTs → 200 | positive edge | integration |

#### BLC-BLOB-009 — POST: owner is always caller's personal team

**Current coverage:** Tested in `blc_blob_crud`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create blob → `owner` is caller's personal team | positive middle | integration (exists) |

#### BLC-BLOB-010 — GET list: only blobs whose owner the caller may read

**Current coverage:** Partial.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User A has blobs, User B has blobs → each sees only own | positive middle | integration |
| 2 | Guest on A's team also sees A's blobs | positive edge | integration |

#### BLC-BLOB-011 — GET .../data: same visibility as metadata GET

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Authorized user GETs data → 200 | positive middle | integration |
| 2 | Unauthorized user GETs data → 404 | negative middle | integration |

#### BLC-BLOB-012 — PUT may only change `file_type`, `width`, `height`, `ocr`

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT changing `file_type` → reflected in response | positive middle | integration |
| 2 | PUT attempting to change `owner` → ignored or rejected | negative middle | integration |

#### BLC-BLOB-013 — DELETE removes blob from API

**Current coverage:** Tested in `blc_blob_crud`.

#### BLC-BLOB-014 — Blob used as collection cover deleted → collection may 404

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create collection with cover blob, delete blob, GET collection → 404 or degraded | negative middle | integration |

#### BLC-BLOB-015 — User deleted → blobs owned by personal team disappear

**Current coverage:** None. See BLC-USER-013.

---

### Collection (BLC-COLL)

> Source: [collection.md](./business-logic-constraints/collection.md)

#### BLC-COLL-001 — Collection belongs to one owning team

**Current coverage:** Implicitly in `blc_collection_crud_and_acl`.

#### BLC-COLL-002 — Read requires team read; mutate requires library edit

**Current coverage:** Partially in `blc_collection_crud_and_acl` (guest update denied).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Non-member reads collection → 404 | negative middle | integration |
| 2 | Guest reads collection → 200 | positive middle | integration |
| 3 | Content maintainer updates → 200 | positive middle | integration |
| 4 | Platform admin reads → 200, but mutate → rejected | positive/negative edge | integration |

#### BLC-COLL-003 — PUT must not change owner

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT with different owner → owner unchanged | negative middle | integration |

#### BLC-COLL-004 — POST/PUT may accept non-existent song IDs

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | POST collection with non-existent song ID → 201 | positive middle | integration |
| 2 | PUT collection with mix of valid and invalid song IDs → 200 | positive edge | integration |

#### BLC-COLL-005 — List supports page, page_size, q

**Current coverage:** None for collections specifically.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | `GET /collections?q=title_substring` → matching collections | positive middle | integration |
| 2 | `GET /collections?page=0&page_size=1` with 3 collections → 1 result | positive middle | integration |
| 3 | `GET /collections?q=nonexistent` → empty | negative middle | integration |

#### BLC-COLL-006 — Cannot read owning team → 404

**Current coverage:** Partial in `blc_collection_crud_and_acl`.

#### BLC-COLL-007 — Guest POST/PUT/DELETE → 404

**Current coverage:** PUT tested in `blc_collection_crud_and_acl`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest POSTs collection → 404 | negative middle | integration |
| 2 | Guest DELETEs collection → 404 | negative middle | integration |

#### BLC-COLL-008 — Owner/admin/content_maintainer may mutate

**Current coverage:** Partial.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Content maintainer creates collection → 201 | positive middle | integration |
| 2 | Team admin deletes collection → 204 | positive edge | integration |

#### BLC-COLL-009 — POST: owner is always caller's personal team

**Current coverage:** Implicit in `blc_collection_crud_and_acl`.

#### BLC-COLL-010 — List visibility + q filter

**Current coverage:** None explicit. See BLC-COLL-005.

#### BLC-COLL-011 — Sub-routes (songs, player, export) follow same visibility

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Authorized user GETs /collections/{id}/songs → 200 | positive middle | integration |
| 2 | Unauthorized user GETs /collections/{id}/player → 404 | negative middle | integration |
| 3 | Authorized user GETs /collections/{id}/export → 200 | positive middle | integration |

#### BLC-COLL-012 — GET .../songs with unresolvable song → may 500

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Collection with deleted song ID, GET songs → 500 or partial | negative middle | integration |

#### BLC-COLL-013 — PUT with non-existent song ID → 200

**Current coverage:** None. See BLC-COLL-004.

#### BLC-COLL-014 — GET songs: entry for unreadable song may be incomplete

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Collection has song from another team, owner GETs songs → entry present but incomplete | positive edge | integration |

#### BLC-COLL-015 — DELETE removes collection

**Current coverage:** Tested in `blc_collection_crud_and_acl`.

#### BLC-COLL-016 — Auto-append song requires library edit on collection's team

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Create song (default collection flow) as owner → song appended to collection | positive middle | integration |
| 2 | User without edit rights on default collection's team → song not appended / error | negative middle | integration |

#### BLC-COLL-017 — Cover blob deleted → collection may 404

**Current coverage:** None. See BLC-BLOB-014.

#### BLC-COLL-018 — User deleted → collections removed

**Current coverage:** None. See BLC-USER-013.

#### BLC-COLL-019 — Song deleted → collection endpoints may error

**Current coverage:** None. See BLC-COLL-012.

---

### Setlist (BLC-SETL)

> Source: [setlist.md](./business-logic-constraints/setlist.md)

**Current coverage:** Best-covered resource. Most constraints have tests in `setlist/service.rs`.

#### BLC-SETL-001 — Setlist belongs to one owning team

**Current coverage:** Implicitly tested.

#### BLC-SETL-002 — Read/write ACL

**Current coverage:** `blc_setl_002_team_acl_configured`.

#### BLC-SETL-003 — PUT must not change owner

**Current coverage:** None explicit.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT with different owner → owner unchanged | negative middle | integration |

#### BLC-SETL-004 — POST requires non-empty title and songs array

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | POST with valid title + songs → 201 | positive middle | integration |
| 2 | POST with empty title → 400 | negative middle | integration/HTTP |
| 3 | POST with missing title → 400 | negative edge | HTTP |
| 4 | POST with missing songs → 400 | negative edge | HTTP |
| 5 | POST with title of 1 char and empty songs array → behavior varies (document) | negative edge | integration |

#### BLC-SETL-005 — List supports page, page_size, q

**Current coverage:** `blc_setl_list_and_pagination`, `blc_setl_list_partial_pagination`, `blc_setl_search`.

#### BLC-SETL-006 — Cannot read → 404

**Current coverage:** `get_returns_not_found_when_setlist_missing` (mock).

#### BLC-SETL-007 — Guest PUT/DELETE → 404

**Current coverage:** `update_rejects_when_user_not_in_write_teams` (mock).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest DELETEs setlist → 404 | negative middle | integration (missing) |

#### BLC-SETL-008 — Owner/admin/content_maintainer may PUT/DELETE

**Current coverage:** `update_succeeds_for_owner` (mock), `blc_setl_update_acl`, `blc_setl_delete_acl`.

#### BLC-SETL-009 — POST: owner is caller's personal team

**Current coverage:** `blc_setl_create_owner_and_title`.

#### BLC-SETL-010 — List visibility + q

**Current coverage:** `blc_setl_list_and_pagination`, `blc_setl_search`.

#### BLC-SETL-011 — Sub-routes follow visibility

**Current coverage:** `blc_setl_songs_acl`, `blc_setl_player_acl`, `blc_setl_export_acl`.

#### BLC-SETL-012 — DELETE removes setlist

**Current coverage:** `blc_setl_delete_acl`.

#### BLC-SETL-013 — User deleted → setlists removed

**Current coverage:** None. See BLC-USER-013.

#### BLC-SETL-014 — Song deleted → setlist retains stale IDs

**Current coverage:** `blc_song_delete_after_setlist_link` (in song/service.rs).

---

### Song (BLC-SONG)

> Source: [song.md](./business-logic-constraints/song.md)

#### BLC-SONG-001 — Song belongs to one owning team

**Current coverage:** Implicit in `blc_song_crud_search_likes`.

#### BLC-SONG-002 — Read requires team read; mutate requires library edit

**Current coverage:** Partial.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest reads song → 200 | positive middle | integration |
| 2 | Non-member reads song → 404 | negative middle | integration |
| 3 | Guest PUTs song → 404 | negative middle | integration |
| 4 | Platform admin (non-member) PUTs → 404 | negative edge | integration |

#### BLC-SONG-003 — PUT must not change owner

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT with different owner → unchanged | negative middle | integration |

#### BLC-SONG-004 — Like state per user per song

**Current coverage:** Partially in `blc_song_crud_search_likes`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User A likes, User B does not → each sees correct state | positive middle | integration |
| 2 | User likes then unlikes → state reflects | positive edge | integration |
| 3 | Like song user cannot read → 404 | negative middle | integration |

#### BLC-SONG-005 — List supports page, page_size, q

**Current coverage:** Partial in `blc_song_crud_search_likes`.

#### BLC-SONG-006 — Cannot read → 404 (not 403)

**Current coverage:** None explicit.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Non-member GETs song → 404 (verify not 403) | negative middle | integration |

#### BLC-SONG-007 — Guest PUT/DELETE → 404

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Guest PUTs song → 404 | negative middle | integration |
| 2 | Guest DELETEs song → 404 | negative middle | integration |

#### BLC-SONG-008 — Owner/admin/content_maintainer may PUT/DELETE

**Current coverage:** Partial (delete in `blc_song_crud_search_likes`).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Content maintainer PUTs song → 200 | positive middle | integration |
| 2 | Team admin DELETEs song → 204 | positive edge | integration |

#### BLC-SONG-009 — POST: owner is caller's personal team

**Current coverage:** Tested in `blc_song_crud_search_likes`.

#### BLC-SONG-010 — POST: auto-creates/appends to default collection

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User with `default_collection` → song appended there | positive middle | integration |
| 2 | User without `default_collection` → "Default" collection created | positive edge | integration |
| 3 | User with `default_collection` pointing to non-existent collection → behavior documented | negative edge | integration |

#### BLC-SONG-011 — List: q matches title, artists, lyrics

**Current coverage:** Partial (title search in `blc_song_crud_search_likes`).

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Search by title substring → match | positive middle | integration (exists) |
| 2 | Search by artist name → match | positive middle | integration |
| 3 | Search by lyric text → match | positive edge | integration |
| 4 | Search with no match → empty | negative middle | integration |

#### BLC-SONG-012 — GET includes `liked` for current user

**Current coverage:** Partial in `blc_song_crud_search_likes`.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | User has liked song → `liked: true` | positive middle | integration |
| 2 | User has not liked song → `liked: false` | negative middle | integration |

#### BLC-SONG-013 — Player/export follow same visibility

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | Authorized user GETs /songs/{id}/player → 200 | positive middle | integration |
| 2 | Unauthorized user GETs /songs/{id}/export → 404 | negative middle | integration |

#### BLC-SONG-014 — DELETE removes song

**Current coverage:** Tested in `blc_song_crud_search_likes`.

#### BLC-SONG-015 — Song deleted → collections/setlists may retain stale IDs

**Current coverage:** `blc_song_delete_after_setlist_link`.

#### BLC-SONG-016 — User deleted → songs removed

**Current coverage:** None. See BLC-USER-013.

#### BLC-SONG-017 — PUT validation failure → 400

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT with empty `data` → 400 | negative middle | integration |
| 2 | PUT with wrong type for `tempo` → 400 | negative edge | HTTP |
| 3 | PUT with valid data → 200 | positive middle | integration |

#### BLC-SONG-018 — PUT with non-existent ID may create (upsert)

**Current coverage:** None.

| # | Test case | Type | Side |
|---|-----------|------|------|
| 1 | PUT with new ID as personal team owner → 200, song created | positive middle | integration |
| 2 | PUT with new ID as guest → 404 | negative middle | integration |
| 3 | PUT with existing ID → normal update | positive edge | integration |

---

## Implementation roadmap

### Phase 1: Pure function unit tests (no DB, fast feedback)

**Effort:** Low. **Impact:** High for `team/model.rs` which encodes core authorization logic.

**Files to add `#[cfg(test)]` modules:**

1. **`team/model.rs`** — test all pure validation functions:
   - `validate_shared_has_admin` (BLC-TEAM-011, BLC-TEAM-015)
   - `ensure_shared_team_has_admin_after_update` (BLC-TEAM-015)
   - `build_create_shared_members` (BLC-TEAM-002, BLC-TEAM-008)
   - `member_self_leave_payload` (BLC-TEAM-013)
   - `validate_personal_members_not_owner` (BLC-TEAM-001)
   - `member_or_owner_readable` (BLC-TEAM-007)
   - `effective_admin` (BLC-TEAM-003)
   - `can_read_team` (BLC-TEAM-007)
   - `team_resource_or_reject_public` (BLC-TEAM-007)
   - `parse_role` — valid and invalid strings

2. **`team/invitation/model.rs`** — test:
   - `invitation_thing` (empty, valid, with table prefix, wrong table)
   - `team_things_match` (same, different table, different id)

3. **`user/model.rs`** — test:
   - `user_resource` (plain id, Thing with correct table, Thing with wrong table)

**Estimated tests: ~40**

### Phase 2: Mock-based service unit tests

**Effort:** Medium. **Impact:** High — covers ACL logic without DB.

Create mock implementations for:
- `TeamInvitationRepository`
- `UserRepository`
- `TeamRepository`
- `SessionRepository`

Add test modules in:

1. **`team/invitation/service.rs`** — BLC-TINV-001 through BLC-TINV-014 (~25 tests)
2. **`user/service.rs`** — BLC-USER-001, 003, 008 (~8 tests)
3. **`user/session/service.rs`** — BLC-SESS-001, 003, 004, 009 (~8 tests)

**Estimated tests: ~41**

### Phase 3: Integration tests with `test_db()`

**Effort:** Medium-high. **Impact:** Covers full stack including SurrealQL.

Extend existing integration tests and add new ones:

1. **`team/service.rs`** — BLC-TEAM-007 to 018 (~20 new tests)
   - Multi-role fixture for team operations
   - Shared team deletion with resource reassignment
   - Self-leave scenarios
2. **`team/invitation/service.rs`** — Full CRUD + accept flow (~15 tests)
3. **`blob/service.rs`** — BLC-BLOB-003, 005, 007, 008, 012 (~12 tests)
4. **`collection/service.rs`** — BLC-COLL-002 to 016 (~15 tests)
5. **`song/service.rs`** — BLC-SONG-002 to 018 (~18 tests)
6. **`user/service.rs`** — BLC-USER-001, 003, 012, 013, 014 (~10 tests)

**Estimated tests: ~90**

### Phase 4: HTTP-layer tests with `actix_web::test`

**Effort:** High (new test harness needed). **Impact:** Covers auth, status codes, response shapes.

Create `backend/src/resources/rest_tests.rs` (or per-resource `http_tests` modules):

1. **Auth middleware** — BLC-AUTH-001, BLC-AUTH-002 (~6 tests)
2. **OpenAPI** — BLC-DOCS-001 (~3 tests)
3. **HTTP contract** — BLC-HTTP-001, BLC-HTTP-002 (~6 tests)
4. **User admin gates** — BLC-USER-005 to 009 (~10 tests)
5. **Session admin gates** — BLC-SESS-005 to 007 (~8 tests)
6. **List pagination** — BLC-LP-004 to 009 across all resources (~15 tests)

**Estimated tests: ~48**

### Phase 5: Cascading delete integration tests

**Effort:** Medium. **Impact:** Validates the most complex cross-resource behavior.

These tests span multiple resources and need careful setup:

1. **User deletion cascade** — BLC-USER-012, 013; BLC-TEAM-017; BLC-BLOB-015; BLC-COLL-018; BLC-SETL-013; BLC-SONG-016 (~6 comprehensive tests)
2. **Shared team deletion cascade** — BLC-TEAM-016, 018 (~4 tests)
3. **Blob → collection cascade** — BLC-BLOB-014, BLC-COLL-017 (~2 tests)
4. **Song → collection/setlist cascade** — BLC-SONG-015, BLC-COLL-012, 019, BLC-SETL-014 (~4 tests)

**Estimated tests: ~16**

---

### Summary

| Phase | Description | Est. new tests | Prerequisite |
|-------|-------------|----------------|-------------|
| 1 | Pure function unit tests | ~40 | None |
| 2 | Mock-based service tests | ~41 | Mock impls |
| 3 | Integration tests (DB) | ~90 | TeamFixture builder |
| 4 | HTTP-layer tests | ~48 | actix_web::test harness |
| 5 | Cascading delete tests | ~16 | Phases 3–4 |
| **Total** | | **~235** | |

Phase 1 can be started immediately with zero infrastructure changes. Phase 2 requires creating mock repository implementations (or adopting `mockall`). Phases 3–5 benefit from the `TeamFixture` builder described in the architectural improvements section.

---

## Commit-sized implementation slices

Each slice below is designed to be a single, self-contained commit. Slices within a phase can generally be done in any order unless noted. Cross-phase dependencies are called out explicitly.

### Phase 1 slices — Pure function unit tests

#### Slice 1A: `team/model.rs` — validation helpers

**File:** `backend/src/resources/team/model.rs`
**BLCs:** BLC-TEAM-001, BLC-TEAM-011, BLC-TEAM-015

Add `#[cfg(test)] mod tests` with:

| # | Test | BLC |
|---|------|-----|
| 1 | `validate_shared_has_admin` with 1 admin → ok | BLC-TEAM-011 |
| 2 | `validate_shared_has_admin` with 2 admins → ok | BLC-TEAM-011 |
| 3 | `validate_shared_has_admin` with 0 admins → error | BLC-TEAM-011 |
| 4 | `validate_shared_has_admin` with only guests → error | BLC-TEAM-011 |
| 5 | `ensure_shared_team_has_admin_after_update` with 1 admin → ok | BLC-TEAM-015 |
| 6 | `ensure_shared_team_has_admin_after_update` with 0 admins → conflict | BLC-TEAM-015 |
| 7 | `validate_personal_members_not_owner` with owner absent → ok | BLC-TEAM-001 |
| 8 | `validate_personal_members_not_owner` with owner in members → error | BLC-TEAM-001 |
| 9 | `validate_personal_members_not_owner` with empty members → ok | BLC-TEAM-001 |
| 10 | `member_user_id` with valid id → ok | — |
| 11 | `member_user_id` with empty string → error | — |
| 12 | `member_user_id` with whitespace-only → error | — |

**~12 tests**

---

#### Slice 1B: `team/model.rs` — `build_create_shared_members`

**File:** `backend/src/resources/team/model.rs`
**BLCs:** BLC-TEAM-002, BLC-TEAM-008

Extend the `#[cfg(test)]` module from slice 1A:

| # | Test | BLC |
|---|------|-----|
| 1 | Creator alone → single admin member | BLC-TEAM-002 |
| 2 | Creator + extra guest → creator is admin, guest present | BLC-TEAM-008 |
| 3 | Creator duplicated in extras as guest → stays admin, no duplicate | BLC-TEAM-008 |
| 4 | Creator + extra admin + extra guest → all present, creator admin | BLC-TEAM-008 |
| 5 | Two extras with same user id → deduplicated | BLC-TEAM-008 |
| 6 | Extra with empty user id → error | — |

**~6 tests**

---

#### Slice 1C: `team/model.rs` — ACL and self-leave helpers

**File:** `backend/src/resources/team/model.rs`
**BLCs:** BLC-TEAM-003, BLC-TEAM-007, BLC-TEAM-013

Extend the `#[cfg(test)]` module:

| # | Test | BLC |
|---|------|-----|
| 1 | `effective_admin` — personal team owner → true | BLC-TEAM-003 |
| 2 | `effective_admin` — shared team admin member → true | BLC-TEAM-003 |
| 3 | `effective_admin` — shared team content_maintainer → false | BLC-TEAM-003 |
| 4 | `effective_admin` — non-member → false | BLC-TEAM-003 |
| 5 | `member_or_owner_readable` — owner → true | BLC-TEAM-007 |
| 6 | `member_or_owner_readable` — guest member → true | BLC-TEAM-007 |
| 7 | `member_or_owner_readable` — non-member → false | BLC-TEAM-007 |
| 8 | `can_read_team` — non-member, app_admin=false → false | BLC-TEAM-007 |
| 9 | `can_read_team` — non-member, app_admin=true → true | BLC-TEAM-007 |
| 10 | `can_read_team` — member, app_admin=false → true | BLC-TEAM-007 |
| 11 | `member_self_leave_payload` — correct self-removal → true | BLC-TEAM-013 |
| 12 | `member_self_leave_payload` — name changed → false | BLC-TEAM-013 |
| 13 | `member_self_leave_payload` — extra member removed → false | BLC-TEAM-013 |
| 14 | `member_self_leave_payload` — user not in members → false | BLC-TEAM-013 |
| 15 | `member_self_leave_payload` — name with leading/trailing whitespace matches trimmed → true | BLC-TEAM-013 |

**~15 tests**

---

#### Slice 1D: `team/model.rs` — `team_resource_or_reject_public` and `parse_role`

**File:** `backend/src/resources/team/model.rs`
**BLCs:** BLC-TEAM-007

Extend the `#[cfg(test)]` module:

| # | Test | BLC |
|---|------|-----|
| 1 | `team_resource_or_reject_public("public")` → NotFound | BLC-TEAM-007 |
| 2 | `team_resource_or_reject_public("some-uuid")` → ok | BLC-TEAM-007 |
| 3 | `team_resource_or_reject_public("team:some-uuid")` → ok, parsed | BLC-TEAM-007 |
| 4 | `parse_role("guest")` → Guest | — |
| 5 | `parse_role("content_maintainer")` → ContentMaintainer | — |
| 6 | `parse_role("admin")` → Admin | — |
| 7 | `parse_role("superadmin")` → error | — |
| 8 | `parse_role("")` → error | — |

**~8 tests** (requires making `parse_role` `pub(crate)` or testing via `TeamFetched::into_team`)

---

#### Slice 1E: `team/invitation/model.rs` — invitation helpers

**File:** `backend/src/resources/team/invitation/model.rs`
**BLCs:** BLC-TINV-006 (indirectly)

Add `#[cfg(test)] mod tests`:

| # | Test | BLC |
|---|------|-----|
| 1 | `invitation_thing("valid-uuid")` → ok, table=team_invitation | BLC-TINV-006 |
| 2 | `invitation_thing("team_invitation:abc")` → ok, parsed | — |
| 3 | `invitation_thing("")` → NotFound | — |
| 4 | `invitation_thing("   ")` → NotFound | — |
| 5 | `invitation_thing("other_table:abc")` → ok, falls back to team_invitation:other_table:abc | — |
| 6 | `team_things_match` — same table and id → true | — |
| 7 | `team_things_match` — same table, different id → false | — |
| 8 | `team_things_match` — different table → false | — |

**~8 tests**

---

#### Slice 1F: `user/model.rs` — `user_resource`

**File:** `backend/src/resources/user/model.rs`
**BLCs:** BLC-HTTP-001 (indirectly)

Add `#[cfg(test)] mod tests`:

| # | Test | BLC |
|---|------|-----|
| 1 | `user_resource("some-uuid")` → ok, table=user | — |
| 2 | `user_resource("user:some-uuid")` → ok, parsed | — |
| 3 | `user_resource("team:abc")` → error (wrong table) | BLC-HTTP-001 |

**~3 tests**

---

### Phase 2 slices — Mock-based service unit tests

#### Slice 2A: Introduce mock repository infrastructure

**Files:** New or extended mock impls, likely in each service's `#[cfg(test)]` module or a shared test module.

No BLC tests yet — this is scaffolding. Create `MockTeamRepository`, `MockTeamInvitationRepository`, `MockUserRepository`, `MockSessionRepository` implementing the respective traits. Can use `mockall` or hand-rolled structs (follow the pattern already used in `blob/service.rs` and `setlist/service.rs`).

**Deliverable:** Mock structs compile and are importable from `#[cfg(test)]` modules.

---

#### Slice 2B: `team/invitation/service.rs` — CRUD access control (mocks)

**File:** `backend/src/resources/team/invitation/service.rs`
**BLCs:** BLC-TINV-001, BLC-TINV-002, BLC-TINV-007, BLC-TINV-008, BLC-TINV-009
**Depends on:** Slice 2A

| # | Test | BLC |
|---|------|-----|
| 1 | Create invitation — shared team, admin → success | BLC-TINV-001, BLC-TINV-007 |
| 2 | Create invitation — personal team → error | BLC-TINV-001 |
| 3 | Create invitation — `team:public` → NotFound | BLC-TINV-001 |
| 4 | Create invitation — content_maintainer → forbidden | BLC-TINV-002 |
| 5 | Create invitation — guest → forbidden/not found | BLC-TINV-002 |
| 6 | Create invitation — non-member → not found | BLC-TINV-002 |
| 7 | List invitations — admin → success | BLC-TINV-008 |
| 8 | List invitations — non-admin → forbidden | BLC-TINV-002 |
| 9 | Get invitation — admin, correct team → success | BLC-TINV-008 |
| 10 | Get invitation — admin, wrong team → not found | BLC-TINV-008 |
| 11 | Get invitation — non-existent id → not found | BLC-TINV-008 |
| 12 | Delete invitation — admin → success | BLC-TINV-009 |
| 13 | Delete invitation — non-existent → not found | BLC-TINV-009 |
| 14 | Delete invitation — non-admin → forbidden | BLC-TINV-009 |

**~14 tests**

---

#### Slice 2C: `team/invitation/service.rs` — accept logic (mocks)

**File:** `backend/src/resources/team/invitation/service.rs`
**BLCs:** BLC-TINV-010, BLC-TINV-011, BLC-TINV-012, BLC-TINV-014
**Depends on:** Slice 2A

| # | Test | BLC |
|---|------|-----|
| 1 | Accept — new user becomes guest | BLC-TINV-010 |
| 2 | Accept — invitation for `team:public` → not found | BLC-TINV-001 |
| 3 | Accept — invitation for personal team → not found | BLC-TINV-001 |
| 4 | Accept — user already content_maintainer → role unchanged | BLC-TINV-011 |
| 5 | Accept — user already admin → role unchanged | BLC-TINV-011 |
| 6 | Accept — user already guest → no duplicate entry | BLC-TINV-012 |
| 7 | Accept — non-existent invitation → not found | BLC-TINV-014 |

**~7 tests**

---

#### Slice 2D: `user/service.rs` — user creation and email (mocks)

**File:** `backend/src/resources/user/service.rs`
**BLCs:** BLC-USER-001, BLC-USER-003, BLC-USER-008
**Depends on:** Slice 2A

| # | Test | BLC |
|---|------|-----|
| 1 | Create user → personal team also created | BLC-USER-003 |
| 2 | Create user → returned user has role `default` | BLC-USER-002 |
| 3 | `get_user_by_email_or_create` — new email → creates | BLC-USER-001 |
| 4 | `get_user_by_email_or_create` — existing email → returns existing | BLC-USER-001 |
| 5 | Create user → duplicate email from repo → error propagated | BLC-USER-008 |

**~5 tests**

---

#### Slice 2E: `user/session/service.rs` — session scoping (mocks)

**File:** `backend/src/resources/user/session/service.rs`
**BLCs:** BLC-SESS-001, BLC-SESS-009
**Depends on:** Slice 2A

| # | Test | BLC |
|---|------|-----|
| 1 | Create session → session user matches input | BLC-SESS-001 |
| 2 | `create_session_for_user_by_id` — valid user → session created | BLC-SESS-001 |
| 3 | `create_session_for_user_by_id` — non-existent user → error | BLC-SESS-001 |
| 4 | Delete session → subsequent validate returns None | BLC-SESS-009 |
| 5 | `get_sessions_by_user_id` → returns only that user's sessions | BLC-SESS-003 |

**~5 tests**

---

### Phase 3 slices — Integration tests with `test_db()`

#### Slice 3A: `TeamFixture` builder in test helpers

**File:** `backend/src/test_helpers.rs`
**BLCs:** None (infrastructure)

Add a `TeamFixture` struct and builder that creates:
- An owner user with personal team
- A shared team with admin, content_maintainer, guest members
- A non-member user
- A platform admin user

All subsequent Phase 3 slices benefit from this. Can also be used retroactively in existing tests.

---

#### Slice 3B: `team/service.rs` — visibility and read ACL

**File:** `backend/src/resources/team/service.rs`
**BLCs:** BLC-TEAM-004, BLC-TEAM-007, BLC-TEAM-010
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Two shared teams with same name → both created | BLC-TEAM-004 |
| 2 | Member lists teams → sees own teams | BLC-TEAM-007 |
| 3 | Non-member lists teams → does not see team | BLC-TEAM-007 |
| 4 | Platform admin lists teams → sees all except `team:public` | BLC-TEAM-007 |
| 5 | `GET /teams/public` → not found | BLC-TEAM-007 |
| 6 | Guest reads shared team → ok | BLC-TEAM-010 |
| 7 | Content maintainer reads shared team → ok | BLC-TEAM-010 |
| 8 | Non-member reads shared team → not found | BLC-TEAM-010 |

**~8 tests**

---

#### Slice 3C: `team/service.rs` — update and self-leave

**File:** `backend/src/resources/team/service.rs`
**BLCs:** BLC-TEAM-011, BLC-TEAM-012, BLC-TEAM-013, BLC-TEAM-014, BLC-TEAM-015
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Admin changes shared team name → ok | BLC-TEAM-012 |
| 2 | Admin changes members → ok | BLC-TEAM-012 |
| 3 | Personal owner changes name → ok | BLC-TEAM-012 |
| 4 | PUT removing all admins → 409 | BLC-TEAM-011, BLC-TEAM-015 |
| 5 | PUT with owner in personal team members → rejected | BLC-TEAM-011 |
| 6 | Guest self-leaves (name unchanged, members minus self) → ok | BLC-TEAM-013 |
| 7 | Guest changes name → rejected | BLC-TEAM-013 |
| 8 | Guest removes another member → rejected | BLC-TEAM-013 |
| 9 | Content maintainer self-leaves → ok | BLC-TEAM-013 |
| 10 | PUT personal team with different owner → rejected | BLC-TEAM-014 |

**~10 tests**

---

#### Slice 3D: `team/service.rs` — shared team delete with reassignment

**File:** `backend/src/resources/team/service.rs`
**BLCs:** BLC-TEAM-016, BLC-TEAM-018
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Shared team with songs, admin deletes → songs on admin's personal team | BLC-TEAM-016 |
| 2 | Shared team with collections, admin deletes → collections reassigned | BLC-TEAM-016 |
| 3 | Shared team with blobs, admin deletes → blobs reassigned | BLC-TEAM-016 |
| 4 | Non-admin attempts delete → rejected | BLC-TEAM-016 |
| 5 | After delete, former member lists teams → team absent | BLC-TEAM-018 |
| 6 | After delete, items findable on admin's personal team | BLC-TEAM-018 |

**~6 tests**

---

#### Slice 3E: `team/invitation/service.rs` — full CRUD + accept integration

**File:** `backend/src/resources/team/invitation/service.rs`
**BLCs:** BLC-TINV-001 through BLC-TINV-014
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Admin creates invitation for shared team → success, UUID id | BLC-TINV-001, BLC-TINV-006, BLC-TINV-007 |
| 2 | Create for personal team → rejected | BLC-TINV-001 |
| 3 | Content maintainer create → forbidden | BLC-TINV-002 |
| 4 | Admin lists → sees invitation | BLC-TINV-008 |
| 5 | Admin GETs invitation by id → matches | BLC-TINV-008 |
| 6 | Admin GET wrong team's invitation → not found | BLC-TINV-008 |
| 7 | Admin deletes invitation → gone | BLC-TINV-004, BLC-TINV-009 |
| 8 | Deleted invitation accept → not found | BLC-TINV-004 |
| 9 | New user accepts → becomes guest | BLC-TINV-010 |
| 10 | Accept invitation, admin GETs → still exists | BLC-TINV-005 |
| 11 | Content maintainer accepts → role unchanged | BLC-TINV-011 |
| 12 | Admin accepts → role unchanged | BLC-TINV-011 |
| 13 | Guest accepts twice → no duplicate | BLC-TINV-012 |
| 14 | User A accepts, User B accepts same → both members | BLC-TINV-013 |
| 15 | Two invitations for same team → different IDs | BLC-TINV-006 |

**~15 tests**

---

#### Slice 3F: `blob/service.rs` — ACL and validation gaps

**File:** `backend/src/resources/blob/service.rs`
**BLCs:** BLC-BLOB-002, BLC-BLOB-003, BLC-BLOB-005, BLC-BLOB-007, BLC-BLOB-008, BLC-BLOB-012
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Non-member reads blob → not found | BLC-BLOB-002 |
| 2 | Content maintainer updates blob → ok | BLC-BLOB-002 |
| 3 | Guest updates blob → not found | BLC-BLOB-007 |
| 4 | Guest deletes blob → not found | BLC-BLOB-007 |
| 5 | Personal owner PUTs blob → ok | BLC-BLOB-008 |
| 6 | Team admin PUTs blob → ok | BLC-BLOB-008 |
| 7 | PUT does not change owner | BLC-BLOB-003 |
| 8 | PUT only changes `file_type`, `width`, `height`, `ocr` | BLC-BLOB-012 |
| 9 | Create blob with `image/png` → ok | BLC-BLOB-005 |
| 10 | Create blob with `application/pdf` → rejected | BLC-BLOB-005 |
| 11 | Create blob with empty file_type → rejected | BLC-BLOB-005 |
| 12 | Platform admin (non-member) updates blob → not found | BLC-BLOB-002 |

**~12 tests**

---

#### Slice 3G: `collection/service.rs` — ACL and CRUD gaps

**File:** `backend/src/resources/collection/service.rs`
**BLCs:** BLC-COLL-002 through BLC-COLL-011
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Non-member reads collection → not found | BLC-COLL-002, BLC-COLL-006 |
| 2 | Guest reads collection → ok | BLC-COLL-002 |
| 3 | Content maintainer updates collection → ok | BLC-COLL-002 |
| 4 | Guest POSTs collection → not found | BLC-COLL-007 |
| 5 | Guest DELETEs collection → not found | BLC-COLL-007 |
| 6 | PUT does not change owner | BLC-COLL-003 |
| 7 | POST with non-existent song IDs → 201 | BLC-COLL-004 |
| 8 | List with `q` filter → matches title | BLC-COLL-005, BLC-COLL-010 |
| 9 | List with pagination → correct page | BLC-COLL-005 |
| 10 | Authorized user GETs /songs sub-route → ok | BLC-COLL-011 |
| 11 | Unauthorized user GETs /player sub-route → not found | BLC-COLL-011 |

**~11 tests**

---

#### Slice 3H: `song/service.rs` — ACL, search, and upsert gaps

**File:** `backend/src/resources/song/service.rs`
**BLCs:** BLC-SONG-002 through BLC-SONG-013, BLC-SONG-017, BLC-SONG-018
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Non-member reads song → not found (not 403) | BLC-SONG-002, BLC-SONG-006 |
| 2 | Guest PUTs song → not found | BLC-SONG-007 |
| 3 | Guest DELETEs song → not found | BLC-SONG-007 |
| 4 | Content maintainer PUTs song → ok | BLC-SONG-008 |
| 5 | PUT does not change owner | BLC-SONG-003 |
| 6 | Search by artist name → match | BLC-SONG-011 |
| 7 | Search by lyric text → match | BLC-SONG-011 |
| 8 | Search with no match → empty | BLC-SONG-011 |
| 9 | GET song includes `liked: true` when liked | BLC-SONG-012 |
| 10 | GET song includes `liked: false` when not liked | BLC-SONG-012 |
| 11 | User A likes, User B does not → independent | BLC-SONG-004 |
| 12 | Like song user cannot read → not found | BLC-SONG-004 |
| 13 | PUT with new ID as owner → song created (upsert) | BLC-SONG-018 |
| 14 | PUT with new ID as guest → not found | BLC-SONG-018 |
| 15 | Authorized user GETs player → ok | BLC-SONG-013 |
| 16 | Unauthorized user GETs export → not found | BLC-SONG-013 |

**~16 tests**

---

#### Slice 3I: `song/service.rs` — default collection auto-create

**File:** `backend/src/resources/song/service.rs`
**BLCs:** BLC-SONG-010, BLC-COLL-016
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | User with `default_collection` → song appended to existing collection | BLC-SONG-010 |
| 2 | User without `default_collection` → "Default" collection created + song placed | BLC-SONG-010 |
| 3 | `default_collection` points to non-existent collection → behavior documented | BLC-SONG-010 |

**~3 tests**

---

#### Slice 3J: `user/service.rs` — user lifecycle integration

**File:** `backend/src/resources/user/service.rs`
**BLCs:** BLC-USER-001, BLC-USER-003, BLC-USER-008, BLC-USER-014
**Depends on:** Slice 3A

| # | Test | BLC |
|---|------|-----|
| 1 | Create user → personal team exists with correct owner | BLC-USER-003 |
| 2 | Create user → personal team has empty members | BLC-USER-003 |
| 3 | Duplicate email (case-insensitive) → conflict | BLC-USER-001 |
| 4 | Two distinct emails → both succeed | BLC-USER-001 |
| 5 | Delete user, delete again → not found | BLC-USER-014 |
| 6 | Delete non-existent user → not found | BLC-USER-014 |

**~6 tests**

---

### Phase 4 slices — HTTP-layer tests

#### Slice 4A: HTTP test harness setup

**File:** `backend/src/resources/rest_tests.rs` (new) or similar
**BLCs:** None (infrastructure)

Create the `actix_web::test`-based harness:
- Helper to build a test `App` with all services wired to `test_db()`
- Helper to create an authenticated request (valid session token)
- Helper to create an unauthenticated request

---

#### Slice 4B: Auth middleware tests

**File:** HTTP test module
**BLCs:** BLC-AUTH-001, BLC-AUTH-002
**Depends on:** Slice 4A

| # | Test | BLC |
|---|------|-----|
| 1 | No `Authorization` header → 401 | BLC-AUTH-001 |
| 2 | `Authorization: Basic abc` → 401 | BLC-AUTH-001 |
| 3 | Empty `Authorization:` header → 401 | BLC-AUTH-001 |
| 4 | `Authorization: Bearer invalidtoken` → 401 | BLC-AUTH-002 |
| 5 | `Authorization: Bearer <deleted-session>` → 401 | BLC-AUTH-002 |
| 6 | Valid `Authorization: Bearer <token>` → passes (not 401) | BLC-AUTH-001 |

**~6 tests**

---

#### Slice 4C: OpenAPI endpoint test

**File:** HTTP test module
**BLCs:** BLC-DOCS-001
**Depends on:** Slice 4A

| # | Test | BLC |
|---|------|-----|
| 1 | `GET /api/docs/openapi.json` without auth → 200, valid JSON | BLC-DOCS-001 |
| 2 | `GET /api/docs/openapi.json` with auth → still 200 | BLC-DOCS-001 |
| 3 | `GET /api/v1/docs/openapi.json` → 404 | BLC-DOCS-001 |

**~3 tests**

---

#### Slice 4D: HTTP contract — invalid path IDs and idempotent DELETE

**File:** HTTP test module
**BLCs:** BLC-HTTP-001, BLC-HTTP-002
**Depends on:** Slice 4A

| # | Test | BLC |
|---|------|-----|
| 1 | `GET /songs/blob:wrongtable` → 400 | BLC-HTTP-001 |
| 2 | `DELETE /setlists/collection:x` → 400 | BLC-HTTP-001 |
| 3 | `GET /songs/song:validid` → 200 or 404 (not 400) | BLC-HTTP-001 |
| 4 | `GET /songs/plainid` → 200 or 404 (not 400) | BLC-HTTP-001 |
| 5 | DELETE song, repeat DELETE → 404 | BLC-HTTP-002 |
| 6 | DELETE non-existent → 404 | BLC-HTTP-002 |

**~6 tests**

---

#### Slice 4E: User admin gates

**File:** HTTP test module
**BLCs:** BLC-USER-005, BLC-USER-006, BLC-USER-007, BLC-USER-009
**Depends on:** Slice 4A

| # | Test | BLC |
|---|------|-----|
| 1 | Authenticated `GET /users/me` → 200, matches user | BLC-USER-005 |
| 2 | Different users each see own record via `/users/me` | BLC-USER-005 |
| 3 | Raw token (no Bearer) on `GET /users/me` → 200 | BLC-USER-006 |
| 4 | Raw token on other endpoint → 401 | BLC-USER-006 |
| 5 | Non-admin `GET /users` → 403 | BLC-USER-007 |
| 6 | Non-admin `POST /users` → 403 | BLC-USER-007 |
| 7 | Non-admin `DELETE /users/{id}` → 403 | BLC-USER-007 |
| 8 | Non-admin `GET /users/{id}` → 403 | BLC-USER-007 |
| 9 | Admin `GET /users` → 200 | BLC-USER-007 |
| 10 | Admin `GET /users/{id}` → 200 | BLC-USER-009 |

**~10 tests**

---

#### Slice 4F: Session admin gates

**File:** HTTP test module
**BLCs:** BLC-SESS-003, BLC-SESS-004, BLC-SESS-005, BLC-SESS-006, BLC-SESS-009
**Depends on:** Slice 4A

| # | Test | BLC |
|---|------|-----|
| 1 | `GET /users/me/sessions` → only own sessions | BLC-SESS-003 |
| 2 | User with 0 sessions → 200, empty | BLC-SESS-003 |
| 3 | DELETE own session → ok | BLC-SESS-004 |
| 4 | GET other user's session via `/me/sessions/{id}` → 404 | BLC-SESS-004 |
| 5 | Non-admin `GET /users/{other}/sessions` → 403 | BLC-SESS-005 |
| 6 | Non-admin `POST /users/{other}/sessions` → 403 | BLC-SESS-005 |
| 7 | Admin `GET /users/{other}/sessions` → 200 | BLC-SESS-006 |
| 8 | Admin `DELETE /users/{other}/sessions/{id}` → ok | BLC-SESS-006 |
| 9 | Admin invalid user_id → 404 | BLC-SESS-006 |
| 10 | Deleted session token on authenticated route → 401 | BLC-SESS-009 |

**~10 tests**

---

#### Slice 4G: List pagination HTTP validation

**File:** HTTP test module
**BLCs:** BLC-LP-004 through BLC-LP-009
**Depends on:** Slice 4A

| # | Test | BLC |
|---|------|-----|
| 1 | `page=abc` → 400 | BLC-LP-004 |
| 2 | `page_size=1.5` → 400 | BLC-LP-004 |
| 3 | `page=0&page_size=10` → 200 | BLC-LP-004 |
| 4 | `q=%20%20` → same as no `q` | BLC-LP-005 |
| 5 | `q=` (empty) → same as no `q` | BLC-LP-005 |
| 6 | `page_size=0` → returns all | BLC-LP-006 |
| 7 | Only `page` supplied (no `page_size`) → 200 | BLC-LP-007 |
| 8 | Only `page_size` supplied (no `page`) → 200 | BLC-LP-007 |
| 9 | `page=999` → 200, empty array | BLC-LP-008 |
| 10 | `q` + pagination → filter first, then page | BLC-LP-009 |

**~10 tests**

---

### Phase 5 slices — Cascading delete integration tests

#### Slice 5A: User deletion cascade

**File:** `backend/src/resources/user/service.rs` (or a dedicated cross-resource test file)
**BLCs:** BLC-USER-012, BLC-USER-013, BLC-TEAM-017, BLC-BLOB-015, BLC-COLL-018, BLC-SETL-013, BLC-SONG-016
**Depends on:** Slices 3A, 3J

| # | Test | BLC |
|---|------|-----|
| 1 | Create user + session, delete user, validate session → None | BLC-USER-012, BLC-SESS-008 |
| 2 | Create user + 2 sessions, delete user, both invalid | BLC-USER-012 |
| 3 | Create user + song + collection + blob + setlist, delete user → all gone | BLC-USER-013, BLC-SONG-016, BLC-COLL-018, BLC-BLOB-015, BLC-SETL-013 |
| 4 | Guest on deleted user's personal team → resources 404 | BLC-USER-013, BLC-TEAM-017 |

**~4 tests**

---

#### Slice 5B: Shared team deletion cascade

**File:** `backend/src/resources/team/service.rs` (extend from Slice 3D or same file)
**BLCs:** BLC-TEAM-016, BLC-TEAM-018

If already covered in Slice 3D, this slice is complete. Otherwise add any remaining:

| # | Test | BLC |
|---|------|-----|
| 1 | Shared team with setlists, admin deletes → setlists on personal team | BLC-TEAM-016 |
| 2 | After delete, items still fetchable by admin via personal team | BLC-TEAM-018 |

**~2 tests**

---

#### Slice 5C: Blob → collection cascade

**File:** Cross-resource test or `blob/service.rs`
**BLCs:** BLC-BLOB-014, BLC-COLL-017

| # | Test | BLC |
|---|------|-----|
| 1 | Collection with cover blob, delete blob, GET collection → 404 or degraded | BLC-BLOB-014, BLC-COLL-017 |
| 2 | Collection without cover, delete unrelated blob → collection unaffected | BLC-BLOB-014 |

**~2 tests**

---

#### Slice 5D: Song → collection / setlist cascade

**File:** Cross-resource test or `song/service.rs`
**BLCs:** BLC-SONG-015, BLC-COLL-012, BLC-COLL-019, BLC-SETL-014

| # | Test | BLC |
|---|------|-----|
| 1 | Song in collection, delete song, GET collection songs → 500 or partial | BLC-COLL-012, BLC-COLL-019 |
| 2 | Song in setlist, delete song, GET setlist songs → stale ID present | BLC-SETL-014 |
| 3 | PUT collection/setlist after song delete to clean up → ok | BLC-SONG-015 |
| 4 | Song deleted, collection PUT with valid+stale ids → 200 | BLC-SONG-015 |

**~4 tests**

---

## Slice summary

| Slice | Scope | BLCs | Est. tests | Depends on |
|-------|-------|------|------------|------------|
| **1A** | `team/model` — validation helpers | TEAM-001, 011, 015 | 12 | — |
| **1B** | `team/model` — `build_create_shared_members` | TEAM-002, 008 | 6 | — |
| **1C** | `team/model` — ACL + self-leave helpers | TEAM-003, 007, 013 | 15 | — |
| **1D** | `team/model` — public team + parse_role | TEAM-007 | 8 | — |
| **1E** | `invitation/model` — invitation helpers | TINV-006 | 8 | — |
| **1F** | `user/model` — `user_resource` | HTTP-001 | 3 | — |
| **2A** | Mock repository infrastructure | — | 0 | — |
| **2B** | `invitation/service` — CRUD access (mocks) | TINV-001, 002, 007–009 | 14 | 2A |
| **2C** | `invitation/service` — accept logic (mocks) | TINV-010–012, 014 | 7 | 2A |
| **2D** | `user/service` — user creation (mocks) | USER-001–003, 008 | 5 | 2A |
| **2E** | `session/service` — session scoping (mocks) | SESS-001, 003, 009 | 5 | 2A |
| **3A** | `TeamFixture` builder | — | 0 | — |
| **3B** | `team/service` — visibility + read ACL | TEAM-004, 007, 010 | 8 | 3A |
| **3C** | `team/service` — update + self-leave | TEAM-011–015 | 10 | 3A |
| **3D** | `team/service` — shared delete + reassignment | TEAM-016, 018 | 6 | 3A |
| **3E** | `invitation/service` — full CRUD + accept integration | TINV-001–014 | 15 | 3A |
| **3F** | `blob/service` — ACL + validation gaps | BLOB-002, 003, 005, 007, 008, 012 | 12 | 3A |
| **3G** | `collection/service` — ACL + CRUD gaps | COLL-002–011 | 11 | 3A |
| **3H** | `song/service` — ACL, search, upsert | SONG-002–013, 017, 018 | 16 | 3A |
| **3I** | `song/service` — default collection auto-create | SONG-010, COLL-016 | 3 | 3A |
| **3J** | `user/service` — user lifecycle integration | USER-001, 003, 008, 014 | 6 | 3A |
| **4A** | HTTP test harness setup | — | 0 | — |
| **4B** | Auth middleware tests | AUTH-001, 002 | 6 | 4A |
| **4C** | OpenAPI endpoint | DOCS-001 | 3 | 4A |
| **4D** | HTTP contract — path IDs + idempotent DELETE | HTTP-001, 002 | 6 | 4A |
| **4E** | User admin gates | USER-005–007, 009 | 10 | 4A |
| **4F** | Session admin gates | SESS-003–006, 009 | 10 | 4A |
| **4G** | List pagination HTTP validation | LP-004–009 | 10 | 4A |
| **5A** | User deletion cascade | USER-012, 013; TEAM-017; BLOB-015; COLL-018; SETL-013; SONG-016 | 4 | 3A, 3J |
| **5B** | Shared team deletion cascade | TEAM-016, 018 | 2 | 3D |
| **5C** | Blob → collection cascade | BLOB-014, COLL-017 | 2 | 3F, 3G |
| **5D** | Song → collection/setlist cascade | SONG-015; COLL-012, 019; SETL-014 | 4 | 3G, 3H |
| | | **Total** | **~235** | |

### Suggested commit order

```
Phase 1 (any order, no dependencies):
  1A → 1B → 1C → 1D → 1E → 1F

Phase 2 (2A first, then any order):
  2A → 2B → 2C → 2D → 2E

Phase 3 (3A first, then any order):
  3A → 3B → 3C → 3D → 3E → 3F → 3G → 3H → 3I → 3J

Phase 4 (4A first, then any order):
  4A → 4B → 4C → 4D → 4E → 4F → 4G

Phase 5 (after respective Phase 3 slices):
  5A → 5B → 5C → 5D
```

Within each phase, slices after the infrastructure slice (xA) are independent and can be parallelized across branches or reordered based on priority.
