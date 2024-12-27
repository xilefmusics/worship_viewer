FROM rust:1.83.0-bookworm

RUN export CARGO_BUILD_JOBS=$(nproc) &&\
    cargo install cargo-watch
