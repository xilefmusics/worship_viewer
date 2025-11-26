# Worship Viewer

**Worship Viewer** is an app for managing and displaying digital sheet music.
It allows users to import entire music books as collections, providing a digital table of contents with corresponding metadata that is searchable.

## Setup Development Environment

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

- The initial admin session has the ID: \`admin\`
- Authentication can be done via:
  - Cookie: \`sso_session\`
  - Bearer token

### Start the Frontend

```bash
cd frontend && \
    trunk serve --port 8081
```

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
docker run --rm -p 8000:8000 surrealdb/surrealdb:v2.4.0-dev start --log debug --user root --pass root memory
./surreal import --conn http://localhost:8000 --user root --pass root --ns app --db app ./worshipviewer-2025-11-25.surql
./surreal sql --conn http://127.0.0.1:8000 --user root --pass root --ns app --db app

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