# Runtime image of studio-server (ADR 0066).
# The build stage uses the Rust version of rust-toolchain.toml. The base image
# and cargo-chef have fixed versions (SEC-15). The digest is the digest of the
# image index of rust:1.99.1-alpine. Find it with
# `docker buildx imagetools inspect rust:VERSION-alpine`.

FROM rust:1.99.1-alpine@sha256:7e9e2d068a107a2adafd19930011178d14a78326053cd4c3a9f7ad3fd049557d AS chef

# These packages build the three C libraries: SQLite, OpenSSL and LAME.
RUN apk add --no-cache \
    build-base \
    cmake \
    linux-headers \
    perl \
    pkgconf \
 && cargo install cargo-chef --version 0.1.78 --locked

WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release -p studio-server --locked --recipe-path recipe.json

COPY . .
RUN cargo build --release -p studio-server --locked

# ── Runtime ────────────────────────────────────────────────────────────────────

FROM alpine:3.24@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6 AS studio-server

# The release workflow gives these two values. They come last, so they do not
# spoil the build cache.
ARG STUDIO_REVISION=unknown
ARG STUDIO_VERSION=unknown
LABEL org.opencontainers.image.revision=$STUDIO_REVISION \
      org.opencontainers.image.version=$STUDIO_VERSION

RUN apk add --no-cache ca-certificates \
 && addgroup -S studio \
 && adduser -S -G studio studio \
 && install -d -m 0700 -o studio -g studio /data

WORKDIR /data
COPY --from=builder /build/target/release/studio-server /usr/local/bin/studio-server

USER studio

VOLUME /data
EXPOSE 8080

ENTRYPOINT ["studio-server"]
CMD ["serve", "--data-dir", "/data", "--listen", "0.0.0.0:8080"]
