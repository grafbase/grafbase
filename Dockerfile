# Build
FROM rust:1.76-alpine3.18 AS build

WORKDIR /grafbase

RUN mkdir -p packages/grafbase-sdk

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY ./cli ./cli
COPY ./engine ./engine
COPY ./packages/grafbase-sdk/package.json ./packages/grafbase-sdk
COPY ./packages/cli-app ./packages/cli-app

RUN apk add --no-cache git musl-dev npm

WORKDIR /grafbase/packages/cli-app

RUN npx --yes pnpm i
RUN npx --yes pnpm run cli-app:build

WORKDIR /grafbase

RUN cargo build -p grafbase --release

# Run
FROM alpine:3.19

WORKDIR /grafbase

# used curl to run a health check query against the server in a docker-compose file
RUN apk add --no-cache nodejs npm curl

RUN adduser -g wheel -D grafbase -h "/data" && mkdir -p /data && chown grafbase: /data
USER grafbase

COPY --from=build /grafbase/target/release/grafbase /bin/grafbase

ENTRYPOINT ["/bin/grafbase"]

CMD ["start"]

EXPOSE 4000

VOLUME ["/data"]
WORKDIR "/data"
