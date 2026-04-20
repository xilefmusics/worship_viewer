# README review — 2026-04-18

Review of the repository root [`README.md`](../../README.md) against the current codebase, with manual checks of the documented setup paths.

## 1. Implemented but not documented (or under-documented)

### Environment variables (backend)

[`Settings::from_env`](../../backend/src/settings.rs) loads configuration via `envy` (uppercase env names matching struct fields) plus one extra override:

| Area | Examples | Notes |
|------|----------|--------|
| HTTP server | `HOST`, `PORT` | README implies port `8080`; default `port` is 8080, but this is configurable. |
| Cookies / session | `POST_LOGIN_PATH`, `COOKIE_NAME`, `COOKIE_SECURE`, `SESSION_TTL_SECONDS` | Not mentioned; relevant for production and HTTPS. |
| Email OTP | `OTP_TTL_SECONDS`, `OTP_PEPPER`, `OTP_MAX_ATTEMPTS`, `WORSHIP_OTP_ALLOW_SELF_SIGNUP` | OTP and rate limits are real product behavior ([`docs/business-logic-constraints/authentication.md`](../business-logic-constraints/authentication.md)); README only covers the admin test session. |
| OIDC (Google) | `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_REDIRECT_URL`, `OIDC_SCOPES` | Defaults exist in code (Google issuer, empty client id); production login flows are not described. |
| Outbound mail | `GMAIL_APP_PASSWORD`, `GMAIL_FROM` | [`MailService`](../../backend/src/mail.rs) always targets Gmail SMTP; README does not say OTP email requires valid Gmail credentials (empty defaults may only be viable for dev paths that never send mail). |
| Static assets / blobs | `STATIC_DIR`, `BLOB_DIR`, `BLOB_UPLOAD_MAX_BYTES` | Docker/runtime layout uses `static` + migrations; local dev docs do not explain serving the SPA from the backend. |
| Rate limiting | `AUTH_RATE_LIMIT_RPS`, `AUTH_RATE_LIMIT_BURST`, `API_RATE_LIMIT_RPS`, `API_RATE_LIMIT_BURST` | Tuned in settings; absent from README. |
| OpenAPI metadata | `OPENAPI_CONTACT_EMAIL`, `OPENAPI_IMPRINT_URL` | Used for docs branding; not listed. |
| Production guard | — | [`main.rs`](../../backend/src/main.rs) refuses to start with `INITIAL_ADMIN_USER_TEST_SESSION` when `WORSHIP_PRODUCTION` or `RUST_ENV=production`; README does not warn operators. |

### Architecture / behavior

- **Single-origin frontend:** The Yew app uses `window.location.origin` as the API base ([`frontend/src/api/provider.rs`](../../frontend/src/api/provider.rs)). Running **only** `trunk serve` on port 8081 while the API listens on 8080 sends browser requests to `http://localhost:8081/...`, not the backend, unless a reverse proxy (e.g. Caddy) or Trunk proxy is configured. The README lists backend, then frontend, then Caddy as optional—**the minimal two-process flow is misleading** without Caddy or without building the SPA into `STATIC_DIR` and running a single backend process.
- **`wasm32` target:** Required for `trunk build` / `trunk serve`; README mentions adding the target (good). No note that the repo’s [Dockerfile](../../Dockerfile) pins Rust **1.94.1** and Trunk **0.21.14** for reproducible builds.
- **CLI:** [`WORSHIPVIEWER_OUTPUT`](../../cli/src/commands.rs) exists as an environment variable for the global `--output` flag; README documents flags only, not this env var. The CLI’s [`load_file_config`](../../cli/src/config.rs) may **create** `~/.worshipviewer/config.toml` with defaults on first access—worth a one-line note for onboarding.

### Prerequisites

- Instructions are **Homebrew-centric** (`brew install rustup`, `brew install caddy`). Linux or Windows contributors get no equivalent (e.g. `rustup` from rustup.rs, distro packages).

## 2. Documented but stale, inaccurate, or mismatched

- **CLI section:** “see the steps in **Install Prerequisites** below” — that section is **above** the CLI block, not below (wording error).
- **Docker quick start:** `docker run ... xilefmusics/worshipviewer:latest` — on **Apple Silicon (linux/arm64)** Docker reported *no matching manifest* for this image (image appears **amd64-only**). README should state platform requirements or `docker run --platform linux/amd64` where appropriate.
- **Typos in README (quality / trust):** e.g. “slieds”, “Spirt”, “worhip”, “spontanious”, “helps you lead” → grammar (“help”).
- **“A lot more to come”** under the tagline is vague; either remove or point to issues/roadmap if one exists.

### Verified vs code (not stale)

- ChordPro / chordlib and the Ultimate Guitar note align with dependencies and usage ([`shared/Cargo.toml`](../../shared/Cargo.toml), [`backend/Cargo.toml`](../../backend/Cargo.toml)).
- Logging env vars (`RUST_LOG`, `LOG_FORMAT`, `WORSHIP_PRODUCTION`, `RUST_ENV`) match [`observability.rs`](../../backend/src/observability.rs).
- Link [`docs/architecture/backend-request-flow.md`](../architecture/backend-request-flow.md) resolves in-repo.

## 3. Quick Start / Installation — manual verification

| Step | Result |
|------|--------|
| Backend: `cd backend` + `INITIAL_ADMIN_USER_EMAIL=...` `INITIAL_ADMIN_USER_TEST_SESSION=true` `cargo build` | **Succeeded** |
| Backend: short `cargo run` | Process started (`target/debug/backend`); terminated after a few seconds for the test |
| Frontend: `trunk build` in `frontend/` | **Failed** with `NO_COLOR=1` in the environment (`invalid value '1' for '--no-color'`). **Succeeded** after `env -u NO_COLOR trunk build` |
| CLI: `cargo install --path cli` from repo root | **Succeeded**; installs binary `worshipviewer` (crate name `worshipviewer-cli`) |
| `https://app.worshipviewer.com` | **Reached** (HTTP fetch returned page content) |
| `docker pull xilefmusics/worshipviewer:latest` | **Failed on arm64** (no manifest for `linux/arm64/v8`) |

**Not executed here:** Full `caddy run` with the pasted JSON (Caddy not verified in this run). The config shape matches Caddy 2 JSON apps style; still worth a maintainer re-check on their installed Caddy version.

**Broken / confusing onboarding (summary):** split backend + `trunk serve` without proxy; Docker on ARM without `--platform`; Trunk under `NO_COLOR=1` (common in CI and some terminals).

## 4. README structure and best-practice gaps

Aligned with common “Standard README” / “Awesome README” expectations:

| Topic | Current state | Suggestion |
|-------|----------------|------------|
| **Title + one-liner + badges** | Title and tagline present; no badges | Optional: CI, license badge, MSRV or Rust version |
| **Table of contents** | None | Useful given length (CLI, Docker, dev stack) |
| **Requirements** | Partial (Rust, Trunk, optional Caddy) | Add platform notes (Docker arch), Node not required (confirm), SurrealDB section already present |
| **Quick start** | Docker one-liner + long-form dev | Add “recommended dev paths”: (A) Caddy 8082, (B) single backend + built static, (C) Trunk with documented proxy |
| **Configuration** | Fragmented | Link or embed a generated env list from `Settings`, or point to a single `docs/` reference |
| **Contributing** | Short invite only | No `CONTRIBUTING.md` in repo; add guidelines, branch/PR expectations, `cargo fmt` / `clippy`, or link to team process |
| **License** | AGPL-3.0 + `LICENSE` | Accurate; file is AGPL text |
| **Security / support** | None | Optional: security contact, disclaimer for self-hosting |
| **Visual clarity** | Monolithic markdown | Section dividers, TOC, or a small diagram (request flow: browser → Caddy → API/static) would help |

## 5. Recommended follow-ups (for maintainers)

1. Fix the **frontend + API origin** story in README (Caddy as default dev path, or document `STATIC_DIR` + `trunk build` output copied into `backend/static`, or Trunk `[build.proxy]` if adopted).
2. Document **Docker platform** (amd64 vs arm64) and optional `--platform linux/amd64`.
3. Add a concise **environment variable** subsection or link to `Settings` / internal ops doc.
4. Correct **CLI “below”** → “above”, document **`WORSHIPVIEWER_OUTPUT`**, and mention **auto-created** `~/.worshipviewer/config.toml`.
5. Note **`NO_COLOR=1`** + Trunk incompatibility or upstream tracking issue.
6. Proofread typos; add **CONTRIBUTING** (or link) if the project welcomes PRs.

---

*Review produced by comparing README to `backend/src/settings.rs`, `backend/src/main.rs`, `backend/src/observability.rs`, `frontend/src/api/provider.rs`, `cli/src/config.rs`, `cli/src/commands.rs`, and the root `Dockerfile`, plus the command checks in §3.*
