# Architecture review (2026-04-18)

This review compares **what the repository implements** and **what is described** in `docs/architecture/*`, `docs/business-logic-constraints/*`, and related notes. It also evaluates fit with common **AWS Well-Architected**, **Google Cloud Architecture Framework**, and **CNCF** practices at a high level.

**Scope:** The workspace contains a Rust (Actix) backend, SurrealDB, a Yew/WASM frontend, Docker packaging, and GitHub Actions. There is **no** Terraform, CloudFormation, Kubernetes manifests, or other cloud IaC in this repository, so **production topology** (regions, load balancers, managed DB) is **not verifiable from code** and is treated as external.

---

## 1. Implemented but not reflected in architecture diagrams or specs (“shadow” surface)

These items are real in code or delivery pipelines but are **absent** from `docs/architecture/backend-resource.md` and `docs/architecture/backend-request-flow.md`, which focus on **in-process** layering (handlers → services → SurrealDB + filesystem blobs).

### 1.1 Supply chain and release

| Item | Where it lives | Notes |
|------|----------------|-------|
| **Docker Hub image** `xilefmusics/worshipviewer` | `.github/workflows/backend-ci.yml` (`docker-publish` job), `README.md` | Multi-stage build + push on `main` and tags; not depicted in architecture docs. |
| **Venom** HTTP black-box tests | `Dockerfile` `tester` stage | Downloads Venom binary; runs `backend/tests/*.yml` against a spawned process before the final image. |
| **Trunk** WASM build | `Dockerfile`, `README.md` | Frontend is compiled in CI image build; architecture docs do not mention the WASM toolchain. |

### 1.2 Third-party integrations (runtime)

| Integration | Implementation | Doc visibility |
|-------------|----------------|----------------|
| **Google OAuth 2.0 / OIDC** | `openidconnect` crate; `Settings` env (`OIDC_*`); discovery via HTTP (`async_http_client`) | `backend-request-flow.md` describes OIDC flows and audit events; `backend/src/auth/oidc/client.rs` notes **Google only** in this deployment—broader “multi-IdP” is not modeled in diagrams. |
| **Gmail SMTP** | `MailService` uses **`smtp.gmail.com`** explicitly (`backend/src/mail.rs`) with app-password credentials | Not shown on request-flow diagrams; OTP email is an **out-of-band** dependency. |
| **SurrealDB** | `surrealdb::engine::any::connect` + optional DB auth (`backend/src/database/mod.rs`) | Diagrams show “SurrealDB” but not **transport** (`mem://` vs `ws://` vs `wss://`) or remote vs embedded. |

### 1.3 Client-side “hidden” data paths and caching

| Item | Where | Gap |
|------|-------|-----|
| **Service Worker offline cache** | `frontend/service-worker.js` | Caches GETs for **static assets** and **selected** `/api/v1/*` paths (`setlists`, `collections`, and player/songs subpaths). This is a **browser-side cache** layer not represented in backend architecture docs. |
| **Legal / marketing links** | `frontend/src/components/legal_links.rs` | Points to `https://worshipviewer.com/...` (imprint, privacy, terms)—external web surface not in backend diagrams. |

### 1.4 Operational and security-adjacent details

| Item | Notes |
|------|--------|
| `https://worshipviewer.invalid/problems/...` | Used in problem JSON (`governor_audit.rs`); stable **type** URIs, not necessarily resolvable—fine for RFC 7807, but not documented in architecture specs. |
| **Per-request team list memoization** | `UserPermissions` + `OnceCell` in `team/resolver.rs`—documented in `backend-request-flow.md` as **application-level** caching, not a distributed cache. |

---

## 2. Documented constraints or expectations vs implementation gaps

### 2.1 Business logic: contract vs code

| Reference | Documented expectation | Implementation note |
|-----------|------------------------|------------------------|
| **BLC-USER-006** (`user.md`) | `GET /users/me` **MAY** accept a raw `Authorization` value **without** `Bearer ` | **Likely mismatch:** prior review (`docs/reviews/2026-04-18-business-logic.md`) notes middleware/tests expect **Bearer**; optional behavior is documented as deployment-specific but **not** implemented as a configurable switch in-repo. |
| **BLC-HTTP-004** (`http-contract.md`) | `/api/v1/*` rate-limited **per client IP** | Keys use **`req.peer_addr()`** (see `PeerOrFallbackIpKeyExtractor` / `ProblemJsonPeerIpKeyExtractor`). **Behind a reverse proxy or load balancer**, the observed IP is often the **proxy**, not the end client, unless the proxy injects trusted headers and the app honors them. **Not implemented:** `Forwarded` / `X-Forwarded-For` handling for rate-limit keys. |
| **BLC-MON-001–004** (`monitoring.md`) | One `http_request_audit` row per request; admin-only listing | **Implemented** in spirit: `HttpAudit` middleware writes asynchronously (`tokio::spawn` except in tests); failures are logged, not surfaced to HTTP. |

### 2.2 Architecture / product docs that imply resilience not present in the codebase

The checked architecture documents **do not** promise multi-AZ, clustering, or external caches. The following are **not** gaps between doc and code, but **missing capabilities** if the product goal is cloud-grade HA:

| Concern | In repo? |
|---------|----------|
| **Circuit breakers** (DB, SMTP, OIDC discovery) | **No**—failures surface as errors/timeouts per library behavior. |
| **Dedicated caching layer** (Redis, CDN for API) | **No** server-side cache; **yes** client-side SW cache for some GETs. |
| **Health / readiness HTTP endpoints** | **No** `health`, `live`, or `ready` routes in `backend` (grep). `authentication.md` states health checks are **deployment-specific**—consistent with the repo. |
| **Horizontal scaling assumptions** | Stateless-ish app (sessions in DB), but **blob storage** (`blob_dir` / filesystem) and **in-memory rate limit state** (`actix-governor`) imply **single instance** or **shared filesystem + sticky sessions** unless deployment adds external stores and rate-limit store. |

### 2.3 Single points of failure (from code structure)

| Component | SPOF character |
|-----------|------------------|
| **SurrealDB** | Single logical database; no app-side replication or failover logic. |
| **Filesystem blob store** | `FsBlobStorage` + `blob_dir`—scaling out requires shared storage or object storage not in this repo. |
| **Gmail SMTP** | Single relay host; no fallback provider in code. |
| **Google OIDC** | Single IdP registration in `OidcClients`; discovery depends on Google availability. |

---

## 3. Well-Architected / Google CAF / CNCF-oriented observations

### 3.1 Security

**Strengths**

- Secrets in env; `Settings::Debug` redacts passwords and tokens (`backend/src/settings.rs`).
- Production guardrails: `initial_admin_user_test_session` forbidden when `WORSHIP_PRODUCTION` / `RUST_ENV=production` (`main.rs`).
- Session auth, OTP pepper, rate limits on `/auth` and `/api/v1`, audit events for auth and rate-limit (`docs/architecture/backend-request-flow.md`).
- Structured logging and request correlation (`tracing`, `X-Request-Id`, W3C `traceparent`).

**Risks / bottlenecks**

- **Rate limiting by peer IP** without trusted proxy headers can **under-limit** (one bucket per proxy) or **mis-attribute** audit `client_ip` fields.
- **No** central secrets manager in-repo—operators must inject secrets via environment (expected for many deployments, but worth formalizing for production).
- **CSP / HSTS / framing** at the edge are called out in `docs/reviews/2026-04-18-rest-api-review.md` as **reverse-proxy** concerns—still accurate; not enforced in Actix for static+API.

### 3.2 Reliability and performance efficiency

- **HTTP audit** writes are **fire-and-forget** (`tokio::spawn`)—good for latency; under extreme load, DB backpressure could theoretically queue tasks (no explicit backpressure/circuit breaker).
- **Team ACL resolution** uses per-request caching—reduces duplicate Surreal round-trips within one request.
- **Conditional GET / ETag** (`http_cache.rs`) supports efficient revalidation where used.
- **Frontend** service worker can reduce repeat network GETs for listed routes when offline or flaky—**stale data** risk should be understood for setlist/collection views.

### 3.3 Cost optimization

- **Single binary** + static files + optional Surreal—low moving parts for small scale.
- **Docker `scratch`**-style final image reduces attack surface and image size; build cost is higher (full Rust + WASM build in CI).
- **No** evidence of autoscaling policies or FinOps instrumentation in-repo (expected outside repo).

### 3.4 Operational excellence (CNCF-aligned)

**Strengths**

- **Observability:** `tracing` + JSON logs in production; audit event catalog in `backend-request-flow.md`.
- **CI:** `cargo test`, Clippy, fmt, Spectral on `openapi.json`.
- **Contract tests:** Venom suites in Docker build.

**Gaps**

- **No** distributed tracing exporter (OpenTelemetry) in the reviewed paths—**W3C traceparent** is consumed for request id only.
- **No** documented SLOs or runbooks in this repo.
- **Kubernetes-style** probes: not applicable in code; operators must add externally if they use orchestrators.

---

## 4. Recommendations (prioritized)

1. **Document or implement proxy-aware client IP** for rate limiting and audit (`X-Forwarded-For` / `Forwarded` with **trusted hop** configuration), or document that rate limits apply to **proxy IP** only when behind a reverse proxy.
2. **Resolve BLC-USER-006** vs implementation: either implement optional raw-token acceptance behind a flag or narrow the BLC to “Bearer only” to match code and tests.
3. **Add architecture doc section** for: CI/CD → Docker Hub, SMTP/OIDC external dependencies, and **service worker** caching behavior (or link to a short `docs/frontend-*` note if created later).
4. **If HA is required:** specify target deployment (e.g. multi-instance + shared blob storage + SurrealDB HA or managed DB) and add **health/readiness** routes and **stateless** rate limiting (Redis or gateway) as part of that design.
5. **Consider** OpenTelemetry export for traces/metrics in production to align with CNCF observability stack patterns.

---

## 5. Sources reviewed

- `docs/architecture/backend-resource.md`, `docs/architecture/backend-request-flow.md`
- `docs/business-logic-constraints/*` (especially `http-contract.md`, `monitoring.md`, `user.md`, `authentication.md`)
- `docs/reviews/2026-04-18-business-logic.md`, `docs/reviews/2026-04-18-rest-api-review.md`
- `README.md`, `Dockerfile`, `.github/workflows/backend-ci.yml`
- `backend/src/main.rs`, `settings.rs`, `http_audit.rs`, `governor_peer.rs`, `mail.rs`, `database/mod.rs`, `auth/oidc/client.rs`, `frontend/service-worker.js`
