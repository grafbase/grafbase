#
# === Build image ===
#
FROM rust:1.82-bookworm AS chef

COPY rust-toolchain.toml rust-toolchain.toml
RUN cargo install cargo-chef
WORKDIR /grafbase

FROM chef AS planner
# At this stage we don't really bother selecting anything specific, it's fast enough.
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV CARGO_INCREMENTAL=0
COPY --from=planner /grafbase/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY ./gateway/crates ./gateway/crates
COPY ./cli ./cli
COPY ./graphql-introspection ./graphql-introspection
COPY ./graph-ref ./graph-ref
COPY ./graphql-lint ./graphql-lint
COPY ./gqlint ./gqlint
COPY ./engine/crates ./engine/crates
COPY ./workspace-hack ./workspace-hack

RUN cargo build --release --bin grafbase-gateway

#
# === Final image ===
#
FROM debian:bookworm-slim

LABEL org.opencontainers.image.url='https://grafbase.com' \
    org.opencontainers.image.documentation='https://grafbase.com/docs' \
    org.opencontainers.image.source='https://github.com/grafbase/grafbase/tree/main/gateway' \
    org.opencontainers.image.vendor='Grafbase' \
    org.opencontainers.image.description='The Grafbase GraphQL Federation Gateway' \
    org.opencontainers.image.licenses='MPL-2.0'

WORKDIR /grafbase

# used curl to run a health check query against the server in a docker-compose file
RUN apt update && apt install -y curl && rm -rf /var/lib/apt/lists/*

RUN adduser --home "/data" grafbase && mkdir -p /data && chown grafbase /data
USER grafbase

COPY --from=builder /grafbase/target/release/grafbase-gateway /bin/grafbase-gateway
COPY --from=builder /grafbase/gateway/crates/federated-server/config/grafbase.toml /etc/grafbase.toml

VOLUME /data
WORKDIR /data

ENTRYPOINT ["/bin/grafbase-gateway"]
CMD ["--config", "/etc/grafbase.toml", "--listen-address", "0.0.0.0:5000"]

EXPOSE 5000
