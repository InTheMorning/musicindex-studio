# Runtime image of studio-server (ADR 0066).
# The build stage uses the Rust version of rust-toolchain.toml.

FROM rust:1.99.1-alpine AS chef

# These packages build the three C libraries: SQLite, OpenSSL and LAME.
RUN apk add --no-cache \
    build-base \
    cmake \
    linux-headers \
    perl \
    pkgconf \
 && cargo install cargo-chef --locked

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
 && mkdir -p /data \
 && chown studio:studio /data

WORKDIR /data
COPY --from=builder /build/target/release/studio-server /usr/local/bin/studio-server

USER studio

VOLUME /data
EXPOSE 8080

ENTRYPOINT ["studio-server"]
CMD ["serve", "--data-dir", "/data", "--listen", "0.0.0.0:8080"]
