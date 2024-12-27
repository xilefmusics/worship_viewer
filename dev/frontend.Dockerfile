FROM rust:1.83.0-bookworm

RUN export CARGO_BUILD_JOBS=$(nproc) && \
    cargo install cargo-binstall && \
    cargo binstall trunk --version 0.21.5 --no-confirm && \
    rustup target add wasm32-unknown-unknown
