FROM alpine/git:v2.49.1 AS dependencydownloader

WORKDIR /fancy_yew
RUN git clone --depth 1 --branch 0.6.3 https://github.com/xilefmusics/fancy_yew.git .

FROM rust:1.91.1-slim AS builder

COPY --from=dependencydownloader /fancy_yew /fancy_yew
COPY --from=dependencydownloader /chordlib /chordlib

RUN export CARGO_BUILD_JOBS=$(nproc) && \
    cargo install cargo-binstall && \
    cargo binstall trunk --version 0.21.14 --no-confirm && \
    rustup target add wasm32-unknown-unknown && \
    apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev build-essential ca-certificates curl && \
    VENOM_VERSION=1.2.0 && \
    curl -L "https://github.com/ovh/venom/releases/download/v${VENOM_VERSION}/venom.linux-amd64" -o /usr/local/bin/venom && \
    chmod +x /usr/local/bin/venom

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

FROM scratch AS tester

# runtime libraries required for backend and Venom
COPY --from=builder /lib/x86_64-linux-gnu/libdl.so.2 /lib/x86_64-linux-gnu/libdl.so.2
COPY --from=builder /lib/x86_64-linux-gnu/libpthread.so.0 /lib/x86_64-linux-gnu/libpthread.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/libm.so.6
COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=builder /lib/x86_64-linux-gnu/librt.so.1 /lib/x86_64-linux-gnu/librt.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/libc.so.6
COPY --from=builder /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
COPY --from=builder /usr/lib/x86_64-linux-gnu/libssl.so.3 /usr/lib/x86_64-linux-gnu/libssl.so.3
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcrypto.so.3 /usr/lib/x86_64-linux-gnu/libcrypto.so.3
COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libzstd.so.1 /usr/lib/x86_64-linux-gnu/libzstd.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libstdc++.so.6 /usr/lib/x86_64-linux-gnu/libstdc++.so.6
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# shell & utilities to orchestrate tests
COPY --from=builder /bin/sh /bin/sh
COPY --from=builder /bin/sleep /bin/sleep

SHELL ["/bin/sh", "-c"]

COPY --from=builder /usr/local/bin/venom /usr/local/bin/venom
COPY --from=builder /wrk/backend/tests /app/tests
COPY --from=builder /wrk/backend/target/release/backend /app/worship_viewer
COPY --from=builder /wrk/backend/surrealdb /app/surrealdb
COPY --from=builder /wrk/frontend/dist/ /app/static

WORKDIR /app

ENV INITIAL_ADMIN_USER_EMAIL="admin@example.com" \
    INITIAL_ADMIN_USER_TEST_SESSION=true

RUN set -eux; \
    ./worship_viewer & \
    backend_pid=$!; \
    trap "kill $backend_pid 2>/dev/null || true" EXIT; \
    sleep 5; \
    /usr/local/bin/venom run /app/tests/*.yml; \
    kill $backend_pid; \
    wait $backend_pid 2>/dev/null || true

FROM scratch

COPY --from=builder /lib/x86_64-linux-gnu/libdl.so.2 /lib/x86_64-linux-gnu/libdl.so.2
COPY --from=builder /lib/x86_64-linux-gnu/libpthread.so.0 /lib/x86_64-linux-gnu/libpthread.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/libm.so.6
COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=builder /lib/x86_64-linux-gnu/librt.so.1 /lib/x86_64-linux-gnu/librt.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/libc.so.6
COPY --from=builder /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
COPY --from=builder /usr/lib/x86_64-linux-gnu/libssl.so.3 /usr/lib/x86_64-linux-gnu/libssl.so.3
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcrypto.so.3 /usr/lib/x86_64-linux-gnu/libcrypto.so.3
COPY --from=builder /usr/lib/x86_64-linux-gnu/libz.so.1 /usr/lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libzstd.so.1 /usr/lib/x86_64-linux-gnu/libzstd.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libstdc++.so.6 /usr/lib/x86_64-linux-gnu/libstdc++.so.6
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY --from=tester /app/worship_viewer /app/worship_viewer
COPY --from=builder /wrk/backend/surrealdb/ /app/surrealdb
COPY --from=builder /wrk/frontend/dist/ /app/static

EXPOSE 8080
WORKDIR /app
ENTRYPOINT ["/app/worship_viewer"]
