# Running

## Assets

Assets in the API repo need to be built:

```sh
RELEASE_FLAG="--dev" GATEWAY_FEATURES=local ./scripts/dev/build-cli-assets.sh
```

## Running the CLI

```sh
rm -rf ~/.grafbase # version is the same so the cache can contain the wrong wasm variant

cargo run -p grafbase -- -t 2 dev
```

## Running the tests

```sh
cargo nextest run -P ci
```
