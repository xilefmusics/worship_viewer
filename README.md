# Worship Viewer

**Worship Viewer** is an app for managing and displaying digital sheet music.
It allows users to import entire music books as collections, providing a digital table of contents with corresponding metadata that is searchable.

## Setup Devlopment environment

```bash
# install all prerequesites
brew install rustup                         # Tool to manage rust tool chains
rustup update stable                        # Install newest tool chain
rustup target add wasm32-unknown-unknown    # Add webassembly support
cargo install trunk                         # Install the tool to bundle the webassembly frontend
brew install caddy                          # Optional: Install reverse proxy

# start the backend
cd backend && INITIAL_ADMIN_USER_EMAIL="admin@example.com" INITIAL_ADMIN_USER_TEST_SESSION=true cargo run
# The initial admin session does have the ID admin.
# You can use it to authenticate as cookie sso_session or as bearer token

# start the frontend
cd frontend && trunk serve --port 8081 

# serve backend & frontend under the same port
echo '{"apps":{"http":{"servers":{"srv":{"listen":[":8082"],"routes":[{"match":[{"path":["/api*"]}],"handle":[{"handler":"reverse_proxy","upstreams":[{"dial":"localhost:8080"}]}]},{"handle":[{"handler":"reverse_proxy","upstreams":[{"dial":"localhost:8081"}]}]}]}}}}}' | caddy run --config -
```

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
cd backend \
  && INITIAL_ADMIN_USER_EMAIL="admin@example.com" \
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
cd frontend && trunk serve --port 8081
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
