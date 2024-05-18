FROM rust:1.78.0-bookworm as builder

RUN cargo install --locked trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /wrk
COPY ./shared ./shared

WORKDIR /wrk
COPY ./backend ./backend
WORKDIR /wrk/backend
RUN cargo build --release

WORKDIR /wrk
COPY ./frontend ./frontend
WORKDIR /wrk/frontend
RUN trunk build --release

FROM ubuntu:24.04

COPY --from=builder /wrk/backend/target/release/worship_viewer_backend /app/worship_viewer
COPY --from=builder /wrk/frontend/dist/ /app/static

ENV PORT="8000" \
    DB_HOST="db" \
    DB_PORT="8000" \
    DB_USER="root" \
    DB_PASSWORD="root" \
    DB_NAMESPACE="test" \
    DB_DATABASE="test" \
    STATIC_DIR="/app/static" \
    BLOB_DIR="/app/blobs"

VOLUME "/app/blobs"

ENTRYPOINT ["/app/worship_viewer"]
