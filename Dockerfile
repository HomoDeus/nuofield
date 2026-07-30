# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.88
ARG DEBIAN_VERSION=bookworm

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo build --release --locked -p nuofield-server -p nuofield-cli

FROM debian:${DEBIAN_VERSION}-slim AS runtime

LABEL org.opencontainers.image.title="NuoField" \
      org.opencontainers.image.description="Self-hosted workspace for humans and AI agents" \
      org.opencontainers.image.source="https://github.com/HomoDeus/nuofield" \
      org.opencontainers.image.documentation="https://github.com/HomoDeus/nuofield#readme" \
      org.opencontainers.image.licenses="MIT"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl gosu \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 1000 nuofield \
    && useradd --system --uid 1000 --gid 1000 --home-dir /var/lib/nuofield \
       --create-home --shell /usr/sbin/nologin nuofield \
    && mkdir -p /var/lib/nuofield/data \
    && chown -R nuofield:nuofield /var/lib/nuofield

COPY --from=builder /build/target/release/nuofield-server /usr/local/bin/nuofield-server
COPY --from=builder /build/target/release/nuofield /usr/local/bin/nuofield
COPY --chmod=0755 docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

ENV NUOFIELD_BIND=0.0.0.0:3000 \
    NUOFIELD_DATA_DIR=/var/lib/nuofield/data \
    RUST_LOG=nuofield_server=info,tower_http=info

EXPOSE 3000
VOLUME ["/var/lib/nuofield/data"]

WORKDIR /var/lib/nuofield

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl --fail --silent http://127.0.0.1:3000/readyz || exit 1

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["/usr/local/bin/nuofield-server"]
