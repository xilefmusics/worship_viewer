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

## License

[![AGPL-3.0](https://img.shields.io/badge/License-AGPLv3-blue.svg)](LICENSE)