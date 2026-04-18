# Backend Logging Review

Scope: the Rust/actix-web backend crate in `backend/`.
Evaluated against commonly accepted observability practices for production web services written in Rust with the `tracing` ecosystem.

## Executive summary

The backend uses the right library (`tracing` + `tracing-subscriber`) and has a handful of well-crafted log sites (notably in `backend/src/database/migrations.rs`). Beyond that, however, **logging is almost entirely absent from the application code**: only **4 out of 79 Rust source files** emit any log event. The infrastructure needed for production observability — span instrumentation, request‑ID correlation, structured access logs, error‑chain logging, audit logs, configurable output format — is either missing, under‑wired, or not used consistently.

The most impactful gap is that the `X-Request-Id` created in `backend/src/request_id.rs` is **never surfaced in `tracing` output**. As a consequence, even the few existing log lines cannot be correlated with the HTTP request that produced them, which is the single most important property a log pipeline of this kind should have.

Overall rating: **Needs substantial improvement.** The foundations are sound, but the code base is effectively operating without application‑level observability.

---

## 1. What the backend does today

### 1.1 Dependencies

From `backend/Cargo.toml`:

```27:28:backend/Cargo.toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

No `tracing-actix-web`, no `tracing-bunyan-formatter`, no `tracing-log`, no `tracing-opentelemetry`, no `tracing-appender`, no `json` feature on `tracing-subscriber`.

### 1.2 Subscriber initialization

```31:37:backend/src/main.rs
tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
    )
    .with_target(false)
    .compact()
    .init();
```

- Default level is `info`, overridable via `RUST_LOG` (standard and correct).
- `with_target(false)` suppresses the module path — helpful for humans, harmful for grepping by module.
- `.compact()` emits a human format; **no JSON formatter** is configured.
- No `tracing-log` bridge is installed, so `log::*` calls made by dependencies (e.g. `reqwest`, `surrealdb`, `lettre`, `openidconnect`) are **silently dropped**.
- No file/stderr routing, no non‑blocking writer (`tracing-appender`), no rotation.

### 1.3 HTTP access logs

```181:183:backend/src/main.rs
.wrap(Logger::new(
    r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
))
```

This is actix‑web's built‑in `middleware::Logger`, which writes through the `log` crate facade. Because the `tracing-log` bridge is not installed, these **access log lines never reach `tracing-subscriber`** — they are emitted via `log`'s default behavior (i.e. dropped unless another `log` sink is installed). The format is also positional (Combined‑style), not structured, so it cannot be parsed by a log aggregator without a custom grok/regex.

### 1.4 Request‑ID middleware

```75:98:backend/src/request_id.rs
fn call(&self, req: ServiceRequest) -> Self::Future {
    let id = req
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .and_then(span_id_from_traceparent)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    req.extensions_mut().insert(id.clone());
    let target = req
        .uri()
        .path_and_query()
        .map(|pq| pq.to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    req.extensions_mut().insert(ApiRequestTarget(target));
    // ...
    resp.headers_mut()
        .insert(X_REQUEST_ID.clone(), id.parse().unwrap());
```

A request ID is generated (preferring a W3C `traceparent` span id, falling back to UUIDv4), stored in request extensions, and echoed in the response header. **It is never attached to a `tracing` span**, so it does not appear on any log line. It is only used to populate the `instance` field of the Problem Details body in `error.rs`.

### 1.5 Actual log sites

Only 4 source files emit any `info!/warn!/error!/debug!/trace!`:

| File | Level(s) | Notes |
|------|----------|-------|
| `backend/src/main.rs` | info | Startup: static dir, admin user bootstrap, host/port. |
| `backend/src/database/migrations.rs` | info, error | **Well‑formed structured logs**: `migration = %name, duration_ms = …, status = …`. Full error chain + `Debug` dump for SurrealDB failures. |
| `backend/src/auth/rest.rs` | warn | Single `warn!(session = session_id, …)` when session deletion on logout fails. |
| `backend/src/error.rs` | error | Logs DB conflicts, DB validation errors, and all `AppError::Internal` responses. |

There are **no** `#[instrument]` attributes, no `info_span!` / `error_span!`, no custom spans, and no `in_scope` usages across the crate.

### 1.6 Environment & deployment

- No `RUST_LOG`, `RUST_BACKTRACE`, `LOG_LEVEL`, or `LOG_FORMAT` is set anywhere in the repo (`Dockerfile`, CI, README).
- The container runs on scratch; stdout is the only log sink.
- `.gitignore` correctly excludes `venom*.log` (functional‑test runner logs). There are uncommitted `backend/venom*.log` files on disk, but none tracked.

---

## 2. What is done well

The following patterns should be preserved and replicated elsewhere.

1. **Using the `tracing` crate.** It is the de‑facto standard in async Rust and a superset of `log`. Spans, structured fields, async‑aware context propagation are all available once wired up.
2. **`EnvFilter` with an `info` default.** Operators can raise verbosity per‑module (`RUST_LOG=backend::auth=debug,info`) without a redeploy.
3. **Structured fields in `migrations.rs`.** These entries are exactly what the rest of the crate should look like:

   ```42:59:backend/src/database/migrations.rs
   info!(
       migration = %script_name,
       status = "already_applied",
       "database migration already applied, skipping"
   );
   // ...
   info!(migration = %script_name, "applying database migration");
   apply_migration(db, &script_name, &checksum, &script).await?;
   let elapsed = started.elapsed();
   info!(
       migration = %script_name,
       duration_ms = elapsed.as_millis() as u64,
       status = "applied",
       "database migration finished successfully"
   );
   ```

4. **Error‑chain unrolling in migrations.** `log_surreal_error_chain` walks `std::error::Error::source`, joins the chain with `" <- "`, and also dumps `?err` for `Debug`. This is textbook good practice — adopt it for every `Internal`/`database`/`mail`/`oidc` error site.
5. **Top‑level error logging in `AppError::error_response`.** Logging only `Internal` variants (not 4xx) and keeping a sanitized detail for the client is correct:

   ```210:213:backend/src/error.rs
   fn error_response(&self) -> HttpResponse {
       if matches!(self, AppError::Internal(_)) {
           error!("{}", self);
       }
   ```

6. **Sanitizing surfaces.** The Problem Details response collapses DB validation errors to `"invalid request"` while still logging the raw DB error server‑side (`error!("database validation error: {dberr}")`). This is the right way to keep internal field/index names off the wire.
7. **Generated `X-Request-Id`.** The middleware picks up a W3C `traceparent` span id when present, otherwise a UUIDv4, and echoes it on the response. The infrastructure is in place — it just isn't wired into `tracing` (see §3.1).
8. **No `println!` / `eprintln!` / `dbg!`.** The crate is free of ad‑hoc stderr writes, which is easy to regress on.
9. **Boot‑time safety warning.** `main.rs` refuses to start when `initial_admin_user_test_session` is enabled under `WORSHIP_PRODUCTION=true`. This fails loudly with an `anyhow::bail!` rather than just logging and continuing.

---

## 3. What should be improved

Findings are grouped from most to least impactful.

### 3.1 Request IDs do not appear in logs (critical)

`RequestId` puts the id in `req.extensions_mut()` but never in a `tracing::Span`. Every `info!/warn!/error!` site currently logs with no correlation field. In a production incident you cannot join access logs to application logs to DB‑error logs for the same request.

**Recommended fix.** Either wrap the app with `tracing-actix-web::TracingLogger` (which creates a root span per request and honors `traceparent`), or open a span manually inside `RequestIdMiddleware::call`:

```rust
let span = tracing::info_span!(
    "http.request",
    request_id = %id,
    method = %req.method(),
    route = req.match_pattern().as_deref().unwrap_or(req.path()),
    user_id = tracing::field::Empty,
);
// attach after RequireUser runs:
// span.record("user_id", &tracing::field::display(&user.id));
async move { service.call(req).instrument(span.clone()).await }
```

After this, **every** downstream `info!/warn!/error!` inherits `request_id` with no further code changes, including the ones already in `error.rs` and `auth/rest.rs`.

### 3.2 Access logs and application logs live in different worlds (high)

`actix_web::middleware::Logger` writes through `log`, not `tracing`. The format is also positional and not machine‑parseable:

```
%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T
```

Consequences:

- No JSON, no `request_id` field, no latency histograms.
- If `RUST_LOG` is set for `tracing`, the access log channel is unaffected; operators routinely lose one or the other.
- Combining the two in a log aggregator (Loki, Datadog, CloudWatch) requires per‑line regex.

**Recommended fix.** Replace `Logger::new(...)` with `tracing_actix_web::TracingLogger::default()` (it emits one structured span per request with method/route/status/latency) and/or install `tracing_log::LogTracer::init()` so `log` events (from actix, reqwest, surrealdb, lettre, openidconnect, …) flow into the same subscriber. The second step is essentially free and picks up a lot of dependency diagnostics that are currently silently dropped.

### 3.3 No JSON/structured output in production (high)

`tracing_subscriber::fmt().compact()` produces human‑readable lines. In containerized environments these cannot be parsed reliably by log aggregators.

**Recommended fix.** Select the formatter based on environment:

```rust
let builder = tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_target(true);
if is_production() {
    builder.json().flatten_event(true).init();
} else {
    builder.compact().init();
}
```

Enable `tracing-subscriber`'s `json` feature. Consider `tracing-bunyan-formatter` if you want a schema that aggregates well in ELK/Loki.

### 3.4 Logging is almost entirely absent from business/service/auth code (high)

Only `main.rs`, `error.rs`, `migrations.rs`, and `auth/rest.rs` log. The following high‑value flows emit **nothing**:

| Flow | File | Missing events |
|------|------|----------------|
| OIDC login redirect | `backend/src/auth/oidc/rest.rs` | start of flow, provider, callback success/failure, email claim missing, token exchange failures |
| OIDC callback | `backend/src/auth/oidc/rest.rs` | same |
| OTP request | `backend/src/auth/otp/rest.rs` | OTP generated (without code), mail send failure, rate‑limit hits |
| OTP verify | `backend/src/auth/otp/rest.rs` | success, wrong code, lockout |
| Session creation | `backend/src/resources/user/session/service.rs` | new session id, ttl, user id |
| Session validation / expiry | `backend/src/auth/middleware.rs` | unauthorized reasons, expired vs missing |
| Admin bypass checks | `backend/src/auth/middleware.rs` | forbidden access attempts |
| User create/delete | `backend/src/resources/user/rest.rs` | who did what |
| Team role changes, invitations | `backend/src/resources/team/invitation/service.rs` | accept, decline, mismatch |
| Blob upload/delete (incl. orphaned files) | `backend/src/resources/blob/{rest,service,storage}.rs` | file id, size, content type, delete errors |
| Mail send | `backend/src/mail.rs` | who, subject template name, transport response |
| OIDC client build (startup) | `backend/src/auth/oidc/client.rs` | provider discovery success, scopes |

A good rule of thumb: every `Result` that is propagated to `?` from an I/O boundary should be either logged at the boundary with `tracing::error!(error = %err, …)` or be captured in an instrumented span so a higher‑level handler can log it with the span fields attached.

### 3.5 `#[instrument]` is not used anywhere (medium)

Every service method is async, returns `Result<…, AppError>`, and takes structured arguments. These are exactly the functions that benefit most from `#[tracing::instrument]`:

```rust
#[tracing::instrument(
    skip(self, session),
    fields(user_id = %session.user.id, session_id = tracing::field::Empty),
    err,
)]
pub async fn create_session(&self, session: Session) -> Result<Session, AppError> {
    let stored = self.repo.create_session(session).await?;
    tracing::Span::current().record("session_id", &tracing::field::display(&stored.id));
    Ok(stored)
}
```

Benefits:

- Automatic entry/exit spans with fields.
- `err` attribute logs the error on an early return — covers the "swallowed errors" finding (§3.7).
- Composable with the request span from §3.1.

### 3.6 Error types drop source chains (medium)

Several conversions collapse errors to `to_string()` or `format!("…: {}")`, discarding the causal chain:

```84:90:backend/src/error.rs
pub fn oidc<E: std::fmt::Display>(err: E) -> Self {
    Self::Internal(err.to_string())
}

pub fn database<E: std::fmt::Display>(err: E) -> Self {
    Self::Internal(format!("database error: {}", err))
}

pub fn mail<E: std::fmt::Display>(err: E) -> Self {
    Self::Internal(format!("mail error: {}", err))
}
```

And in `AppError::error_response`:

```211:213:backend/src/error.rs
if matches!(self, AppError::Internal(_)) {
    error!("{}", self);
}
```

`{}` on an `AppError::Internal(String)` prints only the top‑level message. The SurrealDB / lettre / openidconnect source chain is long and almost always the actionable part. `migrations.rs` already shows the right pattern (`log_surreal_error_chain`). Either:

- Log at the construction site before converting to `AppError`, so the original typed error is still in scope:

  ```rust
  .map_err(|err| {
      tracing::error!(error = %err, error_debug = ?err, "mail transport failed");
      AppError::mail(err)
  })?
  ```

- Or extend `AppError::Internal` to carry `anyhow::Error` (keeping the `source()` chain) and walk it inside `error_response`.

### 3.7 Silently swallowed errors (medium)

Several I/O paths drop errors on the floor. These are the exact places an operator wants to know about in an incident:

```45:49:backend/src/resources/blob/storage.rs
fn delete_blob_file(&self, blob: &Blob) {
    if let Some(name) = blob.file_name() {
        let path = Path::new(&self.blob_dir).join(name);
        let _ = std::fs::remove_file(path);
    }
}
```

```149:160:backend/src/resources/user/surreal_repo.rs
let _ = self
    .inner()
    .db
    .query("UPDATE $user SET default_collection = $collection")
    ...
    .await?;
Ok(())
```

```220:228:backend/src/resources/collection/surreal_repo.rs
let _ = db
    .db
    .query(...)
    ...
    .await?;
Ok(())
```

- `delete_blob_file` can leak files forever without a trace. At minimum `warn!(blob_id = %blob.id, path = %path.display(), error = %err, "failed to delete blob file")`.
- The `let _ = db.query(...).await?` idiom does propagate the `Result`, but it also hides `response.take_errors()` — a Surreal `Response` can "succeed" at the transport layer while individual statements return errors, exactly as `ensure_no_statement_errors` in `migrations.rs` handles. That pattern should be reused here (and/or at least logged at `warn` on non‑empty error lists).

### 3.8 No audit trail for sensitive operations (medium)

Operations that legal/compliance/security want a record of are currently invisible:

- Admin user creation and deletion.
- Role changes (`RequireAdmin`‑protected endpoints).
- Login success/failure per provider.
- Session creation/rotation/revocation.
- Team invitation accept/decline.
- Rate‑limit rejections (`actix-governor` drops 429s without a log site).

For each of these, emit one structured `info!` with a stable event name (e.g. `audit.user.created`, `audit.auth.login.success`, `audit.session.revoked`) and the subject/actor ids. Even without shipping to a separate audit stream, this gives you the history you need.

### 3.9 No bootstrap/config summary (low)

`main.rs` logs "Serving static files from …" and "Starting server on http://…:…" but not:

- which OIDC providers were registered,
- whether `cookie_secure` is true/false,
- `session_ttl_seconds`, `otp_ttl_seconds`, OTP self‑signup flag,
- rate‑limit parameters,
- `WORSHIP_PRODUCTION`/`RUST_ENV` resolution.

These are ideal for an `info!` summary at startup (without secrets). A single "settings loaded" event with the resolved knobs would make production triage substantially faster and also document the environment in the log stream.

### 3.10 `Logger` format choices (low)

Even if you keep actix's `Logger`, the current format logs `%T` (seconds as float with 6 decimals) rather than `%D` (milliseconds as integer). Integer‑millisecond fields are far easier to aggregate into histograms. And `%a` is the peer address, which behind a reverse proxy is always the proxy — consider honoring `X-Forwarded-For` (or, better, switch to `tracing-actix-web` and let the span capture a validated `client_ip`).

### 3.11 Settings & secrets risk (low but worth checking)

`Settings` derives `Debug`. It contains `otp_pepper`, `gmail_app_password`, `apple_client_secret`, `oidc_client_secret`, `db_password`. Nothing logs `Settings` today, but nothing structurally prevents a future `tracing::debug!(?settings, …)` from dumping the pepper into the log stream. Consider implementing a custom `Debug`/`Display` that redacts secret fields, or wrap each secret in a `secrecy::Secret<String>` so even explicit formatting fails closed.

### 3.12 `unwrap()` in the request‑ID middleware (very low)

```93:94:backend/src/request_id.rs
resp.headers_mut()
    .insert(X_REQUEST_ID.clone(), id.parse().unwrap());
```

The id is a UUID or a 16‑hex‑char span id, both of which parse as `HeaderValue`, so this cannot panic in practice. It is still worth a `expect("request id is valid header value")` so the intent is documented.

### 3.13 Testing

There is no `tracing-test` or log‑capture assertion anywhere. Tests cannot verify that a failed OTP attempt produced a `warn!`, or that a deletion audit event was emitted. Consider `tracing-test` or a small in‑memory `Layer` for audit‑relevant assertions.

---

## 4. Recommended minimum changes (in order)

Each step is small, independent, and strictly additive:

1. **Add the `tracing-log` bridge** (`LogTracer::init()`) so dependency `log` events reach the subscriber. One line.
2. **Switch `Logger::new(...)` to `tracing_actix_web::TracingLogger::default()`** and put the `RequestId` middleware *below* it. Update `RequestIdMiddleware::call` to inject `request_id` as a field on the current `Span` (`Span::current().record("request_id", …)`), or replace it with `TracingLogger` using a custom `RootSpanBuilder` that respects `traceparent`.
3. **Add `json` formatting when `WORSHIP_PRODUCTION=true`**. Enable the `json` feature on `tracing-subscriber`.
4. **Sprinkle `#[tracing::instrument(skip(self), err)]` on service methods** in `resources/**/service.rs`, `auth/oidc/rest.rs`, `auth/otp/rest.rs`, `auth/middleware.rs`. No new log sites required — `err` handles the error side.
5. **Log error source chains.** Generalize `log_surreal_error_chain` from `migrations.rs` into a reusable helper (`fn log_error_chain(err: &dyn std::error::Error, target: &'static str)`) and call it where `AppError::Internal/database/mail/oidc` are constructed.
6. **Close silent‑swallow sites**: `FsBlobStorage::delete_blob_file`, the `let _ = db.query(...)` calls, and `RequireUser`'s unauthorized branches should emit at least a `warn!`.
7. **Add audit log sites** for: auth success/failure per provider, session create/revoke, admin mutations, team membership changes, rate‑limit rejections.
8. **Startup summary**: one `info!` with the resolved configuration (no secrets).
9. **Document logging** in `README.md` (and/or `docs/architecture/backend-request-flow.md`): supported `RUST_LOG` examples, JSON vs compact mode, request‑id propagation contract (`traceparent` → `X-Request-Id` → span field).
10. **Protect settings**: implement `Debug` manually on `Settings` (or use `secrecy`) so accidental `tracing::debug!(?settings)` never leaks the OTP pepper or SMTP credentials.

---

## 5. Appendix: field vocabulary to standardize

If you adopt span instrumentation, lock in a small set of canonical field names now so ad‑hoc additions don't drift:

| Field | Type | Meaning |
|-------|------|---------|
| `request_id` | string | UUID or W3C span id; set by `RequestId` middleware. |
| `user_id` | string | Authenticated user id; added by `RequireUser`. |
| `session_id` | string | Session id being created/validated/revoked. |
| `team_id` | string | Resolved team context for the request. |
| `route` | string | Matched actix route pattern (`/api/v1/songs/{id}`). |
| `method` | string | HTTP method. |
| `status` | u16 | HTTP status code on response. |
| `latency_ms` | u64 | Total request latency. |
| `event` | string | Stable event name for audit entries (`audit.user.created`, `audit.auth.login.success`). |
| `error` | Display | `%err` — short message. |
| `error_debug` | Debug | `?err` — developer detail. |
| `error_source_chain` | string | `a <- b <- c` rendering of `Error::source`. |

Every new log site should reuse these names instead of inventing new ones (`uid`, `uid_`, `userId`, etc.).

---

## 6. Action plan

Each phase below is independent and shippable on its own. Every task lists the file(s) to touch, a concrete acceptance criterion, and an estimated size (S ≤ ½ day, M ≤ 1 day, L = multi‑day). Unless noted otherwise, nothing here is a breaking change for API consumers — logging is added alongside existing behavior.

The phases are ordered by **value per unit of effort**. You can stop after any phase and still ship something useful.

### Phase 0 — Prerequisites (S, ~2h)

Goal: unblock every subsequent phase without changing runtime behavior yet.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 0.1 | Add `tracing-actix-web = "0.7"`, enable the `json` feature on `tracing-subscriber`, add `tracing-log = "0.2"`. | `backend/Cargo.toml` | `cargo check -p backend` green. |
| 0.2 | Add a private `backend/src/observability.rs` module holding a single `pub fn init() -> anyhow::Result<()>` that configures the subscriber. Call it from `main.rs` in place of the inline `tracing_subscriber::fmt()...init()`. | `backend/src/lib.rs`, `backend/src/main.rs`, new `backend/src/observability.rs` | `main.rs` no longer imports `tracing_subscriber`; behavior unchanged. |
| 0.3 | Inside `observability::init`, install `tracing_log::LogTracer::init()` so `log::*` events (actix, reqwest, surrealdb, lettre, openidconnect) flow into the subscriber. | `backend/src/observability.rs` | Running the backend with `RUST_LOG=surrealdb=debug` surfaces surreal `log` events. |

**Validation**: `cargo run` still starts and prints the existing `info!` lines unchanged.

---

### Phase 1 — Request correlation (S, ~3h) — highest priority

Goal: every log line emitted while serving a request carries `request_id`, `method`, `route`, `status`, `latency_ms`.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 1.1 | Replace `.wrap(Logger::new(...))` with `.wrap(TracingLogger::<WorshipRootSpan>::new())`. | `backend/src/main.rs` | Access log is emitted by `tracing` with structured fields. |
| 1.2 | Implement `WorshipRootSpan: RootSpanBuilder` inside `request_id.rs`. It must: (a) honor incoming `traceparent` — reuse `span_id_from_traceparent` — or fall back to UUIDv4; (b) create an `info_span!("http.request", request_id = %id, method = %req.method(), route = …, user_id = Empty, status = Empty, latency_ms = Empty)`; (c) on response, `span.record("status", status.as_u16())` and elapsed ms. | `backend/src/request_id.rs` | `curl -H 'traceparent: 00-…-abcdef0123456789-01' /auth/login` produces a log entry with `request_id=abcdef0123456789`. |
| 1.3 | Keep `RequestIdMiddleware` but delete its own `tracing` responsibilities — it only inserts `ApiRequestTarget` and the `String` into extensions (still needed by `error.rs`). Order in `main.rs` must be: `TracingLogger` (outermost) → `RequestId` → `RequireUser` (scope‑local). | `backend/src/main.rs`, `backend/src/request_id.rs` | `X-Request-Id` header on the response equals the `request_id` field in the matching log line. |
| 1.4 | In `auth/middleware.rs::RequireUser::call`, after the session is validated, do `tracing::Span::current().record("user_id", &tracing::field::display(&user.id));`. | `backend/src/auth/middleware.rs` | Authenticated requests' root span carries `user_id`. |
| 1.5 | Replace the `.unwrap()` in `request_id.rs` header insertion with `.expect("request id is a valid HeaderValue")`. | `backend/src/request_id.rs` | `cargo clippy` clean. |

**Validation**: issue three different requests (anonymous `/auth/login`, authenticated `/api/v1/users/me`, a deliberately failing `/api/v1/blobs/does-not-exist`) and verify each produces **one** root span line whose `request_id` matches the response header and contains the expected fields.

---

### Phase 2 — Production‑grade formatter (S, ~2h)

Goal: structured JSON in production; human‑friendly output in dev. No secrets on stdout.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 2.1 | In `observability::init`, branch on the same `WORSHIP_PRODUCTION` / `RUST_ENV` rules already used in `main.rs`. Production = `.json().flatten_event(true).with_current_span(true).with_span_list(false)`; dev = `.compact()`. Re‑enable `.with_target(true)` in both — suppressing the target was a loss, not a gain. | `backend/src/observability.rs` | `WORSHIP_PRODUCTION=true cargo run` emits one JSON object per line; `jq .` parses every line. |
| 2.2 | Add a `LOG_FORMAT` env override (`json` \| `compact` \| `pretty`) that wins over the auto‑detection, for parity with local debugging of production issues. | `backend/src/observability.rs` | `LOG_FORMAT=json cargo run` without `WORSHIP_PRODUCTION` still emits JSON. |
| 2.3 | Manually implement `Debug` for `Settings` that redacts `otp_pepper`, `gmail_app_password`, `apple_client_secret`, `oidc_client_secret`, `db_password` (print `"<redacted>"`). | `backend/src/settings.rs` | `format!("{:?}", Settings::default())` never contains the literal value of the pepper. Add a unit test. |
| 2.4 | Remove the `#[derive(Deserialize, Debug)]` `Debug` on `Settings`, replace with `#[derive(Deserialize)]` + hand‑rolled `Debug` from 2.3. | `backend/src/settings.rs` | `cargo build` green. |

**Validation**: `WORSHIP_PRODUCTION=true OTP_PEPPER=hunter2 cargo run` — log output may contain settings dumps added in later phases, but `grep -F hunter2` on stdout returns nothing.

---

### Phase 3 — Instrumentation of service and auth layers (M, ~1 day)

Goal: every fallible business method produces an entry/exit span with error diagnostics, without cluttering the bodies.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 3.1 | Add `#[tracing::instrument(level = "debug", skip(self, …), err)]` to every `pub async fn` in the following service files. Redact large payloads via `skip(…)`, record ids as fields. | `backend/src/resources/blob/service.rs`, `…/collection/service.rs`, `…/song/service.rs`, `…/setlist/service.rs`, `…/team/service.rs`, `…/team/invitation/service.rs`, `…/user/service.rs`, `…/user/session/service.rs` | Every failing service call emits exactly one `ERROR` line (via `err`) whose `error` field matches the `AppError`. |
| 3.2 | Add `#[tracing::instrument(skip_all, fields(provider = %provider, …), err)]` on `auth::oidc::rest::{login, callback}` and `auth::otp::rest::{otp_request, otp_verify}`. In `otp_request`, explicitly do **not** include the generated code in fields. | `backend/src/auth/oidc/rest.rs`, `backend/src/auth/otp/rest.rs` | A successful login flow produces: `login` span (with provider), `callback` span (with provider + user_id), and no code/secret value anywhere on the log stream. |
| 3.3 | Instrument `MailService::send` with `skip(self, body)` and fields `to = %to, subject = subject, transport_ok = Empty`. On success record `transport_ok = true`, on `is_positive()` failure record `false` and log at `warn!`. | `backend/src/mail.rs` | Sending a test OTP emits one span with `transport_ok=true` and no body content. |
| 3.4 | Instrument `SurrealTeamResolver`, `database::Database::connect`, and `database::Database::migrate`. `connect` logs resolved namespace/database but not credentials. | `backend/src/resources/team/resolver.rs`, `backend/src/database/mod.rs` | Startup emits a `database.connected` span with `namespace`, `database`, `has_credentials=true/false`. |

**Validation**: run the backend, perform an OTP flow that fails (wrong code), confirm each layer (`rest` → `service` → `repo`) contributes one log line and all share the same `request_id`.

---

### Phase 4 — Error source chains (S, ~3h)

Goal: when something breaks, the log tells you *why* all the way down.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 4.1 | Extract `log_surreal_error_chain` from `migrations.rs` into `backend/src/observability.rs` as a generic `pub fn log_error_chain(target: &'static str, err: &(dyn std::error::Error + 'static))` that joins `source()` with `" <- "` and also records `?err`. Reuse it from `migrations.rs`. | `backend/src/observability.rs`, `backend/src/database/migrations.rs` | Existing migration error output is byte‑identical to before. |
| 4.2 | In `AppError::error_response`, when the variant is `Internal`, also log `error.code = self.code()` and the chain via 4.1's helper. Promote the log from `error!("{}", self)` to `error!(error.code = self.code(), error = %self, "internal error")`. | `backend/src/error.rs` | A forced `AppError::Internal("boom")` produces a log line with `error.code="internal"`. |
| 4.3 | At every `.map_err(AppError::database)` / `::mail` / `::oidc` site, replace with `.map_err(\|err\| { tracing::error!(error = %err, error_debug = ?err, "…"); AppError::database(err) })`. Create a small helper macro `log_and_convert!(AppError::database, "database call failed")` to avoid repetition. | All `map_err(AppError::…)` sites (see §3.6 list in the review). | Failures in OIDC, mail, DB construction paths now produce a log entry **before** returning the sanitized `AppError`. |
| 4.4 | Decide whether to extend `AppError::Internal` to carry `anyhow::Error` (preserves `source()` for callers and for `error_response` to walk). If adopted, deprecate string‑only `Internal` and migrate constructors. Mark as follow‑up if it balloons. | `backend/src/error.rs`, callers | Optional; if done, update the review appendix. |

**Validation**: force a mail send failure (e.g. invalid SMTP creds in a staging env). Log output must show the lettre error chain, not just "mail error: …".

---

### Phase 5 — Close silent‑swallow sites (S, ~3h)

Goal: no I/O boundary discards errors.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 5.1 | `FsBlobStorage::delete_blob_file`: convert to `Result<(), AppError>` or keep `()` but `warn!(blob_id = %blob.id, path = %path.display(), error = %err, "failed to delete blob file")` on failure. Also downgrade `ErrorKind::NotFound` to `debug!` (idempotent delete). | `backend/src/resources/blob/storage.rs` | Deleting a non‑existent blob produces a `debug` line; a permissions failure produces a `warn`. |
| 5.2 | `SurrealUserRepo::set_default_collection` / `clear_default_collection`: drop the `let _ =` and also run `response.check()` plus `response.take_errors()` like `migrations.rs` does. Fail loudly if any statement errored. | `backend/src/resources/user/surreal_repo.rs` | Forcing a schema violation in this path returns an error and logs it. |
| 5.3 | `SurrealCollectionRepo::add_song_to_collection`: same treatment as 5.2. | `backend/src/resources/collection/surreal_repo.rs` | Same. |
| 5.4 | Grep for `let _ = … .await?` across `backend/src/**/*surreal_repo.rs`; apply 5.2 pattern anywhere the discarded binding is a `surrealdb::Response`. | all `surreal_repo.rs` files | No surreal `Response` is dropped without `take_errors()` being inspected. |
| 5.5 | `RequireUser::call`: log `debug!(reason = "missing_session", …)` or `debug!(reason = "expired_session", …)` before returning `AppError::unauthorized()`. Leave as `debug` to avoid log spam from unauthenticated health probes. | `backend/src/auth/middleware.rs` | Running `curl / -H 'Cookie: sso_session=bogus'` at `RUST_LOG=debug` shows the reason. |

**Validation**: add a temporary test that creates a bad record via 5.2 path; it must now fail with a log line instead of silently succeeding.

---

### Phase 6 — Audit events (M, ~1 day)

Goal: a stable, greppable event name for every security‑sensitive state change. One `info!(event = "audit.…", …, "…")` per outcome.

Event catalog to introduce:

| Event | Where to emit | Required fields |
|-------|---------------|-----------------|
| `audit.auth.login.success` | `auth::oidc::rest::callback` success path, `auth::otp::rest::otp_verify` success path | `provider`, `user_id`, `session_id` |
| `audit.auth.login.failure` | same, error paths | `provider`, `reason`, `email_hash` (never raw email in audit if possible) |
| `audit.auth.otp.requested` | `otp_request` end | `email_domain` (not full email), `delivered` |
| `audit.auth.logout` | `auth::rest::logout` | `session_id` (cookie or bearer), `had_cookie` |
| `audit.session.created` | `SessionService::create_session` | `session_id`, `user_id`, `ttl_seconds` |
| `audit.session.revoked` | `SessionService::delete_session*` | `session_id`, `user_id`, `actor_user_id` |
| `audit.user.created` | `UserServiceHandle::create_user` | `user_id`, `email`, `role` |
| `audit.user.deleted` | `UserServiceHandle::delete_user` | `user_id`, `actor_user_id` |
| `audit.team.role.changed` | team/invitation service | `team_id`, `target_user_id`, `old_role`, `new_role`, `actor_user_id` |
| `audit.team.invitation.accepted` | `team::invitation::service` | `team_id`, `invitation_id`, `user_id` |
| `audit.rate_limit.rejected` | actix‑governor error handler | `route`, `client_ip`, `user_id` (if available) |

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 6.1 | Add a thin helper module `backend/src/observability/audit.rs` exposing `fn audit(event: &'static str, fields: …)`; or just `macro_rules! audit { … }` that expands to `info!(event = $event, audit = true, …)`. | new file | All audit entries share `audit=true` and a stable `event`. |
| 6.2 | Place emission calls at the sites listed in the catalog above. | services + `auth/*` | An end‑to‑end OTP login produces: `audit.auth.otp.requested` → `audit.auth.login.success` → `audit.session.created`. |
| 6.3 | Register a custom `actix-governor` error handler that emits `audit.rate_limit.rejected` and returns the existing Problem Details 429. | `backend/src/auth/rest.rs` (or wherever `Governor::new` is built) | Triggering the 1 rps rate limit produces one audit entry per rejection. |
| 6.4 | Document the event list as a table in `docs/architecture/backend-request-flow.md` so the catalog stays close to the code it describes. | `docs/architecture/backend-request-flow.md` | Docs match reality. |

**Validation**: run the full functional venom suite (`backend/tests/*.yml`) and assert (e.g. via `jq 'select(.audit==true) \| .event'`) that every sensitive flow emitted the expected audit event exactly once per user‑visible outcome.

---

### Phase 7 — Startup summary (S, ~1h)

Goal: one log entry per process lifetime that captures the operational configuration.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 7.1 | After `Settings::from_env()?` and the production‑check block in `main.rs`, emit `info!(event = "startup", host = %settings.host, port = settings.port, cookie_secure = settings.cookie_secure, session_ttl_seconds, otp_ttl_seconds, otp_allow_self_signup, otp_max_attempts, auth_rate_limit_rps, auth_rate_limit_burst, blob_upload_max_bytes, oidc_providers = ?enabled_providers, production, "backend starting");`. Secrets excluded by construction. | `backend/src/main.rs` | Startup log contains every operational knob the reader asked about in production; grep for `otp_pepper` returns nothing. |
| 7.2 | Remove the `info!("Serving static files from …")` and `info!("Starting server on …")` ad‑hoc lines (redundant with 7.1) or consolidate them. | `backend/src/main.rs` | Boot logs are the single `event="startup"` line plus per‑migration entries. |
| 7.3 | In `auth::oidc::build_clients`, emit `info!(event = "oidc.provider.registered", provider = %provider, issuer = %issuer_url, scopes = ?scopes)` for each provider. | `backend/src/auth/oidc/client.rs` | Enabling Apple in dev shows two `oidc.provider.registered` entries. |

**Validation**: compare `docker run` output before/after; a single line should now describe the whole operational surface.

---

### Phase 8 — Developer experience & tests (M, ~½ day)

Goal: make logging behavior part of the contract, not folklore.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 8.1 | Add `tracing-test = "0.2"` as a `dev-dependency` and one canary test per audit event. Minimum viable: a test per event name that exercises the happy path and asserts a single matching entry via `tracing_test::traced_test`. | `backend/Cargo.toml`, new tests under each service module | Running `cargo test -p backend` passes; removing an `audit!` call makes the matching test fail. |
| 8.2 | Document logging in `docs/architecture/backend-request-flow.md`: subscriber setup, `RUST_LOG` examples (`backend=debug,surrealdb=info`), `LOG_FORMAT`, request‑id propagation contract (`traceparent` → `X-Request-Id` → `request_id` span field), audit event catalog location. | `docs/architecture/backend-request-flow.md` | New contributors can answer "how do I see what happened for request X in prod?" from the docs alone. |
| 8.3 | Update `README.md`'s "Start the Backend" block with a paragraph on `RUST_LOG` and `LOG_FORMAT=json`. | `README.md` | README reflects the new knobs. |

**Validation**: a reviewer unfamiliar with the project can follow the docs and, within 5 minutes of a reproducible request, point to the exact log entries for it.

---

### Phase 9 — Longer‑term (optional, L)

Only if/when the operational needs demand it. None of these block earlier phases.

| # | Task | Notes |
|---|------|-------|
| 9.1 | Adopt `tracing-opentelemetry` + OTLP exporter. The existing `traceparent` handling already gives you most of W3C tracing; add the layer, and spans become traces. |
| 9.2 | Switch to `secrecy::Secret<String>` for every secret in `Settings`. Stronger than a custom `Debug` because accidental `format!("{:?}", secret)` prints `"[REDACTED]"` without opt‑in. |
| 9.3 | Use `tracing-appender::non_blocking` with a rolling file writer if/when log volume pressures the async runtime. |
| 9.4 | Add structured metrics via `tracing::instrument` → `metrics` crate bridge, or a `prometheus` endpoint. Out of scope for pure logging, but often part of the same refactor. |

---

### Milestones & ordering

```
Phase 0 ── prerequisites
   │
   ├──► Phase 1 ── request correlation           ← unblocks every later log line
   │       │
   │       ├──► Phase 2 ── json formatter + redacted Settings Debug
   │       │
   │       ├──► Phase 3 ── #[instrument] sweep
   │       │        │
   │       │        └──► Phase 4 ── error source chains
   │       │
   │       ├──► Phase 5 ── close silent‑swallow sites
   │       │
   │       ├──► Phase 6 ── audit event catalog
   │       │
   │       └──► Phase 7 ── startup summary
   │
   └──► Phase 8 ── tests + docs (touches everything, do last or parallel)

Phase 9 ── optional, any time
```

Phases 0–2 together give you correlated, structured, redacted logs — the minimum viable modern observability stack — in roughly **one engineering day**. Phases 3–7 are the delta from "observable" to "auditable" and are each independently valuable. Phase 8 locks everything in via tests and docs so future contributions don't regress.

### Definition of done (for the whole plan)

- Every HTTP request emits a single root span with `request_id`, `method`, `route`, `status`, `latency_ms`, and `user_id` when authenticated.
- Every `AppError::Internal` variant logged at the response boundary has an associated source‑chain log line at the construction site.
- No `let _ = … .await?` on a `surrealdb::Response` anywhere in the tree.
- Every sensitive state change emits exactly one `audit.*` event with `audit=true`.
- Running `WORSHIP_PRODUCTION=true` produces newline‑delimited JSON; running without produces human‑readable compact output.
- `grep` for any secret env value across the full log stream of the venom test suite returns zero matches.
- `docs/architecture/backend-request-flow.md` documents the audit event catalog, field vocabulary (§5), and `RUST_LOG` / `LOG_FORMAT` knobs.

---

### Suggested PR bundles

The 10 phases above are independently shippable, but several of them touch overlapping files or share a single reviewer concern. Bundling reduces file churn, review load, and test re‑runs. The groupings below are ranked by "savings vs. shipping the phases separately."

#### Bundle A — Foundation (Phases 0 + 1 + 2)

- **Why together.** All three changes live in the same two files (`backend/src/observability.rs` and `backend/src/main.rs`) plus one tiny edit to `request_id.rs` / `settings.rs`. No business‑logic code is modified, so the reviewer reasons about subscriber wiring once. Phase 1 structurally requires the `tracing-actix-web` dependency from Phase 0, and Phase 2's formatter lives in the same `init()` function Phase 0 creates — splitting them means editing `observability::init` three times.
- **Combined scope (one PR, ~½–1 day).**
  - New `backend/src/observability.rs` with `init()`: env filter + `tracing_log::LogTracer::init()` + JSON/compact switch + `LOG_FORMAT` override.
  - `main.rs` swaps `Logger::new(...)` → `TracingLogger::<WorshipRootSpan>`, reorders middleware, keeps the `RequestId` extension insertions.
  - `request_id.rs` gains `WorshipRootSpan: RootSpanBuilder` (honors `traceparent`, records `status`/`latency_ms` on response) and replaces the `unwrap()` with `expect`.
  - `auth/middleware.rs::RequireUser` records `user_id` on the current span.
  - `settings.rs` swaps the derived `Debug` for a hand‑rolled one that redacts secrets.
- **Payoff.** Every log line in the crate — existing and future — inherits `request_id`/`user_id` automatically. This is the highest value‑per‑effort bundle and unlocks every later phase.

#### Bundle B — Error‑path quality (Phases 4 + 5)

- **Why together.** Both phases are "at I/O boundaries, stop throwing information away." Phase 4 generalizes `log_surreal_error_chain` into a helper and calls it at every `AppError::{database,mail,oidc}` construction site; Phase 5 reuses the exact same `response.take_errors()` / `response.check()` pattern from `migrations.rs` at the `let _ = db.query(...).await?` sites. Reviewing them in one pass means reviewing *one* rulebook for failing I/O results ("log the typed error with its chain, then convert").
- **Combined scope (one PR, ~½ day).**
  - Add `observability::log_error_chain(target, err)` and retrofit `migrations.rs` to use it.
  - Update `error.rs` so `error_response` logs `error.code` on `AppError::Internal`.
  - Replace `.map_err(AppError::…)` at every construction site with a `log_and_convert!` macro (or equivalent helper) that logs first.
  - Fix `FsBlobStorage::delete_blob_file`, `SurrealUserRepo::{set,clear}_default_collection`, `SurrealCollectionRepo::add_song_to_collection`, and `RequireUser` unauthorized reasons.
- **Payoff.** Nicer after Bundle A because every new log line carries `request_id`, but not strictly required. Can ship standalone.

#### Bundle C — Per‑method observability (Phases 3 + 6 + 7)

- **Why together.** Phase 3 (`#[tracing::instrument]`) is an attribute *on* the same service/auth functions where Phase 6 (`audit!(...)`) adds calls *inside*. Unbundled, you edit every `service.rs` and every `auth/*/rest.rs` twice. Phase 7 (startup summary) is a handful of lines in `main.rs` and `auth/oidc/client.rs` and fits under the same theme: "stable structured events for the things an operator cares about."
- **Combined scope (one PR, ~1–1½ days).**
  - Introduce `macro_rules! audit!` (or `observability::audit(...)`) enforcing `event="audit.*"` and `audit=true`.
  - Walk every `service.rs` under `resources/**`, every `auth/*/rest.rs`, and `mail.rs`: add `#[instrument(skip(...), err)]` at the top, drop `audit!(...)` calls on the right outcomes (login success/failure, session create/revoke, user/team mutations, …).
  - Register an `actix-governor` error handler that emits `audit.rate_limit.rejected` and returns the existing 429 Problem Details.
  - `main.rs` gains a single `event="startup"` info line; `auth::oidc::build_clients` gains `event="oidc.provider.registered"` per provider.
- **Optional split.** If the diff is intimidating, do Phase 3 first (pure attributes, minimal behavior change, easy to review) and ship Phase 6 + 7 as a follow‑up PR.

#### Bundle D — Lockdown (Phase 8)

- **Why on its own.** Phase 8 (`tracing-test` canaries + docs) needs Phases 3 and 6 to exist before it can meaningfully assert anything. It also touches many test files and the docs tree, and benefits from being reviewed separately from code changes.
- **Combined scope.** One PR per audit event family is fine; a single docs PR against `docs/architecture/backend-request-flow.md` and `README.md`.

#### What *not* to bundle

- **Don't bundle Phase 0 with Phase 3 or Phase 6.** Phase 0 is "library plumbing"; Phases 3 and 6 are "touch 20 business‑logic files." Mixing the two produces a PR that is intimidating to review and risky to revert.
- **Don't bundle Phase 2's JSON formatter with Phase 6's audit events.** Tempting because both are "production observability," but JSON formatting is a single‑file config change while audit events span the service layer. Keeping them separate makes a revert cheap if the JSON schema breaks a downstream dashboard.
- **Don't bundle Phase 9 with anything.** It is optional, each sub‑item (OpenTelemetry, `secrecy`, metrics, rolling appenders) is its own project, and most are research spikes.

#### Recommended merge order

```
PR1: Bundle A (0 + 1 + 2)   ~1 day       unlocks everything else
PR2: Bundle B (4 + 5)       ~½ day       visible error‑path wins
PR3: Bundle C (3 + 6 + 7)   ~1½ days     split into 3 / 6+7 if needed
PR4: Bundle D (8)                        after PR3 is merged
PR5+: Phase 9 items                      individually, as appetite dictates
```

If only one bundle fits the current iteration, pick **Bundle A**: it raises the value of every log line — existing and future — without touching business logic.

