#
# === Build image ===
#
FROM rust:1.82-alpine3.20 AS chef

# Patching musl to use mimalloc, see docker/README.md.
# Source: https://github.com/tweag/rust-alpine-mimalloc
COPY docker/build.sh docker/mimalloc.diff /tmp/
RUN /tmp/build.sh
ENV LD_PRELOAD=/usr/lib/libmimalloc.so
ENV RUSTFLAGS="-C target-feature=+crt-static"

COPY rust-toolchain.toml rust-toolchain.toml
RUN apk add --no-cache musl-dev && cargo install --target "$(rustc -vV | sed -n "s|host: ||p")" cargo-chef
WORKDIR /grafbase

FROM chef AS planner
# At this stage we don't really bother selecting anything specific, it's fast enough.
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV CARGO_INCREMENTAL=0
COPY --from=planner /grafbase/recipe.json recipe.json
RUN cargo chef cook --target "$(rustc -vV | sed -n "s|host: ||p")" --release --recipe-path recipe.json

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

RUN cargo build --release --target "$(rustc -vV | sed -n "s|host: ||p")" --bin grafbase-gateway &&\
    mv "/grafbase/target/$(rustc -vV | sed -n "s|host: ||p")/release/grafbase-gateway" /grafbase/target/release/grafbase-gateway

#
# === Final image ===
#
FROM alpine:3.20

LABEL org.opencontainers.image.url='https://grafbase.com' \
    org.opencontainers.image.documentation='https://grafbase.com/docs' \
    org.opencontainers.image.source='https://github.com/grafbase/grafbase/tree/main/gateway' \
    org.opencontainers.image.vendor='Grafbase' \
    org.opencontainers.image.description='The Grafbase GraphQL Federation Gateway' \
    org.opencontainers.image.licenses='MPL-2.0'

WORKDIR /grafbase

# used curl to run a health check query against the server in a docker-compose file
RUN apk add --no-cache curl

RUN adduser -g wheel -D grafbase -h "/data" && mkdir -p /data && chown grafbase: /data
USER grafbase

COPY --from=builder /grafbase/target/release/grafbase-gateway /bin/grafbase-gateway
COPY --from=builder /grafbase/gateway/crates/federated-server/config/grafbase.toml /etc/grafbase.toml

VOLUME /data
WORKDIR /data

ENTRYPOINT ["/bin/grafbase-gateway"]
CMD ["--config", "/etc/grafbase.toml", "--listen-address", "0.0.0.0:5000"]

EXPOSE 5000
