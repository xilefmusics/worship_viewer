# REST API — closing gaps (action plan)

Date: 2026-04-18  
Baseline: [rest-api-review.md](./rest-api-review.md) and follow-up work through PRs **#78–#81** (P0–P3).

This document turns the **remaining** findings in the review into an ordered, implementable backlog. Items are grouped by theme, with **goal**, **scope**, **concrete steps**, **acceptance criteria**, and **primary touchpoints**.

---

## 0. Keep the review doc honest

**Goal:** [rest-api-review.md](./rest-api-review.md) should not read like open P0/P1 bugs when those are fixed.

**Steps**

1. Add a short **“Status (2026-04)”** section at the top listing what #78–#81 resolved (IDOR, admin session scoping, Bearer-only auth, DELETE 204, pagination caps, Problem Details, etc.).
2. Replace or annotate stale **code citations** (e.g. S1 snippet) with “fixed in #78” or point to current handlers.
3. Refresh the **executive summary table (§1)** into “historical” vs “remaining” or move resolved rows to an appendix.

**Acceptance:** A new reader can tell in one screen what is still actionable vs already shipped.

---

## 1. SPA fallback must not swallow API-shaped paths

**Gap:** [rest-api-review.md §5.22](./rest-api-review.md) — unknown `GET` under the static scope can still return `index.html` for paths that look like API or auth URLs, depending on route registration order.

**Goal:** Any request whose path starts with `/api/` or `/auth/` and is not handled by a dedicated route returns a **JSON 404** (or the appropriate API error shape), never the SPA shell.

**Steps**

1. In `backend/src/frontend.rs`, inside the `default_handler` / `spa_fallback` closure, inspect `http_req.path()` (or equivalent).
2. If the path starts with `/api/` or `/auth/`, respond with `404` + `application/problem+json` (reuse `AppError::NotFound` or a small dedicated handler) instead of `index.html`.
3. Add integration tests: e.g. `GET /api/v1/definitely/not/a/route` and `GET /auth/not-a-real-endpoint` expect JSON problem body, not HTML.

**Acceptance:** No HTML `index.html` for those prefixes on unregistered paths; tests green.

**Touchpoints:** `backend/src/frontend.rs`, `backend/src/http_tests.rs` or YAML HTTP tests.

---

## 2. Harmonize OpenAPI response matrices (403 / 422 / PATCH / PUT)

**Gap:** [§4.3](./rest-api-review.md) — `PATCH`/`PUT` routes differ in documented status codes (`403`, `422`, `404`) across resources.

**Goal:** One predictable matrix per method class (aligned with [§11](./rest-api-review.md)), reflected in `utoipa::path` for every `/api/v1` route.

**Steps**

1. Produce a **spreadsheet or table**: each route × documented statuses (401, 403, 404, 409, 422, 500, …).
2. For each handler, compare **actual** `AppError` paths from the service layer with the OpenAPI block; add missing `(status = 403, …)` (and others) where the service can return them.
3. Decide project-wide whether **422** is used for semantic validation failures vs **400**; if 422 is reserved for future use, document “not used” in BLCs and keep 400 only — but be **consistent**.
4. Longer-term (optional in same epic): introduce a **`utoipa::Modify`** or small **macro** that injects common `(401, 403, 500)` + security for authenticated routes to avoid drift ([§5.12](./rest-api-review.md)).

**Acceptance:** Swagger UI shows no route that can return `Forbidden` in code but omits 403 in the spec; PUT/PATCH matrices match §11 intent.

**Touchpoints:** `backend/src/resources/**/rest.rs`, `backend/src/docs.rs`, relevant BLC markdown under `docs/business-logic-constraints/`.

---

## 3. PUT upsert: distinguish create vs update (201 vs 200)

**Gap:** [§5.6 / S7](./rest-api-review.md) — `PUT` can create; OpenAPI and clients still see **200** only.

**Goal:** When `update_song_for_user` (and any other upserting `PUT`) **creates** a resource, return **201 Created** with `Location` (or at least 201 without `Location` if Actix patterns make headers awkward); when updating, **200 OK**.

**Steps**

1. Audit all `PUT` handlers that delegate to “upsert” services (start with songs; repeat for collections/setlists/teams if applicable).
2. Change service APIs to return an enum or `(Resource, UpsertOutcome)` so the REST layer can pick status.
3. Update `utoipa::path` with `(status = 201, …)` and describe when it applies.
4. Update HTTP tests and CLI/client expectations.

**Acceptance:** BLC/tests that assert “PUT with new id creates” expect **201**; update paths expect **200**.

**Touchpoints:** `backend/src/resources/song/service.rs`, `song/rest.rs`, parallel resources, `backend/tests/*.yml`, `shared/src/net/*`, `cli/`, `frontend/`.

---

## 4. Reduce handler boilerplate: `UserPermissions` extractor

**Gap:** [§5.13](./rest-api-review.md) — every handler repeats `UserPermissions::new(&user, &svc.teams)`.

**Goal:** A single `FromRequest` implementation (e.g. `ReqData<UserPerms>` or tuple extractor) used across resources.

**Steps**

1. Define an extractor type holding `User` + `UserPermissions` (or `&User` + `UserPermissions` with appropriate lifetimes — Actix typically uses owned `User` from `ReqData`).
2. Implement `FromRequest` with the same error mapping as today (`401`/`403` as appropriate).
3. Migrate handlers incrementally (one resource module per PR) to avoid a mega-diff.

**Acceptance:** No new `UserPermissions::new` in migrated handlers; behavior unchanged in tests.

**Touchpoints:** `backend/src/resources/common.rs` or new `backend/src/extractors.rs`, all `**/rest.rs` under `resources/`.

---

## 5. Input validation at the edge

**Gap:** [§5.14](./rest-api-review.md) — DTOs lack max lengths, cardinality limits, and format checks until the database complains.

**Goal:** Predictable **400** with stable problem codes for common violations (length, empty name, too many IDs in `CreateSong.blobs`, etc.).

**Steps**

1. Choose **one** approach: `validator` crate + `Validate` on DTOs, or hand-written `fn validate(&self) -> Result<(), AppError>` on request types.
2. Start with high-risk fields called out in the review: team name (length already partially addressed), `CreateSong.blobs` max len, `UpdateTeam.members` max size, string trims.
3. Centralize limits in constants shared with OpenAPI `description` text where useful.
4. Add tests for oversize payloads and assert problem `code` / `detail` policy (human-readable but not leaky).

**Acceptance:** Oversize/malformed bodies fail in the handler layer with **400** and never hit Surreal for trivial cases.

**Touchpoints:** `shared/src/**` request DTOs, `backend/src/resources/**/rest.rs`, tests.

---

## 6. OpenAPI examples and discoverability

**Gap:** [§9](./rest-api-review.md) — no examples; clients must guess shapes.

**Goal:** At least **one request + one success response example** per major resource tag (`Songs`, `Collections`, …) in the generated OpenAPI.

**Steps**

1. Add `example` fields to key schemas in `backend/src/docs.rs` (or via `#[schema(example = …)]` on shared types).
2. Use `utoipa` examples on `request_body` and primary `responses` for POST/PATCH where it matters most.
3. Verify `openapi.json` validates in Swagger UI.

**Acceptance:** Swagger UI shows copy-pastable examples for create song, patch song, list query, and one auth flow.

**Touchpoints:** `backend/src/docs.rs`, selected `shared` types with `ToSchema`.

---

## 7. Conflict (`409`) and other messages — leak audit

**Gap:** [§5.10](./rest-api-review.md) focused on Surreal **400** mapping; **`409 Conflict`** paths may still embed raw `dberr.to_string()` in some branches.

**Goal:** Business conflicts return **safe, user-facing** `detail` strings; raw DB text only in logs.

**Steps**

1. Grep for `AppError::conflict`, `Conflict(`, and `409` paths; list all `detail` sources.
2. Replace raw DB strings with short constants + log the full error server-side.
3. Optionally add a dedicated problem `code` per conflict type (`duplicate_email`, `sole_admin`, …) if not already present.

**Acceptance:** No conflict response includes table names, index names, or query fragments.

**Touchpoints:** `backend/src/error.rs`, `From<surrealdb::Error>`, services returning `Conflict`.

---

## 8. Nested lists and pagination — polish

**Status:** `X-Total-Count` and nested pagination landed in #80; [§4.6](./rest-api-review.md) still mentions **no JSON envelope** (`total`, `next` links).

**Goal (choose one track):**

- **Track A (minimal):** Document in OpenAPI that list endpoints use **`X-Total-Count`** and describe paging algebra; add BLC cross-links.
- **Track B (richer):** Introduce a `Paginated<T> { items, total, page, page_size }` wrapper for selected list endpoints (breaking change — version or coordinate with clients).

**Steps for Track A:** Update descriptions on all list operations; ensure CLI/frontend read `X-Total-Count` consistently.

**Steps for Track B:** Design DTO in `shared`, migrate one resource, then roll out.

**Acceptance:** No ambiguity for frontend authors on how to detect last page.

**Touchpoints:** `shared/src/api/`, `backend/src/resources/**/rest.rs`, `frontend/`, `cli/`.

---

## 9. Downloads: `Content-Disposition` and export OpenAPI

**Status:** Blob download gained `Content-Disposition` in #81; the review’s **export** routes (`pdf`/`zip`/…) may not exist in the current tree or may live outside the paths previously cited.

**Goal:** Any byte-stream download (blob data, future exports) sets **`Content-Disposition: attachment`** where appropriate and documents **accurate** `content_type` in OpenAPI (not only `application/octet-stream` when the server sends `application/pdf`).

**Steps**

1. Inventory all handlers that return files or `Vec<u8>` bodies.
2. For each, set response headers consistently and mirror them in `utoipa` (multiple `content` blocks or documented primary type).
3. If export endpoints are **intentionally removed**, update BLCs and the review doc to say so.

**Acceptance:** Swagger + real responses agree on content type for each format variant.

**Touchpoints:** `backend/src/resources/blob/rest.rs`, any future export handlers, `docs.rs`.

---

## 10. OTP product semantics and documentation

**Gap:** [§5.9 / S6](./rest-api-review.md) — lockout/rate limits improved in #78; **self-signup via OTP verify** and enumeration nuances may still need explicit product decisions.

**Goal:** Behavior is **documented** in OpenAPI + user-facing docs: whether unknown emails can create users, whether OTP is login-only, etc.

**Steps**

1. Confirm intended product policy with stakeholders.
2. If self-signup is unwanted, gate `get_user_by_email_or_create` behind config or separate “register” flow.
3. Update `auth` OpenAPI descriptions and `docs/business-logic-constraints` for OTP.

**Acceptance:** No “surprise” account creation paths; security review can sign off.

**Touchpoints:** `backend/src/auth/otp/rest.rs`, user service, BLC auth/OTP docs.

---

## 11. Operational hardening (from §10)

**Goal:** Close non-HTTP but API-adjacent gaps called out in the review.

**Steps**

1. **Logger redaction:** Ensure `Logger` middleware does not print raw `Cookie` or `Authorization` ([§10](./rest-api-review.md)). Custom format or `tracing-actix-web` config.
2. **Static dir path:** Resolve `static_dir` to an absolute path at startup and log it once ([§10](./rest-api-review.md)).
3. **`initial_admin_user_test_session`:** Consider failing fast in “production-like” settings instead of only logging ([§10](./rest-api-review.md)).

**Acceptance:** Ops checklist ticked; no secrets in default access logs.

**Touchpoints:** `backend/src/main.rs`, `backend/src/settings.rs`, logging config.

---

## 12. Nice-to-have backlog (P3 remainder)

Schedule after the items above unless they unblock a consumer:

| Item | Ref | Notes |
|------|-----|--------|
| Structured filters beyond songs | §8 | Owner/tag/language patterns for other list endpoints |
| `q` on `/users` and `/blobs` | §4.4 | Search semantics + indexes |
| Resource ID canonical form in logs only | §4.7 | Already reject `table:id` at edge; ensure docs/BLC say “plain id only” |
| `default_collection` stale pointer | §5.24 | Heal on write or lazy repair in `create_song` |
| RFC 7807 `instance` URL | §5.11 | Populate `instance` with path + query |
| Rate-limit `429` in OpenAPI for all auth routes | §5.16 | Docs parity with middleware |
| CSRF posture | §6.7 | Document + optional custom header for cookie-authenticated writes |

---

## Suggested implementation order

1. **§0** (doc refresh) — cheap, avoids confusion.  
2. **§1** SPA guard — correctness + security hygiene.  
3. **§7** conflict leak audit — quick security win.  
4. **§2** OpenAPI matrices — improves every client.  
5. **§3** PUT 201 — behavioral + contract change; coordinate versioning.  
6. **§5** validation + **§6** examples — developer experience.  
7. **§4** extractor — refactor once contracts stable.  
8. **§8–§12** as product/ops priority dictates.

---

## Verification checklist (per PR)

- [ ] `cargo test` / HTTP integration suite for affected routes  
- [ ] Regenerate or snapshot `openapi.json` diff reviewed  
- [ ] Frontend + CLI updated if status codes or envelopes change  
- [ ] BLC markdown updated when behavior or error codes change  

This plan can be split into GitHub issues by section (one issue per numbered section, §3 and §5 possibly split by resource).
