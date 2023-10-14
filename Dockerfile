# Build
FROM rust:1.73 AS build

WORKDIR /grafbase

COPY ./cli ./cli
COPY ./engine ./engine

COPY ./cli/Cargo.lock ./cli/Cargo.lock
COPY ./cli/Cargo.toml ./cli/Cargo.toml
COPY ./engine/Cargo.lock ./engine/Cargo.lock
COPY ./engine/Cargo.toml ./engine/Cargo.toml

WORKDIR /grafbase/cli

RUN cargo build --release

# Run
FROM debian:bullseye AS run

COPY --from=build /grafbase/cli/target/release/grafbase .

CMD ["./grafbase", "start"]
