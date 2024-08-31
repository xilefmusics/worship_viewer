FROM bitnami/git:2.43.0-debian-11-r4 as DependencyDownloader

WORKDIR /fancy_surreal
RUN git clone --depth 1 --branch 0.1.4 https://github.com/xilefmusics/fancy_surreal.git .

WORKDIR /fancy_yew
RUN git clone --depth 1 --branch 0.4.0 https://github.com/xilefmusics/fancy_yew.git .

WORKDIR /chordlib
RUN git clone --depth 1 --branch 0.1.0 https://github.com/xilefmusics/chordlib.git .

FROM rust:1.79.0-bookworm as builder

COPY --from=DependencyDownloader /fancy_surreal /fancy_surreal
COPY --from=DependencyDownloader /fancy_yew /fancy_yew
COPY --from=DependencyDownloader /chordlib /chordlib

RUN export CARGO_BUILD_JOBS=$(nproc) && \
    cargo install cargo-binstall && \
    cargo binstall trunk --version 0.20.3 --no-confirm && \
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

COPY --from=builder /wrk/backend/target/release/backend /app/worship_viewer
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
