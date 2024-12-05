# Grafbase Hooks Template

Use this template as the basis of your Grafbase hooks. For development, you need:

- Rust ([install](https://rustup.rs/))
- Cargo Component, if using older Rust than 1.83 ([install](https://github.com/bytecodealliance/cargo-component))
- The wasm32-wasip2 target for Rust version 1.83 or later (`rustup target add wasm32-wasip2`)

To compile the hooks with Rust 1.83 or later, run:

```bash
cargo build --release --target wasm32-wasip2
```

To compile the hooks with Rust 1.82 or older, run:

```bash
cargo component build --release
```

The compiled hooks are in the `target/wasm32-wasip2/release/hooks_template.wasm` file with Rust 1.83 or later, and in `target/wasm32-wasip1/release/hooks_template.wasm` file for earlier rust versions. To change the name of the file, rename the crate in the `Cargo.toml` file. This file should be deployed together with the Grafbase Gateway.
