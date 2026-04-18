# Business logic review (2026-04-18)

This review compares `docs/business-logic-constraints/*.md` (BLC ids) with the Rust backend, automated tests, and a few related docs. It is a snapshot; line-level drift is always possible.

## Method

- **Documented constraints:** all files under `docs/business-logic-constraints/`.
- **Implementation:** primarily `backend/src` (handlers, services, `team/resolver.rs`, middleware).
- **Tests:** `backend/src/http_tests.rs` (BLC-tagged slices), `backend/src/**/service.rs` unit tests, `backend/src/audit_events_tests.rs`, and HTTP integration suites in `backend/tests/*.yml` (Venom), referenced where they map to BLC ids in comments.

---

## 1. Implemented in code but missing or thin in BLC docs

These behaviors exist in the codebase (and often in OpenAPI) but are not called out as numbered BLC rules in the constraint docs, or only appear in one resource file.

| Area | What exists | Gap |
| --- | --- | --- |
| **Conditional HTTP (ETags)** | Weak `ETag`, `If-None-Match` (304), `If-Match` (412) on single-resource GET/PATCH/PUT for songs, collections, setlists, blobs; `http_cache` helpers. | Only blob byte responses spell out caching in **BLC-BLOB-016**. Song/setlist/collection resources lack parallel “BLC-*” bullets for the same pattern. |
| **API documentation beyond BLC-DOCS-001** | `http_tests.rs` documents slices **BLC-DOCS-002** (Problem + `application/problem+json`), **BLC-DOCS-003** (auth route query params), **BLC-DOCS-004** (OpenAPI snake_case keys). | These are **not** listed in `docs/business-logic-constraints/api-documentation.md`, which only defines **BLC-DOCS-001** (unauthenticated OpenAPI JSON). |
| **Monitoring metrics** | `GET /api/v1/monitoring/metrics` with time window validation and admin-only access; tests in `http_tests.rs` (`monitoring_metrics_*`). | `monitoring.md` suggests future endpoints under `/monitoring/` but does not describe this metrics aggregate as a BLC (by design or oversight). |
| **Rate limiting details** | `PeerOrFallbackIpKeyExtractor`, `use_headers()` on the API governor, separate auth-route limits. | **BLC-HTTP-004** names defaults and 429 behavior but not keying/fallback IP behavior. |
| **Team content ACL implementation** | `content_read_team_things` includes all non-public teams for platform admins; `content_write_team_things` **does not** grant extra write teams to platform admins (comment: “Platform admin does not imply global write”). | The “no admin bypass on write” rule is stated per resource (**BLC-SONG-002**, **BLC-COLL-002**, etc.) but the **central invariant** lives only in `resolver.rs`, not as a single cross-cutting BLC. |
| **SPA / unknown routes** | `spa_fallback_guard` tests in `http_tests.rs` ensure unknown paths under `/api` / `/auth` return Problem JSON. | Not part of `http-contract.md`. |

---

## 2. Documented as BLC but wrong, inconsistent, or ambiguous vs implementation

| BLC / doc | Issue |
| --- | --- |
| **BLC-SONG-018** (`song.md`) | Says PUT upsert to a new id returns **200**. The implementation and `create-update-policy.md` use **201 Created** with `Location` for new ids; `song/rest.rs` OpenAPI matches 201. **The song constraint doc should be updated** to 201 (or explicitly defer to create-update policy). |
| **BLC-USER-006** (`user.md`) | States the server **MAY** accept a raw session token without `Bearer ` on **GET /users/me**. Current middleware behavior and `http_tests` expect **401** without the `Bearer ` prefix. Either the deployment must implement the optional acceptance, or **the BLC should be tightened** to “Bearer only” to match code and tests. |
| **BLC-SONG-015 / BLC-COLL-012 / similar** | Several constraints use **MAY** for 500 vs partial data after deletes. That documents uncertainty rather than a single product decision; useful for clients but weak as a “constraint.” |

---

## 3. Documented BLC with incomplete automated test coverage

Coverage is **strong** for auth, OpenAPI, HTTP path ids, idempotent DELETE, user/session admin gates, list query validation, monitoring audit links, team invitations (service + Venom), and many song/team flows (service tests + Venom).

Gaps worth addressing:

| BLC | Notes |
| --- | --- |
| **BLC-HTTP-004** | Rate limiting on `/api/v1` is implemented (`Governor` in `resources/rest.rs`) but there is **no test** that forces 429 / asserts `Retry-After` / `X-RateLimit-*` (brittle under load; may need a test-only limit config). |
| **BLC-LP-010** | **Link** header with `first`/`prev`/`next`/`last` is built via `request_link::list_link_header` on list handlers. **No HTTP test** asserts presence or relation URLs (only `X-Total-Count` appears in some tests). |
| **BLC-SONG-002**, **BLC-COLL-002**, **BLC-SETL-002**, **BLC-BLOB-002** (platform admin: read extra scope, **no** mutate without team rights) | Enforced by `content_read_team_things` vs `content_write_team_things`. There is **no dedicated test** that uses a **platform admin** user attempting PUT/DELETE on another team’s song/collection/setlist/blob expecting **404** (or the documented status). |
| **BLC-AUTH-OTP-001–003** (`authentication.md`) | OTP flows are exercised indirectly (`audit_events_tests` hits `/auth/otp/request` and verify for audit logging). There is **no focused test suite** for rate limits, lockout after max attempts, or **`WORSHIP_OTP_ALLOW_SELF_SIGNUP`** false → 400 behavior. |
| **BLC-MON-001** | “Every request → one `http_request_audit` row” is implied by middleware design; tests **sample** authenticated and deleted-user cases. There is **no exhaustive matrix** (e.g. static assets, 404, governor 429) proving one row per route class. |
| **BLC-TINV-010** (deprecated alternate accept path + **Deprecation** / **Sunset** headers) | Invitation service tests cover acceptance logic; **HTTP-level** tests for the deprecated route’s headers were not verified in this pass. |

**Positive note:** **BLC-USER-009** (guest on personal team cannot read owner user → **403**) is covered in `backend/tests/2_users.yml` (`get-user-read-grant-on-target-owner`). **BLC-USER-012** / **BLC-SESS-008** (sessions stop working after user delete → **401**) appears in the same file (`get-default-user-session-after-delete`).

---

## 4. Recommendations for consistency and good practice

1. **Single source of truth for HTTP semantics**  
   Align **BLC-SONG-018** with `create-update-policy.md` and OpenAPI (status code for song PUT upsert). Avoid three different stories for the same operation.

2. **Resolve BLC-USER-006 vs code**  
   Either implement optional raw-token acceptance behind a flag or remove the “MAY” from the BLC and document Bearer-only consistently (including README’s “Bearer token” line).

3. **Promote cross-cutting rules**  
   Add a short **`docs/business-logic-constraints/platform-admin-content.md`** (or extend `http-contract.md`) with one bullet: platform **admin** expands **read** visibility for teams/content listing but **never** receives library **write** solely from `role = admin`—with a pointer to `content_write_team_things`. Then individual resource docs can say “see platform admin rule” instead of repeating wording.

4. **ETags and caching**  
   Either add mirrored BLC lines for song/collection/setlist conditional requests or add one shared **“conditional GET/PATCH”** note under `http-contract.md` referencing `http_cache.rs` behavior.

5. **OpenAPI documentation BLCs**  
   Fold **BLC-DOCS-002–004** into `api-documentation.md` (or explicitly mark `http_tests.rs` module docs as the canonical list) so contributors do not discover them only by reading tests.

6. **Testing strategy**  
   - Add a **contract test** for **Link** + **X-Total-Count** on one list endpoint.  
   - Add **one** platform-admin negative test for content mutation (any resource).  
   - Optionally add **OTP** integration tests with env overrides in test harness for self-signup off and lockout.

7. **Tighten “MAY 500” language**  
   For collection/song list endpoints, decide whether stale song ids yield a defined error, empty slots, or 500—then replace speculative **MAY** text with a single behavior (or keep **MAY** but add a tracking issue).

---

## Appendix: Where tests reference BLCs

| Location | Role |
| --- | --- |
| `backend/src/http_tests.rs` | Auth, docs, HTTP contract, user/session admin, pagination, monitoring, some song PATCH |
| `backend/src/resources/*/service.rs` | Deep tests for songs, teams, blobs, collections, setlists, invitations, users |
| `backend/tests/*.yml` | End-to-end HTTP; many steps tagged with `# BLC: …` in comments |
| `backend/src/audit_events_tests.rs` | Auth/OTP/logout audit events (not full BLC-OTP matrix) |

This split is healthy; the main improvement is **aligning docs with code** where they diverge and **closing the listed test gaps** for cross-cutting concerns (admin write denial, rate limit, Link header, OTP settings).
