FROM rust:1.70.0-bookworm as builder

RUN cargo install --locked trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /wrk
COPY ./worship_viewer_shared ./worship_viewer_shared

WORKDIR /wrk
COPY ./worship_viewer_backend ./worship_viewer_backend
WORKDIR /wrk/worship_viewer_backend
RUN cargo build --release

WORKDIR /wrk
COPY ./worship_viewer_frontend ./worship_viewer_frontend
WORKDIR /wrk/worship_viewer_frontend
RUN trunk build --release

FROM ubuntu:22.04

COPY --from=builder /wrk/worship_viewer_backend/target/release/worship_viewer_backend /app/worship_viewer
COPY --from=builder /wrk/worship_viewer_frontend/dist/ /app/static

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
