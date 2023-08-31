# Grafbase SDK integration tests

Crate that provides integrations tests for `grafbase-sdk`. These tests leverage Miniflare
and cover the following:

- Cloudflare worker written in Rust (`src/**/*.rs`) that uses `grafbase-sdk`
- Cloudflare worker written in JavaScript (`js/**/*.js`) that uses the Wasm package of `grafbase-sdk`

To interact with the crate, make sure you have the following installed:

    - cargo-make
    - pnpm
    - npx
    - wasm-pack
    - worker-build
    - wrangler

## Build

    cargo make build

## Test

    cargo make test

## Run local-js

    cargo make serve-js

## Run local-rust

    cargo make serve-rust
