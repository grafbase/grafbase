#
# === Build image ===
#
FROM rust:1.89-bookworm AS chef

COPY rust-toolchain.toml rust-toolchain.toml
RUN cargo install --locked cargo-chef sccache
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

WORKDIR /grafbase

FROM chef AS planner
# At this stage we don't really bother selecting anything specific, it's fast enough.
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV CARGO_INCREMENTAL=0
COPY --from=planner /grafbase/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY ./crates ./crates
COPY ./cli ./cli
COPY ./gateway ./gateway

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release --bin grafbase-gateway

#
# === Final image ===
#
FROM cgr.dev/chainguard/wolfi-base:latest

LABEL org.opencontainers.image.url='https://grafbase.com' \
    org.opencontainers.image.documentation='https://grafbase.com/docs' \
    org.opencontainers.image.source='https://github.com/grafbase/grafbase/tree/main/gateway' \
    org.opencontainers.image.vendor='Grafbase' \
    org.opencontainers.image.description='The Grafbase GraphQL Federation Gateway' \
    org.opencontainers.image.licenses='MPL-2.0'

WORKDIR /grafbase

# used curl to run a health check query against the server in a docker-compose file
RUN apk add --no-cache curl

RUN adduser -D -u 1000 grafbase && mkdir -p /data && chown grafbase:grafbase /data
USER grafbase

COPY --from=builder /grafbase/target/release/grafbase-gateway /bin/grafbase-gateway
COPY --from=builder /grafbase/crates/federated-server/config/grafbase.toml /etc/grafbase.toml

VOLUME /data
WORKDIR /data

ENTRYPOINT ["/bin/grafbase-gateway"]
CMD ["--config", "/etc/grafbase.toml", "--listen-address", "0.0.0.0:5000"]

EXPOSE 5000
