FROM rust:1.54-alpine3.14 AS BackendBuilder
RUN apk update
RUN apk add openssl-dev=1.1.1q-r0
RUN apk add ncurses-dev=6.2_p20210612-r1
RUN apk add alpine-sdk=1.0-r1
COPY ./backend /wrk
WORKDIR /wrk
RUN rustup override set nightly-2021-06-01
RUN cargo build --release

FROM node:lts-alpine3.14 AS FrontendBuilder
COPY ./frontend /wrk
WORKDIR /wrk
RUN npm install
RUN npm run build-www

FROM scratch
COPY --from=BackendBuilder /wrk/target/release/worship_viewer .
COPY --from=FrontendBuilder /wrk/www ./www
VOLUME ["/songs"]
CMD ["/worship_viewer", "server", "-w", "/www", "/songs"]
