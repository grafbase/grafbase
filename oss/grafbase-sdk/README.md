# Grafbase SDK

The Grafbase Rust SDK that exposes interfaces to work with the Grafbase platform.

To interact with the crate make sure you have the following installed: - [cargo-make](https://github.com/sagiegurari/cargo-make#installation) - [esbuild](https://esbuild.github.io/getting-started/#install-esbuild)

## Build

Builds the crate

    cargo make build

## Bundle

Generates the distribution bundle

    cargo make bundle

## Release

Release the distribution bundle to npm

    TODO

## Rust Usage

    ```Cargo.toml

    [dependencies]
    grafbase-sdk = "0.0.1"

    ```

    ```worker.rs

    use grafbase_sdk::api::kv::{KvStore};

    async fn kv_store_example(env: &worker::Env) -> worker::Result<()> {
        let kv_store = KvStore::new(env)?;
        let js_value = kv_store.get_with_metadata(key, None).await?;

        ...
    }

    ```

## JS Usage

    ```package.json
        {
            "dependencies": {
                "grafbase-wasm-sdk": "0.0.1"
            }
        }
    ```

    ```worker.js
        import { KvStore } from 'grafbase-wasm-sdk'

        export default {
            async fetch(request, env, ctx) {
                const key = "key";

                const kvStore = new KvStore(env)
                const value = await kvStore.get(key)

                ...
            }
        }
    ```
