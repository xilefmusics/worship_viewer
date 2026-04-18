# REST API design and architecture review (2026-04-18)

This review examines the Worship Viewer HTTP API as implemented in `backend/src` (Actix handlers, middleware, services), documented in `backend/openapi.json` (generated via `utoipa` in `backend/src/docs.rs`), and linted with [Spectral](https://stoplight.io/open-source/spectral). It uses the frameworks below as evaluation lenses, not as a compliance checklist.

**References:** [Microsoft REST API Guidelines](https://github.com/microsoft/api-guidelines/blob/vNext/Guidelines.md), [Zalando RESTful API Guidelines](https://opensource.zalando.com/restful-api-guidelines/), [OWASP API Security Top 10 (2023)](https://owasp.org/API-Security/editions/2023/en/0xa1-broken-object-level-authorization/), [Google Cloud API Design Guide](https://cloud.google.com/apis/design), [OpenAPI Specification 3.0.3](https://spec.openapis.org/oas/v3.0.3).

---

## Executive summary

The API shows **strong alignment** with common REST conventions: plural resource paths under a **URL major version** (`/api/v1`), clear separation of **auth** (`/auth/*`) from versioned resources, **RFC 7807** error bodies (`application/problem+json`), **offset pagination** with `X-Total-Count` and `Link`, **rate limiting** on `/api/v1/*` and most `/auth/*` routes, **admin-only** scopes (`RequireAdmin`) for sensitive surfaces, and **OpenAPI** as the contract with CI Spectral checks. Deprecation is handled explicitly for at least one legacy route (OpenAPI `deprecated` plus `Deprecation` / `Sunset` response headers).

Main improvement themes: **tighten the versioning story** (`info.version` vs path), **gradually enable stricter Spectral rules** (many are currently disabled), consider **cursor-based pagination** and **idempotency keys** for future scale and safer retries, and treat **BOLA** as an ongoing test and design discipline rather than a single endpoint concern.

---

## 1. Industry standards and best practices (Microsoft and Zalando)

### 1.1 Resource naming and URL structure

| Aspect | Observation |
| --- | --- |
| Plural nouns | Collections use plural segments (`/songs`, `/users`, `/teams`, `/collections`, `/setlists`, `/blobs`), consistent with both guidelines’ preference for noun-based collection URLs. |
| Hierarchy | Nested relationships are expressed in the path (for example `/api/v1/teams/{team_id}/invitations`, `/api/v1/users/me/sessions`). Zalando’s guidance to model relations in paths is followed. |
| Versioning | **URL path** major version `/api/v1` matches the common “explicit and visible” pattern both catalogs recommend for REST over the wire. |
| Auth split | Authentication and session establishment live under **unversioned** `/auth/*` while resources are under `/api/v1`. This is coherent if documented as stable (it is, in `info.description`), though it differs from APIs that version *all* externally visible HTTP surfaces. |

### 1.2 HTTP methods

| Pattern | Assessment |
| --- | --- |
| `GET` / `POST` / `PUT` / `PATCH` / `DELETE` | Standard mapping: lists and reads use `GET`; creates use `POST` (often `201 Created`); full replacement uses `PUT` (with documented upsert semantics for songs); partial updates use `PATCH`; deletes use `DELETE` with `204 No Content` where documented. |
| Verbs in paths | Sub-resource **actions** exist (`…/like`, `…/accept`). Both guideline sets allow this when domain operations are not a natural CRUD mapping; the trade-off is slightly more RPC-flavored URLs, offset by consistent noun-first collections elsewhere. |
| Conditional requests | Weak **ETag** on responses and **If-Match** / **If-None-Match** handling with **412 Precondition Failed** for conflicts align with predictable caching and concurrency (Microsoft’s emphasis on consistency and correct cache behavior). |

### 1.3 Status codes

- **201 Created** for successful creation (and for PUT upsert when creating a new song id, with `Location`), **200 OK** for successful reads and updates, **204 No Content** for successful deletes and logout, **302** for OIDC redirects, **400** for validation/pagination errors, **401** / **403** for authz/authn, **404** for missing resources, **409** for conflicts (e.g. duplicate user email), **412** for ETag precondition failures, **429** for rate limits, **500** for server failures — overall **consistent** with both guidelines’ expectations when paired with problem bodies.
- **Deprecation:** `POST /api/v1/invitations/{invitation_id}/accept` is marked deprecated in OpenAPI and emits **Deprecation** and **Sunset** headers in code, which supports client migration (Zalando’s deprecation practices).

### 1.4 Versioning clarity (minor gap)

OpenAPI `info.version` is **2.0.0** while the path prefix remains **`/api/v1`**. The long-form `info.description` explains that `info.version` reflects **wire-format generations**, not the path segment. That is defensible, but it can confuse client authors who equate “API version” with the URL (Microsoft explicitly warns about ambiguity in versioning stories). Consider one of: renaming `info.version` to reduce collision with “v1”, or adding a short **“Path `v1` vs spec version”** subsection in the description.

---

## 2. Security and code smells (OWASP API Security Top 10, 2023)

Below, items map loosely to [OWASP API categories](https://owasp.org/API-Security/editions/2023/en/0xa1-broken-object-level-authorization/); several require **runtime** and **process** validation, not static review alone.

| Risk | Relevant project behavior | Notes / residual risk |
| --- | --- | --- |
| **API1: Broken object-level authorization** | Team-scoped content flows through `UserPermissions` + team resolver services; admin monitoring lives behind `RequireAdmin`; user admin routes are under the same middleware. | **BOLA** remains the top API risk: every new handler must enforce ownership/team rules. Favor integration tests that attempt cross-user/cross-team access and expect `403`/`404` consistently (avoid resource enumeration where product policy requires it). |
| **API2: Broken authentication** | Session cookie and `Authorization: Bearer` session id; OTP flows; OIDC login/callback. Logout is idempotent (`204`). | Ensure production cookie flags (`secure`, `http_only`, `SameSite`) stay aligned with deployment (see `CookieConfig`). Document bearer format clearly for API clients (there is historical tension between optional “raw token” acceptance and strict `Bearer` in BLC docs — see `docs/reviews/2026-04-18-business-logic.md`). |
| **API3: Broken object property level authorization** | `PATCH` payloads use explicit `Patch*` types rather than unfiltered merge of arbitrary JSON. | Good direction; continue to avoid exposing internal-only fields on DTOs. |
| **API4: Unrestricted resource consumption** | Token-bucket **rate limits** per IP (`actix_governor`) on `/api/v1/*` and most `/auth/*`; separate limits for auth; **blob upload** max bytes enforced at payload config; list **page_size** capped (1–500). | Consider documenting **OIDC callback** being intentionally outside the governor (see `auth/rest.rs`) so security reviews do not flag it as an oversight. |
| **API5: Broken function level authorization** | `RequireAdmin` on monitoring and user administration subscopes. | Clear separation; keep OpenAPI response tables aligned with actual `403` cases. |
| **API6: Unrestricted access to sensitive business flows** | OTP verify documents lockout behavior in prose. | Ensure lockout and audit events remain wired in service code whenever auth flows change. |
| **API7: Server side request forgery** | No obvious “pass a URL and fetch” pattern in the reviewed surfaces; metrics windows are validated in handlers. | If future features accept URLs, add an explicit SSRF review. |
| **API8: Security misconfiguration** | Problem format is consistent; rate-limit responses documented with `Retry-After` and `X-RateLimit-*`. | Security headers at the edge (HSTS, CSP, framing) are typically **reverse-proxy** concerns; note if the API is fronted by a CDN or ingress and document expectations. |
| **API9: Improper inventory management** | **OpenAPI** is published at `GET /api/docs/openapi.json` without auth (**BLC-DOCS-001**), with a committed **`backend/openapi.json`** artifact. | Good for discoverability; pairs with the inventory risk that **public** docs reveal attack surface — acceptable if treated as intentional. |
| **API10: Unsafe consumption of APIs** | Primarily a **client** concern; server returns structured errors and stable `code` values in `Problem`. | Helps integrators fail safely. |

**Smells worth tracking:** mixing **platform admin** vs **team admin** semantics (documented: platform admin does not automatically get global write on team content). That is correct for BOLA but easy to regress in new endpoints.

---

## 3. Documentation and tooling (OpenAPI and Spectral)

### 3.1 OpenAPI quality

- **OpenAPI 3.0.3**, `components.schemas` with examples on several core DTOs, **securitySchemes** for cookie and bearer-shaped session header, per-operation **`operationId`**, and **tags** with **externalDocs** links into business-logic constraint docs in the repo.
- Errors reference **`Problem`** with `application/problem+json` in responses, matching **RFC 7807**-style usage.
- Deprecated schemas (`ErrorResponse`, `ProblemDetails`) remain for compatibility; the narrative in `info.description` states deprecation — aligns with cautious evolution.

### 3.2 Spectral

**Commands run (2026-04-18):**

- `npx @stoplight/spectral-cli@6 lint backend/openapi.json -r .spectral.yaml` — **passes** (no errors).
- `npx @stoplight/spectral-cli@6 lint backend/openapi.json` (stock `spectral:oas`) — **passes** at default fail severity.

The project `.spectral.yaml` **extends** `spectral:oas` but **turns off** several built-in rules, including:

- `oas3-valid-schema-example`
- `oas3-parameter-description`
- `operation-description`
- `openapi-tags-alphabetical`
- `info-contact` (contact is injected when env vars are set)
- `oas3-unused-component`

and adds a **custom** rule enforcing **snake_case** property names under `components.schemas`.

**Implication:** CI enforces naming consistency but **does not** currently fail builds on missing parameter descriptions, missing per-operation descriptions, or example/schema drift. Turning rules on incrementally (or adding a second “strict” ruleset) would catch more documentation inconsistencies without abandoning utoipa’s generated layout.

### 3.3 Non-standard or implementation-specific choices

- **Servers:** `servers.url` includes `/` and a production host — acceptable; clients should follow deployment-specific base URLs.
- **Global modifier `SessionSecurity`:** applies session-related security schemes across the document where utoipa attaches them; auth paths correctly omit session requirements in the published paths reviewed.

---

## 4. Developer experience (Google Cloud API Design Guide)

Themes from the [Google API Design Guide](https://cloud.google.com/apis/design) applied pragmatically to this codebase:

### 4.1 Pagination: offset vs cursor

- **Current:** Offset pagination via **`page`** (0-based) and **`page_size`**, with **`X-Total-Count`** and **RFC 5988 `Link`** (`first` / `prev` / `next` / `last`). This matches classical list semantics and is easy for clients.
- **Improvement:** For large collections or frequently changing data sets, **cursor-based** paging (opaque `page_token` / `next_page_token`) reduces **skipped-row cost** and duplicate/missing rows when the underlying set shifts between requests. Google’s standard lists strongly favor cursor tokens for scale; consider introducing optional cursor mode for hot lists first (for example songs with search), while keeping offset for admin UIs if needed.

### 4.2 Error message clarity

- **Strengths:** Stable **`code`** values documented in `info.description`; **`detail`** for human text; optional **`instance`** for correlation (see request-id / audit context in architecture).
- **Improvement:** Ensure **`type`** URIs for `Problem` are **actionable and stable** (often `https://…` URIs per problem type). If they are deployment-relative today, document the resolution base URL or mint permanent documentation URIs.

### 4.3 Idempotency

- **Strong:** `DELETE` semantics, logout **`204`**, conditional writes with **ETags**.
- **Gap:** **POST** creates (songs, blobs, teams, etc.) do not document **Idempotency-Key** (or similar) headers. Retries from mobile or flaky networks can duplicate resources. Google’s design often pairs mutating methods with idempotency for create operations that must not double-commit. Worth a design pass for the handful of high-value `POST` endpoints.

### 4.4 Long-running and standard methods

The API maps well to **standard list/get/create/update/delete** patterns for core resources. **Custom methods** (`like`, `accept`) are acceptable if kept small in number and consistently documented (they are).

---

## 5. Prioritized recommendations

1. **Clarify versioning in docs:** Explicitly reconcile **`/api/v1`** with **`info.version` 2.0.0** in one short, normative paragraph (and optionally in `CHANGELOG.md`) so generated SDKs and Postman collections do not mislabel generations.
2. **Spectral strictness:** Introduce a **gradual** tightening: e.g. enable `operation-description` or `oas3-parameter-description` on a subset of paths, or a weekly `spectral lint` report in CI at **warn** level without failing the build.
3. **Idempotency:** For critical `POST` operations, specify optional **`Idempotency-Key`** support (header name, storage duration, duplicate response behavior).
4. **Pagination roadmap:** Document when **cursor** pagination would appear (thresholds, which collections first) even if not implemented yet — sets client expectations.
5. **BOLA regression tests:** Keep Venom/HTTP tests that prove **no cross-tenant reads/writes** for songs, collections, blobs, teams, and invitations as the resource surface grows.

---

## Method

- **Code and config:** `backend/src/resources/**/rest.rs`, `backend/src/auth/rest.rs`, `backend/src/resources/rest.rs`, `backend/src/governor_*.rs`, `backend/src/auth/middleware.rs`, `backend/src/docs.rs`, `shared/src/error/*`, `.spectral.yaml`.
- **Contract:** `backend/openapi.json` (committed snapshot).
- **Lint:** Spectral CLI v6 against the default ruleset and `.spectral.yaml`.

This document is a **point-in-time** snapshot; generated OpenAPI and handlers may drift unless regenerated and re-reviewed after major changes.
