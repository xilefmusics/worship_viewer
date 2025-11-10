FROM alpine/git:v2.49.1 AS dependencydownloader

WORKDIR /fancy_yew
RUN git clone --depth 1 --branch 0.6.3 https://github.com/xilefmusics/fancy_yew.git .

WORKDIR /chordlib
RUN git clone --depth 1 --branch 0.4.8 https://github.com/xilefmusics/chordlib.git .

FROM rust:1.91.0-slim AS builder

COPY --from=dependencydownloader /fancy_yew /fancy_yew
COPY --from=dependencydownloader /chordlib /chordlib

RUN export CARGO_BUILD_JOBS=$(nproc) && \
    cargo install cargo-binstall && \
    cargo binstall trunk --version 0.21.14 --no-confirm && \
    rustup target add wasm32-unknown-unknown && \
    apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev build-essential ca-certificates

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
COPY --from=builder /wrk/backend/surrealdb/ /app/surrealdb
COPY --from=builder /wrk/frontend/dist/ /app/static

ENTRYPOINT ["/app/worship_viewer"]