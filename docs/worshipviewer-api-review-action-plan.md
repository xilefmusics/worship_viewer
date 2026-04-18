# Worship Viewer API Review — Action Plan

Date: 2026-04-18  
Baseline: [worshipviewer-api-review.md](./worshipviewer-api-review.md) (OpenAPI 3.0.3, API `1.0.0`).  
Related: [rest-api-review-closing-gaps-action-plan.md](./rest-api-review-closing-gaps-action-plan.md).

This document turns the review's findings into an ordered, implementable backlog. Items are grouped into three phases matching §5 of the review:

- **Phase 1 — Ship now** (spec-only, non-breaking): §§1–10
- **Phase 2 — Ship this quarter** (scoped, minor-breaking): §§11–18
- **Phase 3 — Schedule for v2** (breaking): §§19–23
- **Phase 4 — Cross-cutting hygiene and backlog**: §§24–27

Each item contains **gap**, **goal**, **concrete steps**, **acceptance criteria**, and **primary touchpoints**. Section numbers map 1:1 to the "Prioritized Action List" in the review; extra sections collect cross-cutting work that the review surfaces but does not number.

---

## Phase 1 — Ship now (spec-only, no behavioral changes)

The goal of Phase 1 is to make the OpenAPI document a faithful, consistent contract without forcing existing clients to change. All items should be landable behind spec-only PRs; where handler code must change (e.g. to emit the new content type) the change is strictly additive.

---

### 1. Unify the error model on `Problem` / `application/problem+json`

**Gap:** [§4.5](./worshipviewer-api-review.md#45-error-model) — two schemas (`ErrorResponse`, `ProblemDetails`) are used for the same error class across different routes, and neither is served with the RFC 7807 media type.

**Goal:** One canonical `Problem` schema referenced by **every** 4xx/5xx response, served as `application/problem+json`.

**Steps**

1. In `backend/src/error.rs` (and/or `shared/src/error/`), define a single `Problem` type matching the "After (single)" shape in [§6.2](./worshipviewer-api-review.md#62-error-envelope): `type`, `title`, `status`, `code`, optional `detail`, optional `instance`, `additionalProperties: true`.
2. Mark `ErrorResponse` and `ProblemDetails` as **deprecated aliases** of `Problem` in `utoipa` and add `#[deprecated]` on the Rust types; leave their serde shapes intact for one release.
3. Update `AppError::error_response` to emit `Content-Type: application/problem+json` on all 4xx/5xx paths. Keep the JSON body byte-compatible with `Problem`.
4. In every `#[utoipa::path(... responses(...))]` block under `backend/src/resources/**/rest.rs` and `backend/src/auth/**/rest.rs`, replace the `body = ErrorResponse` / `body = ProblemDetails` references with `body = Problem` and set `content_type = "application/problem+json"`.
5. Publish a short "Error model" section in the OpenAPI `info.description` listing the stable `code` values. Source the list from a single enum in `shared/src/error/codes.rs` and assert (unit test) that the enum and the docs stay in sync.
6. Run the OpenAPI diff tool (or `cargo test -p backend -- openapi_snapshot`) and review the diff before merging.

**Acceptance**

- Every 4xx/5xx response in `openapi.json` uses `application/problem+json` with `$ref: #/components/schemas/Problem`.
- `ErrorResponse` and `ProblemDetails` are still resolvable (for one release) but marked `deprecated: true`.
- A documented enum of `code` values ships in the intro.

**Touchpoints:** `backend/src/error.rs`, `backend/src/docs.rs`, `backend/src/resources/**/rest.rs`, `shared/src/error/`, integration tests that assert response content-type.

---

### 2. Fix auth parameter locations (`in: path` → `in: query`)

**Gap:** [§4.15](./worshipviewer-api-review.md#415-auth-endpoints) — `/auth/callback` declares `code`, `state` and `/auth/login` declares `redirect_to`, `provider` as `in: path`, which cannot be correct given the route shape.

**Goal:** The OpenAPI spec accurately reflects the runtime routing: all four parameters are `in: query`.

**Steps**

1. Open `backend/src/auth/**/rest.rs` and locate the `utoipa::path` macros for `/auth/callback` and `/auth/login`.
2. Change `(path = ...)` parameter annotations to `(query = ...)` / `ParameterIn::Query`. Ensure each parameter carries `required = true/false` matching the handler signature.
3. Add an `enum` to `provider` listing the real providers (e.g. `google`, `microsoft`, `apple`) and keep the Rust type in sync.
4. Document `redirect_to` validation in prose: "must match an allow-listed origin/path; otherwise the server returns 400". Link to `backend/src/auth/redirect.rs` (or equivalent) validation logic.
5. Add an HTTP/YAML test that fetches `/api/docs/openapi.json` and asserts the four parameters resolve to `in: query`.

**Acceptance**

- `openapi.json` shows `in: query` for `code`, `state`, `redirect_to`, `provider`.
- `provider` has a concrete enum.
- A regression test fails if this drifts.

**Touchpoints:** `backend/src/auth/callback.rs`, `backend/src/auth/login.rs`, `backend/src/docs.rs`, `backend/tests/auth_*.yml`.

---

### 3. Correct `page_size` bounds (`min: 1`, `max: 500`, `default: 50`)

**Gap:** [§4.8](./worshipviewer-api-review.md#48-pagination) — schema says `minimum: 0` but prose says `1–500`; schema lacks `maximum` and `default`.

**Goal:** The `page_size` parameter schema matches its documented semantics.

**Steps**

1. Locate the shared query type (likely `shared/src/net/pagination.rs` or `shared/src/validation_limits.rs`).
2. Update the `utoipa::IntoParams` / `ToSchema` annotations: `minimum = 1`, `maximum = 500`, `default = 50`, `nullable = true`.
3. Update the parameter `description` to match: *"Items per page. Must be 1–500. Defaults to 50."*
4. In the handler layer, reject `page_size = 0` explicitly with `400` + `code = invalid_page_size` (should already happen in validation, but assert test coverage).
5. Update [list-pagination BLC](./business-logic-constraints/list-pagination.md) to reflect the canonical bounds.

**Acceptance**

- Every list operation in `openapi.json` reports the corrected bounds on `page_size`.
- A regression test asserts `page_size=0` → `400`, `page_size=501` → `400`, and omitted → server uses `50`.

**Touchpoints:** `shared/src/net/pagination.rs`, `shared/src/validation_limits.rs`, list handlers in `backend/src/resources/**/rest.rs`, `docs/business-logic-constraints/list-pagination.md`.

---

### 4. Correct SVG MIME type (`image/svg` → `image/svg+xml`)

**Gap:** [§4.14](./worshipviewer-api-review.md#414-binary--blob-handling) — `image/svg` is not a registered media type.

**Goal:** Blob responses and the `FileType` hint use the correct media type for SVG.

**Steps**

1. In `shared/src/blob/mod.rs` (or wherever `FileType` lives), update the SVG variant's serialization helper to emit `image/svg+xml` on responses.
2. In `backend/src/resources/blob/rest.rs`, ensure the `GET /blobs/{id}/data` response sets `Content-Type: image/svg+xml` for SVG blobs.
3. Update `utoipa::path` response content tables to list `image/png`, `image/jpeg`, `image/svg+xml`.
4. Accept both `image/svg` and `image/svg+xml` on uploads for one release with a deprecation log line.
5. Update HTTP test fixtures and any frontend/CLI code that hardcodes `image/svg`.

**Acceptance**

- SVG downloads are served with `image/svg+xml`.
- No `image/svg` string remains in the OpenAPI document.
- SVG uploads still succeed while the legacy value is accepted; a log warns on legacy use.

**Touchpoints:** `shared/src/blob/`, `backend/src/resources/blob/rest.rs`, `backend/src/docs.rs`, `frontend/src/**`, `cli/src/**`.

---

### 5. Drop `required: ["data"]` on `PatchSong` (and audit siblings)

**Gap:** [§4.10](./worshipviewer-api-review.md#410-patch--partial-update-semantics) — a partial update should not force `data` to be present.

**Goal:** All `Patch<X>` input shapes honor the "fields absent = unchanged" rule at the top level.

**Steps**

1. In `shared/src/song/patch.rs` (or equivalent), remove `required = "data"` from the `PatchSong` schema and make the field `Option<PatchSongData>`.
2. Audit every other `Patch*` type (`PatchCollection`, `PatchSetlist`, `PatchTeam`, `PatchUser`, …) for the same anti-pattern; fix in the same PR.
3. Extend handler tests to send `{ "not_a_song": true }` without `data` and assert the body is accepted, `data` is untouched, and the response reflects the change.
4. Update [song BLC](./business-logic-constraints/song.md) and the API prose section on PATCH.

**Acceptance**

- No `Patch*` schema has top-level `required` fields.
- Patching a single scalar without nesting succeeds on every resource.

**Touchpoints:** `shared/src/*/patch.rs`, `backend/src/resources/**/rest.rs`, `backend/tests/*.yml`.

---

### 6. Close missing-response-code gaps (`404`, `403`, `412`, `409`)

**Gap:** [§4.6](./worshipviewer-api-review.md#46-http-status-coverage) — several routes omit statuses the code can actually return (notably `404` on `PUT /setlists/{id}`, `403` on cross-user mutations).

**Goal:** The OpenAPI response matrix reflects every non-5xx status the handler can produce.

**Steps**

1. Build a **spreadsheet** of every `/api/v1/*` route × (`400`, `401`, `403`, `404`, `409`, `412`, `422`, `429`). This is similar to [§2 of the prior action plan](./rest-api-review-closing-gaps-action-plan.md).
2. For each handler, trace `AppError` variants out of the service layer and annotate the spreadsheet with which ones can reach the REST layer.
3. Update the `responses(...)` blocks in `utoipa::path` macros to add missing statuses, all pointing at the unified `Problem` schema from §1.
4. Resolve the `400` vs `422` question project-wide (see Phase 2 §17): either use `422` for semantic validation failures or document explicitly that the API uses `400` for both. Record the decision in `docs/business-logic-constraints/http-contract.md`.
5. Consider extracting a small `utoipa::Modify` or macro (`authenticated_responses!()`, `mutating_responses!()`) so the matrix cannot drift again.
6. Add Spectral rules (see §25) that flag mutating operations missing `403`/`404`/`412`.

**Acceptance**

- No handler path can emit an `AppError` variant absent from the OpenAPI block.
- The spreadsheet is checked in under `docs/` or replaced by a generated report.

**Touchpoints:** every `backend/src/resources/**/rest.rs`, `backend/src/docs.rs`, `docs/business-logic-constraints/http-contract.md`.

---

### 7. Rename `CreateUserRequest` → `CreateUser`

**Gap:** [§4.11](./worshipviewer-api-review.md#411-put--create--update-shapes) — the `Request` suffix is naming drift; every other `Create<X>` type omits it.

**Goal:** All create DTOs follow the `Create<Resource>` naming scheme.

**Steps**

1. Rename the Rust type in `shared/src/user/create.rs` from `CreateUserRequest` to `CreateUser`.
2. Use `#[schema(as = CreateUserRequest)]` or `#[serde(rename = "CreateUserRequest")]` on the Rust type for one release to preserve the public OpenAPI component name. Alternatively, bump the component name immediately and note it in the changelog.
3. Update all imports across `backend/`, `cli/`, `frontend/`.
4. Regenerate `openapi.json` and diff it to confirm the intended shape.

**Acceptance**

- No `CreateUserRequest` identifier in the Rust tree (except the serde-compat attribute).
- OpenAPI either retains `CreateUserRequest` for one release (with a deprecation note) or fully renames in a coordinated PR.

**Touchpoints:** `shared/src/user/`, `backend/src/resources/user/`, `cli/`, `frontend/`.

---

### 8. Document under-specified domain fields

**Gap:** [§4.18](./worshipviewer-api-review.md#418-domain-fields-without-documentation) — `Song.not_a_song`, `Song.data`, `Song.user_specific_addons`, `Collection.cover`, `Player.scroll_type_cache_other_orientation`, `Player.between_items`, `Blob.ocr`, `TocItem.nr`, `TocItem.id`, `User.request_count`, `SongLink.key`, units for `tempo`, format for `languages` — all under-documented.

**Goal:** Every field in the public contract has a one-line description explaining purpose, units, nullability intent, and BCP/ISO references where relevant.

**Steps**

1. Walk each field listed above in `shared/src/**` and add `#[schema(description = "...")]` on the corresponding `ToSchema` derive. Keep descriptions short enough to render well in Swagger UI.
2. For `tempo`: description `"Tempo in BPM (beats per minute)."`.
3. For `languages`: description `"BCP 47 language tags (e.g. `en`, `de-CH`)."` and add a `pattern` regex approximating BCP 47.
4. For `SongLink.key`: description defining the grammar (note `A`–`G`, optional accidental `#`/`b`, optional mode `m`/`maj`/`min`). Add `pattern` if the grammar is strict enough.
5. For `Song.data`: schedule the structural promotion to `SongData` under Phase 2 §11; this item only adds a description referencing that work.
6. For `Player.scroll_type_cache_other_orientation` and `Player.between_items`: either add a product-facing description or log a decision to deprecate (tracked under §22).
7. Update BLCs (`docs/business-logic-constraints/song.md`, `player.md`, etc.) with the same text so the docs and the spec agree.

**Acceptance**

- Spectral rule `oas3-parameter-description` (or equivalent) passes across the spec.
- No field in the "Domain fields without documentation" list lacks a description.

**Touchpoints:** every `shared/src/**` `ToSchema` type listed above, relevant BLCs.

---

### 9. Remove `q` from sub-resources that don't implement search

**Gap:** [§4.8](./worshipviewer-api-review.md#48-pagination) — several list endpoints declare `q` as "Reserved; not used", adding noise and tempting silent semantics in the future.

**Goal:** A parameter appears in the spec only where it has defined semantics today.

**Steps**

1. Grep for `"Reserved"` and `q:` across `backend/src/resources/**/rest.rs`.
2. Delete the parameter from every handler that doesn't actually filter on it.
3. If search is planned for a given resource, file a GitHub issue and keep the parameter out of the spec until that issue ships.
4. Update BLCs that mention reserved parameters.

**Acceptance**

- `rg '"Reserved"' backend/src/ shared/src/` returns nothing.
- No list operation in `openapi.json` declares an unused parameter.

**Touchpoints:** `backend/src/resources/**/rest.rs`.

---

### 10. Add timezone, ID format, and identifier guidance to the intro

**Gap:** [§4.17](./worshipviewer-api-review.md#417-dates-numbers-constraints) and [§4.19](./worshipviewer-api-review.md#419-openapi-hygiene) — timezone policy, ID format, and general conventions are not stated in `info.description`.

**Goal:** A newcomer reading only the OpenAPI intro can answer: what timezone is used, what is an ID, what are the casing rules, where are errors documented, how does pagination work.

**Steps**

1. Expand `info.description` (in `backend/src/docs.rs`) to include, in order:
   - **Timestamps**: "All timestamps are UTC and rendered with the `Z` suffix (RFC 3339)."
   - **Identifiers**: "Resource IDs are opaque printable strings of form `…`; treat them as opaque."
   - **Casing**: "JSON object keys use `snake_case`. Enum values use `lower_snake_case` (in progress, see §4.4)."
   - **Pagination**: one paragraph pointing at `page`, `page_size`, `X-Total-Count`, and (after §14) `Link` headers.
   - **Errors**: one paragraph pointing at `application/problem+json` + a link to the stable `code` enum.
   - **Versioning**: `/api/v1` vs unversioned `/auth/*` (already noted; keep).
2. Add `info.contact` and `info.termsOfService` if product is ready for them.
3. If staging and production are both real, add two entries under `servers:`.

**Acceptance**

- Swagger UI landing page covers the five bullets above in prose.
- `servers` reflects real deployments.

**Touchpoints:** `backend/src/docs.rs`, `README.md` if it cross-links.

---

## Phase 2 — Ship this quarter (scoped, minor-breaking)

Phase 2 items change observable behavior or require client coordination. Each should ship with a changelog entry and, where noted, a deprecation window.

---

### 11. Expose `SongData` as a real schema

**Gap:** [§4.18](./worshipviewer-api-review.md#418-domain-fields-without-documentation) — `Song.data` is declared as untyped `object`, forcing clients to model it as `unknown` while `PatchSongData` already carries rich structure.

**Goal:** `Song.data` is a named `SongData` schema; `PatchSongData` describes partial updates of it; both are internally consistent.

**Steps**

1. Factor the fields currently inside `PatchSongData` into a concrete `SongData` struct in `shared/src/song/data.rs`. Apply the three-state semantics already documented on `PatchSongData` only to the patch variant.
2. Attach the new `SongData` type to `Song.data` and update `utoipa::ToSchema` derives.
3. Add `#[schema(example = ...)]` for each field so Swagger UI renders meaningful sample payloads.
4. Audit callers (`cli/`, `frontend/`) that previously treated `data` as a generic map; migrate to the typed form.
5. Add a regression test that round-trips a full song through `POST` → `GET` → `PATCH` without losing fields.

**Acceptance**

- `Song.data` in `openapi.json` references `#/components/schemas/SongData`.
- `frontend/` and `cli/` consume the typed form.
- Round-trip test green.

**Touchpoints:** `shared/src/song/`, `backend/src/resources/song/`, `frontend/`, `cli/`.

---

### 12. Standardize enum casing on `lower_snake_case`

**Gap:** [§4.4](./worshipviewer-api-review.md#44-enums--value-casing) — four different conventions across `FileType`, `Orientation`, `ScrollType`, `Role`, `TeamRole`.

**Goal:** Every enum value in the API is `lower_snake_case` on output; legacy values are accepted on input during one release.

**Steps**

1. For each enum, add `#[serde(rename_all = "snake_case")]` (or explicit `rename = "..."` on variants where the mapping is non-trivial: e.g. `TwoHalfPage` → `two_half_page`).
2. Implement a tolerant deserializer: a custom `Visitor` (or a helper macro) that accepts both the old and new representations, records a counter (`metrics!("legacy_enum_value", enum = "FileType", value = "PNG")`), and maps to the new variant.
3. Emit a log line at `warn!` level on the first legacy value seen per process, then at `debug!` afterwards.
4. Update `utoipa::ToSchema` to advertise only the new values.
5. Evaluate whether `FileType` should be replaced by actual MIME strings (`image/png`, `image/jpeg`, `image/svg+xml`) per [§4.4](./worshipviewer-api-review.md#44-enums--value-casing). If yes, fold that into this epic; if no, record the decision.
6. Coordinate with `frontend/` and `cli/` to send new values only.
7. Plan the legacy-input sunset date (≥ 1 release after this change).

**Acceptance**

- No enum in `openapi.json` has mixed casing.
- Metric/log coverage exists for legacy inputs.
- Frontend/CLI send only the canonical values.

**Touchpoints:** `shared/src/**` enums, `backend/src/**` deserialization, `frontend/`, `cli/`, `docs/business-logic-constraints/*.md`.

---

### 13. `If-Match` / `412 Precondition Failed` on mutating endpoints

**Gap:** [§4.7](./worshipviewer-api-review.md#47-conditional-requests--concurrency) — ETag is read-only; no write-side optimistic concurrency.

**Goal:** Every `PUT`, `PATCH`, `DELETE` on an ETag-bearing resource supports `If-Match`; mismatches return `412`.

**Steps**

1. Extract a reusable Actix extractor `IfMatch(Option<String>)` in `backend/src/http_cache.rs` (or a new `if_match.rs`).
2. In each mutating service, compare the ETag (already computed server-side) with the header and return `AppError::PreconditionFailed` on mismatch; document the new `code = precondition_failed`.
3. Add `(status = 412, ...)` to every relevant `utoipa::path` block.
4. Decide and document **weak vs strong** ETag per resource. Prefer weak (`W/"…"`) for mutable resources; be consistent across the API.
5. Optional: add `428 Precondition Required` behind a config flag for strict mode.
6. Update BLCs (`http-contract.md`, per-resource BLCs) and add integration tests: happy path, mismatching ETag, missing ETag (under strict mode).

**Acceptance**

- Concurrent updates where the second client sent a stale `If-Match` return `412`.
- All ETag-returning reads have a matching `If-Match` path on their writers.

**Touchpoints:** `backend/src/http_cache.rs`, all mutating handlers under `backend/src/resources/**/rest.rs`, `docs/business-logic-constraints/http-contract.md`, integration tests.

---

### 14. Add `Link` pagination headers (RFC 5988)

**Gap:** [§4.8](./worshipviewer-api-review.md#48-pagination) — clients must hand-roll page URLs; no `next`/`prev`/`first`/`last`.

**Goal:** Every list endpoint emits `Link` headers with `rel` values `first`, `prev`, `next`, `last` where applicable, in addition to the existing `X-Total-Count`.

**Steps**

1. Add a `pagination::link_header(req, page, page_size, total)` helper in `shared/src/net/pagination.rs` or `backend/src/resources/common.rs`.
2. Apply it in every list handler; wrap `HttpResponse::Ok().insert_header(("Link", …))`.
3. Document the header in `info.description` and in [list-pagination BLC](./business-logic-constraints/list-pagination.md).
4. Add integration tests asserting that a middle page returns all four rel values, and edge pages omit `prev`/`next` correctly.
5. Update frontend/CLI to consume `Link` where it simplifies logic.

**Acceptance**

- `Link` header present on every list response.
- `X-Total-Count` retained.
- Tests cover first/middle/last page edge cases.

**Touchpoints:** `shared/src/net/pagination.rs`, `backend/src/resources/**/rest.rs`, `docs/business-logic-constraints/list-pagination.md`, `frontend/`, `cli/`.

---

### 15. Single sort syntax across the API

**Gap:** [§4.9](./worshipviewer-api-review.md#49-sorting--filtering) — `/songs` uses custom tokens (`id_desc`, `title_asc`, `relevance`); nothing else is sortable.

**Goal:** One sort grammar — recommended `sort=-id,title` (JSON:API-flavored `-` prefix) — available on every sortable list endpoint.

**Steps**

1. Design the grammar in `shared/src/net/sort.rs`:
   - Comma-separated fields; optional `-` prefix for descending.
   - Reserved token `relevance` (for search-bearing endpoints).
2. Build a parser + validator with a per-resource allowlist of sortable fields.
3. Migrate `/songs` to the new grammar; accept the legacy tokens (`id_desc`, …) for one release with a deprecation warning.
4. Extend sorting to `collections`, `setlists`, `teams`, `users` where the product supports it.
5. Update the OpenAPI parameter descriptions and add examples.
6. Update `docs/business-logic-constraints/*.md` for each resource that gains sorting.

**Acceptance**

- Every sortable list accepts the same grammar.
- Legacy tokens still work on `/songs` with a log warning, and are scheduled for removal.

**Touchpoints:** `shared/src/net/sort.rs` (new), `backend/src/resources/**/rest.rs`, `frontend/`, `cli/`.

---

### 16. Symmetric invitation acceptance

**Gap:** [§4.1](./worshipviewer-api-review.md#41-resource--url-design) — `.../accept` lives at `/api/v1/invitations/{invitation_id}/accept`, while invitations otherwise hang off `/teams/{team_id}/invitations/...`.

**Goal:** `POST /api/v1/teams/{team_id}/invitations/{invitation_id}/accept` exists; the old path is deprecated.

**Steps**

1. Add the new nested route in `backend/src/resources/team/invitation/rest.rs`, reusing the existing service method.
2. Keep the old top-level route handler for one release, returning the same response and logging a deprecation metric.
3. Add `Sunset` and `Deprecation` response headers on the old path (per RFC 8594 / draft-deprecation-header).
4. Update `frontend/`, `cli/`, and BLC `team-invitation.md` to use the new path.
5. Optional: introduce `GET /api/v1/invitations/{invitation_id}` (read-only) so a client with only the invitation ID can discover `team_id` before accepting.

**Acceptance**

- Both routes work for one release; only the new route is documented as the primary.
- Clients migrated.

**Touchpoints:** `backend/src/resources/team/invitation/rest.rs`, `frontend/`, `cli/`, `docs/business-logic-constraints/team-invitation.md`.

---

### 17. Rate-limit documentation on `/api/v1/*`

**Gap:** [§4.16](./worshipviewer-api-review.md#416-security-csrf-rate-limits) — `429` is documented on `/auth/*` but nowhere else.

**Goal:** The real rate-limit behavior (which certainly exists at the reverse-proxy / middleware layer) is visible to API consumers.

**Steps**

1. Inventory existing rate-limit middlewares (`backend/src/main.rs`, any `governor`/`tower-governor`/custom layers).
2. For every `/api/v1/*` operation, add `(status = 429, body = Problem, headers = ("Retry-After", "X-RateLimit-Limit", "X-RateLimit-Remaining", "X-RateLimit-Reset"))` to the `utoipa::path` block.
3. Emit those headers from the middleware for real (if not already).
4. Document the numerical bucket sizes in `docs/business-logic-constraints/http-contract.md`.

**Acceptance**

- Spec and runtime agree on `429` + `Retry-After` + `X-RateLimit-*` headers.

**Touchpoints:** `backend/src/main.rs`, rate-limit middleware, `backend/src/resources/**/rest.rs`, `docs/business-logic-constraints/http-contract.md`.

---

### 18. `ETag` and `Cache-Control` on blob byte responses

**Gap:** [§4.14](./worshipviewer-api-review.md#414-binary--blob-handling) — `GET /blobs/{id}/data` has no cache metadata declared.

**Goal:** Binary blob responses are cacheable by browsers and CDNs, with correct privacy semantics.

**Steps**

1. In `backend/src/resources/blob/rest.rs`, compute an ETag for the bytes (hash of stored content or a generation counter) and set `ETag: W/"…"`.
2. Set `Cache-Control: private, max-age=3600` (or a product-agreed value). Document why `private` (user-scoped access).
3. Honor `If-None-Match` → `304 Not Modified` (reuse the existing singleton pattern).
4. Add `Content-Length` to every response.
5. Optional (schedule if infra allows): support `Content-Digest: sha-256=:…:` on uploads; reject mismatches with `400` `code = digest_mismatch`.
6. Update `openapi.json` `responses` headers blocks and BLC `blob.md`.

**Acceptance**

- A second request for the same blob yields `304` when `If-None-Match` matches.
- Browsers cache blobs; a test with a mock fetch validates the `Cache-Control` string.

**Touchpoints:** `backend/src/resources/blob/rest.rs`, `backend/src/http_cache.rs`, `docs/business-logic-constraints/blob.md`.

---

## Phase 3 — Schedule for v2 (breaking)

These require coordinated client changes. Gate each behind a `/api/v2` path (or a feature flag) and run both shapes during the transition.

---

### 19. Redesign `PlayerItem` with an internally-tagged discriminator

**Gap:** [§4.12](./worshipviewer-api-review.md#412-polymorphism-playeritem) — externally tagged Rust-serde shape with PascalCase keys and no `discriminator`.

**Goal:** `PlayerItem` uses `{ "type": "blob" | "chords", … }` with a proper `discriminator`; keys are `snake_case`; `blob_id` is descriptive.

**Steps**

1. Introduce a v2 serde representation behind a `#[cfg(feature = "apiv2")]` or versioned module: `PlayerBlobItem { type: "blob", blob_id }` and `PlayerChordsItem { type: "chords", song }` matching [§6.3 of the review](./worshipviewer-api-review.md#63-playeritem).
2. Add `#[serde(tag = "type")]` on the Rust enum; register the discriminator in `utoipa`.
3. During transition, serve both shapes: v1 path emits the old shape; v2 path emits the new. Internally deserialize both so clients can migrate at their own pace.
4. Write a migration guide in `docs/` describing the rename and the discriminator mapping.
5. Schedule the v1 removal window.

**Acceptance**

- v2 `PlayerItem` JSON uses the discriminator and snake_case.
- `openapi.json` for v2 exposes `discriminator.mapping`.
- Both versions pass integration tests for one release.

**Touchpoints:** `shared/src/player/`, `backend/src/resources/setlist/player/`, `frontend/`, `cli/`.

---

### 20. Unify cross-resource references (`{ id, … }` links + `?expand=`)

**Gap:** [§4.2](./worshipviewer-api-review.md#42-identifier--reference-style-the-blobs-question) and [§4.13](./worshipviewer-api-review.md#413-reference-vs-expansion-strategy) — four different reference shapes across the API.

**Goal:** Every cross-resource reference is a `{ id, …link-metadata }` object; full embedding is opt-in via `?expand=...`.

**Steps**

1. Pick the **always-reference** policy (per [§4.13](./worshipviewer-api-review.md#413-reference-vs-expansion-strategy)) and publish it in the API intro.
2. Introduce a `BlobLink = { id, … }` schema in `shared/src/blob/`. Populate it alongside the existing `blobs: string[]` (marked `deprecated`) on `Song`, matching [§6.1 of the review](./worshipviewer-api-review.md#61-songblobs--songblob_links). This can actually start in Phase 1 as an additive v1 change; promote to required in v2.
3. Convert full embeds (`Session.user`, `Team.owner`, `Team.members[].user`, `TeamInvitation.created_by`) to references with `?expand=` to opt back into full objects. Implement an `Expand` extractor that parses `?expand=owner,members.user` and drives the serializer.
4. Rename `PlayerItem.Blob` string → `PlayerBlobItem.blob_id` (covered by §19).
5. Provide a client migration cookbook enumerating every changed path.
6. Deprecate and remove the old shapes in v2.

**Acceptance**

- Every `components.schemas` relation uses the `{ id, … }` link shape.
- `?expand=...` produces the embedded variants for documented relations.

**Touchpoints:** `shared/src/**`, `backend/src/resources/**`, `frontend/`, `cli/`, migration cookbook under `docs/`.

---

### 21. Drop the `error` legacy alias from `Problem`

**Gap:** [§4.5](./worshipviewer-api-review.md#45-error-model) — `error` is already marked "legacy alias".

**Goal:** The v2 `Problem` schema no longer includes `error`.

**Steps**

1. Remove `error` from the Rust `Problem` type (or gate it under the v1 module only).
2. Scan log/observability consumers (frontend/cli, dashboards, alerting) for references and migrate to `code`/`detail`.
3. Announce the removal in the v2 migration guide alongside §19/§20.

**Acceptance**

- `rg '"error":' openapi.v2.json` returns no field matches.
- No client reads `error` after v2 cut-over.

**Touchpoints:** `shared/src/error/`, clients.

---

### 22. Drop PascalCase keys (and any other non-snake_case key)

**Gap:** [§4.3](./worshipviewer-api-review.md#43-naming-conventions) — `PlayerItem` currently carries `Blob`/`Chords` keys.

**Goal:** 100% snake_case JSON keys, enforced in CI.

**Steps**

1. Finish §19 — this removes the last known violator.
2. Add a Spectral rule (or a small custom script) that scans `openapi.json` for any `properties` key that doesn't match `^[a-z][a-z0-9_]*$` and fails CI.
3. Audit the spec manually for less obvious violators (e.g. nested anyOf shapes, webhooks).

**Acceptance**

- CI rule enforces the invariant.
- No property key in `openapi.json` uses PascalCase or camelCase.

**Touchpoints:** `.spectral.yaml` (see §25), `shared/src/**`.

---

### 23. Consolidate create/update types

**Gap:** [§4.11](./worshipviewer-api-review.md#411-put--create--update-shapes) — `Create<X>` is reused for both POST and PUT; `User` has naming drift; `Team` uses three separate shapes.

**Goal:** A consistent, justified policy for create vs update DTOs across resources.

**Steps**

1. Write a short policy doc: "When do we use a separate `UpdateX` type?"  Suggested rule: if the fields settable on PUT differ from the fields required on POST (e.g. server-owned IDs, immutable fields), use `CreateX` + `UpdateX`; otherwise reuse `CreateX`.
2. Apply it: rename `CreateUserRequest` → `CreateUser` (Phase 1 §7 started this); consider splitting `Create*` into `Create*` + `Update*` where PUT semantics differ from POST.
3. Re-evaluate `Team`: do `CreateTeam`, `UpdateTeam`, `PatchTeam` all pull their weight? If not, merge.
4. Document upsert behavior on every PUT endpoint or explicitly deny it and return `404`.

**Acceptance**

- Policy doc checked in.
- Rust types follow the policy across `shared/src/**`.
- Every `PUT` handler has documented upsert behavior.

**Touchpoints:** `shared/src/**`, `backend/src/resources/**/rest.rs`, `docs/business-logic-constraints/http-contract.md`.

---

## Phase 4 — Cross-cutting hygiene and backlog

Work that the review mentions but that doesn't cleanly map to a single prioritized list item.

---

### 24. OpenAPI hygiene pass

**Gap:** [§4.19](./worshipviewer-api-review.md#419-openapi-hygiene) — missing `info.contact`, no `examples`, thin operation descriptions.

**Goal:** The spec is polished enough to host as the primary API reference.

**Steps**

1. Fill `info.contact`, `info.termsOfService`, and `info.license`.
2. Add at least one `example` (request and success response) per resource tag in `backend/src/docs.rs` or via `#[schema(example = ...)]` — complements the earlier action plan's §6.
3. Replace any "Creates a new user"-style thin descriptions with a sentence on side effects, emitted events, and idempotency.
4. Add `externalDocs` per tag pointing at BLCs.

**Acceptance**

- `info.contact` filled.
- Every major resource has at least one example.
- Operations have non-trivial descriptions.

**Touchpoints:** `backend/src/docs.rs`, `shared/src/**` `ToSchema`.

---

### 25. Spectral (OpenAPI) linting in CI

**Gap:** [§4.19](./worshipviewer-api-review.md#419-openapi-hygiene) — no linter guards the spec.

**Goal:** Every CI run rejects spec regressions for casing, missing responses, missing descriptions, duplicate schemas, etc.

**Steps**

1. Add `.spectral.yaml` at repo root extending `spectral:oas` plus custom rules for:
   - `operation-tag-defined`, `operation-operationId`, `operation-description`.
   - Casing rule: all `properties` keys match `^[a-z][a-z0-9_]*$`.
   - Response matrix rule: every mutating operation declares `400` (or `422`), `401`, `403`, `404`, `409`, `412`, `429`.
   - Error content-type rule: every 4xx/5xx uses `application/problem+json`.
2. Add a CI job (GitHub Actions) that runs `npx @stoplight/spectral-cli lint backend/openapi.json`.
3. Snapshot-test the diff between `backend/src/docs.rs`-produced JSON and a committed `openapi.json`.

**Acceptance**

- CI fails when any rule above is violated.
- The current spec passes (or the PR includes targeted fixes/waivers).

**Touchpoints:** `.spectral.yaml` (new), `.github/workflows/*.yml`, `openapi.json` snapshot.

---

### 26. Missing / asymmetric operations

**Gap:** [§4.20](./worshipviewer-api-review.md#420-missing--asymmetric-operations) — decline-invitation, liked-songs list, per-member team operations, bulk endpoints, export, search.

**Goal:** Product-led decisions on which of these to build; filed issues for the rest.

**Steps**

1. Open a GitHub issue per item:
   - `POST /teams/{team_id}/invitations/{invitation_id}/decline` (matches §16).
   - `GET /users/me/liked-songs` (pagination + sort; mind [§6.1](./worshipviewer-api-review.md#61-songblobs--songblob_links)).
   - `POST /teams/{team_id}/members` and `DELETE /teams/{team_id}/members/{user_id}`.
   - `POST /songs/batch` (if product agrees).
   - `GET /users/me/export` (GDPR-style).
   - Search under `/songs?q=...` promoted out of "Reserved" in Phase 1 §9.
2. For each issue, capture: user story, proposed contract, HTTP semantics, test plan.
3. Prioritize against roadmap; don't let them ship piecemeal without consistent response shapes.

**Acceptance**

- Issues exist and are triaged.

**Touchpoints:** GitHub issues; eventually `backend/src/resources/**`.

---

### 27. Backlog and nice-to-haves

Schedule after the above unless they unblock a consumer:

| Item | Ref | Notes |
|------|-----|-------|
| Cursor pagination on `/songs` and nested song lists | §4.8 | Required if client-visible row churn or large exports become routine |
| `Set-Cookie` modeled in `/auth/otp/verify` response headers | §4.15 | Makes SDKs aware of the session cookie |
| `CsrfToken` security scheme + per-operation requirement | §4.16 | Pairs with the cookie-auth story |
| `Content-Digest: sha-256=:…:` on blob uploads | §4.14 | Optional; reject on mismatch with `400` |
| `Range` / `206 Partial Content` on blob data | §4.14 | Only if large assets (PDF, audio) ever land |
| `format: uuid` (or similar) on all ID fields | §4.17 | Pick one and apply repo-wide |
| Tag-filter rephrasing to domain terms | §4.9 | Decouples API from internal `data.tags` representation |
| `maxLength` / `maxItems` on free-text and arrays | §4.17 | DoS + validation; align with `shared/src/validation_limits.rs` |
| Consolidate `/users/me/sessions` vs `/users/{user_id}/sessions` | §4.1 | Decide whether admin-impersonation is a distinct verb (`impersonation-sessions`) |
| Deprecate `Player.scroll_type_cache_other_orientation` | §4.18 | Leaks internal cache concept; compute server-side or drop |
| `User.request_count` review | §4.18 | Confirm that a usage counter belongs on the public user object |

---

## Suggested implementation order

Phase 1 is all independent and can be parallelized across PRs. A sensible serial ordering for a single contributor:

1. **§1** (error model) — unblocks every subsequent response-shape change.
2. **§3**, **§4**, **§5**, **§7**, **§9** — tiny, mechanical, each a one-PR win.
3. **§2** (auth params) — tests easy, fixes a real client footgun.
4. **§6** (response matrix) and **§8** (field descriptions) — spreadsheet-driven, best as two contained efforts.
5. **§10** (intro prose) — last so it can reference the work above.
6. **Phase 2** — start with **§13** (`If-Match`) because it's pure-add, then **§11** (`SongData`), **§14**–**§15** (pagination/sort), **§18** (blob caching), **§16** (invitations), **§12** (enum casing, deprecation-heavy), **§17** (rate-limit docs).
7. **Phase 3** — plan as a **single `v2` epic**. Do **§19** + **§20** together (both change client shape), then **§21**–**§23** alongside.
8. **Phase 4** is ongoing.

---

## Verification checklist (per PR)

- [ ] `cargo test` and HTTP/YAML integration suite green for affected routes.
- [ ] `openapi.json` regenerated and diff reviewed; Spectral (§25) passes.
- [ ] Frontend + CLI updated if status codes, content types, or envelopes changed.
- [ ] Relevant BLC under `docs/business-logic-constraints/` updated when behavior or error codes change.
- [ ] Changelog entry for any client-observable change; deprecation/sunset dates recorded where applicable.

This plan is intended to be split into GitHub issues by section (one issue per numbered section; §6, §12, §13, §20 may split further by resource).
