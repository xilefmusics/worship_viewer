# Worship Viewer

A tool to helps you lead worship — then steps aside when the Spirit takes over.
It's main functionality is to manage and display digital sheet music, but there is a lot more to come.

## Main Principles

1. **Single Source Of Truth**: You have one source (your song definition) to render sheets, display slieds, sample click and cue tracks, ... Each member of your worship team has access to the exact same song entities. So as soon as you have your song every one has the exact same song in the format he needs it.
2. **Be prepared but have all the freedom**: It should be possible to plan your whole set to the resolution of a beat, but to break out of it whenever the Holy Spirit wants to take over. Or even start 100% spontanious session.
3. **All for His glory**: The whole purpose of this App is to worhip and glorify the one true God the Father, the Son and the Holy Spirt.

## Try It Out

Create your free account at [app.worshipviewer.com](https://app.worshipviewer.com).
Or run it locally:

```bash
docker run --rm -p 8080:8080 xilefmusics/worship-viewer:latest
```

## Contribute

This app is from worshippers for worshippers. You are free to contribute. Yes coding can be an act of worship as well.

### Install Prerequisites

```bash
# Install Rust toolchain manager
brew install rustup

# Install the latest stable Rust toolchain
rustup update stable

# Add WebAssembly compilation target
rustup target add wasm32-unknown-unknown

# Install Trunk (Wasm bundler for the frontend)
cargo install trunk

# (Optional) Install Caddy as a reverse proxy
brew install caddy
```

### Start the Backend

```bash
cd backend && \
  INITIAL_ADMIN_USER_EMAIL="admin@example.com" \
  INITIAL_ADMIN_USER_TEST_SESSION=true \
  cargo run
```

Notes:

- The initial admin session has the ID: admin
- Authentication can be done via:
  - Cookie: sso_session
  - Bearer token

**Logs:** The backend uses [`tracing`](https://docs.rs/tracing). Set [`RUST_LOG`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) to control verbosity (for example `RUST_LOG=backend=debug,surrealdb=info`). Use `LOG_FORMAT=json` for newline-delimited JSON on stdout (this is also the default when `WORSHIP_PRODUCTION=true` or `RUST_ENV=production`). Incoming `traceparent` may supply the span id used as `X-Request-Id` and the `request_id` field on the per-request span. See `docs/architecture/backend-request-flow.md` for the full logging and audit-event notes.

### Start the Frontend

```bash
cd frontend && \
    trunk serve --port 8081
```

The in-app song editor expects **ChordPro** text (via **chordlib**). **Ultimate Guitar is not fetched over HTTP** by chordlib anymore: paste ChordPro you already have, or download UG/tab HTML yourself and convert outside the app.

### Serve Backend & Frontend on the Same Port (Caddy Reverse Proxy)

```bash
echo '{
  "apps": {
    "http": {
      "servers": {
        "srv": {
          "listen": [":8082"],
          "routes": [
            {
              "match": [{"path": ["/api*"]}],
              "handle": [{
                "handler": "reverse_proxy",
                "upstreams": [{"dial": "localhost:8080"}]
              }]
            },
            {
              "handle": [{
                "handler": "reverse_proxy",
                "upstreams": [{"dial": "localhost:8081"}]
              }]
            }
          ]
        }
      }
    }
  }
}' | caddy run --config -
```

### You want the data to survive backend newstarts

```bash
# Start the database as a separate process
docker run --rm -p 8000:8000 surrealdb/surrealdb:v2.4.0-dev start --log debug --user root --pass root memory

# Start the backend to connect to the database
cd backend && \
  INITIAL_ADMIN_USER_EMAIL="admin@example.com" \
  INITIAL_ADMIN_USER_TEST_SESSION=true \
  DB_ADDRESS="ws://localhost:8000" \
  DB_NAMESPACE="app" \
  DB_DATABASE="app" \
  DB_USERNAME="app" \
  DB_PASSWORD="app" \
  cargo run
```

## Command-Line Interface (CLI)

You can also interact with the Worship Viewer REST API from the command line using an AI-first CLI called `worship-viewer`. It speaks the same API that the frontend uses and is designed to be easy to drive from scripts and AI agents.

### Installation

- **Prerequisite**: a recent Rust toolchain (see the steps in **Install Prerequisites** below).
- From the repository root, install the CLI globally:

```bash
cargo install --path cli
```

This will install a `worship-viewer` binary on your `$PATH`.

### Configuration

The CLI can be configured via a small config file, environment variables, or flags. Precedence is:

1. CLI flags
2. Environment variables
3. Config file
4. Built-in defaults

- **Config file (optional)**  
Location: `~/.worshipviewer/config.toml`
  ```toml
  base_url = "http://127.0.0.1:8080"
  sso_session = "admin"
  ```
- **Base URL** (backend address)
  - Flag: `--base-url`
  - Env: `WORSHIP_VIEWER_BASE_URL`
  - Config: `base_url` in `~/.worshipviewer/config.toml`
  - Default: `http://127.0.0.1:8080`
- **Authentication**
  - Cookie-based (recommended for local dev):
    - Backend uses the `sso_session` cookie.
    - Flag: `--sso-session`
    - Env: `WORSHIP_VIEWER_SSO_SESSION`
    - Config: `sso_session` in `~/.worshipviewer/config.toml`
    - The CLI sends `Cookie: sso_session=<value>`.
  - Bearer token:
    - Flag: `--bearer-token`
    - Env: `WORSHIP_VIEWER_BEARER_TOKEN`
    - The CLI sends `Authorization: Bearer <WORSHIP_VIEWER_BEARER_TOKEN>`.
- **Timeout**
  - Env: `WORSHIP_VIEWER_TIMEOUT_SECS`
  - Flag: `--timeout-secs`

### Output & AI-friendly behavior

The CLI always emits machine-readable JSON and is optimized for being called by tools or agents:

- Global flag: `--output auto|json|pretty|ndjson`
  - `auto` (default): pretty JSON when in a TTY, compact JSON when piped.
  - `json`: compact JSON.
  - `pretty`: human-friendly, pretty-printed JSON.
  - `ndjson`: one JSON object per line (best for large lists and streaming).

For scripting and AI agents, prefer `--output json` or `--output ndjson`.

### Common commands

Inspect the API schema exposed by the backend:

```bash
worship-viewer schema --output json
worship-viewer schema --path-prefix /api/v1/songs --output json
```

List and get songs:

```bash
worship-viewer songs list --output ndjson
worship-viewer songs get --id <song_id> --output json
```

Create or update a song using a raw JSON payload:

```bash
worship-viewer songs create \
  --json '{"not_a_song":false,"blobs":[],"data":{...}}' \
  --output json
```

Use dry-run to validate a mutating request without actually changing data:

```bash
worship-viewer songs update \
  --id <song_id> \
  --json '{...}' \
  --dry-run \
  --output json
```

### Auth quickstart for local development

When you start the backend locally as described below, it creates an initial admin session with the ID `admin` and uses the `sso_session` cookie for authentication.

For a quick local setup:

```toml
# ~/.worshipviewer/config.toml
base_url = "http://127.0.0.1:8080"
sso_session = "admin"
```

Then you can run:

```bash
worship-viewer songs list --output json
```

## License

[AGPL-3.0](LICENSE)