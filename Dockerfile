# Build
FROM rust:1.78-alpine3.18 AS build

WORKDIR /grafbase

RUN mkdir -p packages/grafbase-sdk

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY ./gateway ./gateway
COPY ./cli ./cli
COPY ./graphql-introspection ./graphql-introspection
COPY ./graph-ref ./graph-ref
COPY ./graphql-lint ./graphql-lint
COPY ./gqlint ./gqlint
COPY ./engine ./engine

RUN apk add --no-cache git musl-dev

RUN cargo build -p grafbase-gateway --release

# Run
FROM alpine:3.19

WORKDIR /grafbase

# used curl to run a health check query against the server in a docker-compose file
RUN apk add --no-cache curl

RUN adduser -g wheel -D grafbase -h "/data" && mkdir -p /data && chown grafbase: /data
USER grafbase

COPY --from=build /grafbase/target/release/grafbase-gateway /bin/grafbase-gateway
COPY --from=build /grafbase/gateway/crates/federated-server/config/grafbase.toml /etc/grafbase.toml

ENTRYPOINT ["/bin/grafbase-gateway"]

# these args should be set so the binary can start, they have to be changed for successfully running the gateway
ARG GRAFBASE_GRAPH_REF
ARG GRAFBASE_ACCESS_TOKEN

CMD ["--config", "/etc/grafbase.toml", "--listen-address", "0.0.0.0:5000"]

EXPOSE 4000

VOLUME ["/data"]
WORKDIR "/data"
