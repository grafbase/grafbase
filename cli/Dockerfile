#
# === Pathfinder & UDF wrapper ===
#
FROM oven/bun:1.1.29-alpine AS assets
WORKDIR /grafbase
COPY ./cli/wrappers ./cli/wrappers
COPY ./packages/cli-app ./packages/cli-app
RUN apk add --no-cache git npm && \
    cd packages/cli-app && \
    npx --yes pnpm i && \
    npx --yes pnpm run cli-app:build && \
    cd ../../cli/wrappers && \
    bun i && \
    bun run build

#
# === Grafbase CLI ===
#
# Using the same alpine image as oven/bun
# https://github.com/oven-sh/bun/blob/bun-v1.1.12/dockerhub/alpine/Dockerfile
FROM rust:1.81-alpine3.20 AS chef

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
COPY ./packages/grafbase-sdk/package.json ./packages/grafbase-sdk/package.json
COPY --from=assets /grafbase/packages/cli-app/dist ./packages/cli-app/dist
COPY --from=assets /grafbase/cli/wrappers/dist.js ./cli/wrappers/dist.js
RUN cargo build --release --target "$(rustc -vV | sed -n "s|host: ||p")" --bin grafbase &&\
    mv "/grafbase/target/$(rustc -vV | sed -n "s|host: ||p")/release/grafbase" /grafbase/target/release/grafbase

#
# === Final image ===
#
FROM oven/bun:1.1.29-alpine

LABEL org.opencontainers.image.url='https://grafbase.com/cli' \
    org.opencontainers.image.documentation='https://grafbase.com/docs/cli' \
    org.opencontainers.image.source='https://github.com/grafbase/grafbase/tree/main/cli' \
    org.opencontainers.image.vendor='Grafbase' \
    org.opencontainers.image.description='The Grafbase Command Line Interface' \
    org.opencontainers.image.licenses='Apache-2.0'

# gosu is a better sudo/su to step down from root into the service user in the docker-entrypoint.sh script
# See https://github.com/tianon/gosu/blob/master/INSTALL.md
ENV GOSU_VERSION=1.17
RUN set -eux; \
    \
    apk add --no-cache --virtual .gosu-deps \
    ca-certificates \
    dpkg \
    gnupg \
    ; \
    \
    dpkgArch="$(dpkg --print-architecture | awk -F- '{ print $NF }')"; \
    wget -O /usr/local/bin/gosu "https://github.com/tianon/gosu/releases/download/$GOSU_VERSION/gosu-$dpkgArch"; \
    wget -O /usr/local/bin/gosu.asc "https://github.com/tianon/gosu/releases/download/$GOSU_VERSION/gosu-$dpkgArch.asc"; \
    \
    # verify the signature
    export GNUPGHOME="$(mktemp -d)"; \
    gpg --batch --keyserver hkps://keys.openpgp.org --recv-keys B42F6819007F00F88E364FD4036A9C25BF357DD4; \
    gpg --batch --verify /usr/local/bin/gosu.asc /usr/local/bin/gosu; \
    gpgconf --kill all; \
    rm -rf "$GNUPGHOME" /usr/local/bin/gosu.asc; \
    \
    # clean up fetch dependencies
    apk del --no-network .gosu-deps; \
    \
    chmod +x /usr/local/bin/gosu; \
    # verify that the binary works
    gosu --version; \
    gosu nobody true

RUN mkdir /etc/grafbase && \
    touch /etc/grafbase/inside-docker && \
    adduser -g wheel -D grafbase -h "/home/grafbase" && \
    mkdir /data && \
    chown grafbase:grafbase /data

COPY --from=builder /grafbase/target/release/grafbase /bin/
COPY ./cli/docker-entrypoint.sh /bin/

VOLUME /data
WORKDIR /data

EXPOSE 4000

ENTRYPOINT ["/bin/docker-entrypoint.sh"]
CMD ["start", "--listen-address", "0.0.0.0:4000"]
