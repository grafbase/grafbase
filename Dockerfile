#
# === Build image ===
#
FROM rust:1.78-alpine3.20 AS chef
COPY rust-toolchain.toml rust-toolchain.toml
RUN apk add --no-cache musl-dev && cargo install cargo-chef
WORKDIR /grafbase

FROM chef AS planner
# At this stage we don't really bother selecting anything specific, it's fast enough.
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /grafbase/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY ./gateway ./gateway
COPY ./cli ./cli
COPY ./graphql-introspection ./graphql-introspection
COPY ./graph-ref ./graph-ref
COPY ./graphql-lint ./graphql-lint
COPY ./gqlint ./gqlint
COPY ./engine ./engine

RUN cargo build -p grafbase-gateway --release

#
# === Final image ===
#
FROM alpine:3.20

WORKDIR /grafbase

# used curl to run a health check query against the server in a docker-compose file
RUN apk add --no-cache curl

RUN adduser -g wheel -D grafbase -h "/data" && mkdir -p /data && chown grafbase: /data
USER grafbase

COPY --from=builder /grafbase/target/release/grafbase-gateway /bin/grafbase-gateway
COPY --from=builder /grafbase/gateway/crates/federated-server/config/grafbase.toml /etc/grafbase.toml

# these args should be set so the binary can start, they have to be changed for successfully running the gateway
ARG GRAFBASE_GRAPH_REF
ARG GRAFBASE_ACCESS_TOKEN

VOLUME /data
WORKDIR /data

ENTRYPOINT ["/bin/grafbase-gateway"]
CMD ["--config", "/etc/grafbase.toml", "--listen-address", "0.0.0.0:5000"]

EXPOSE 5000
