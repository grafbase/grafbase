# Grafbase Hooks Template

Use this template as the basis of your Grafbase hooks. For development, you need:

- Rust ([install](https://rustup.rs/))
- Cargo Component ([install](https://github.com/bytecodealliance/cargo-component))
- A C compiler (e.g., clang) and pkg-config

To compile the hooks, run:

```bash
cargo component build --release
```

The compiled hooks are in the `target/wasm32-wasip1/release/hooks_template.wasm` file. To change the name of the file, rename the crate in the `Cargo.toml` file. This file should be deployed together with the Grafbase Gateway.
