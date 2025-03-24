# Grafbase Gateway Hooks SDK

This crate provides the necessary types and macros to implement hooks for the [Grafbase Gateway](https://grafbase.com/docs/self-hosted-gateway).
A hook is a function that is called by the gateway at specific points in the request processing.

Build your own hooks by implementing the [`Hooks`] trait, add the [`grafbase_hooks`] attribute on top of the hooks
implementation and register the hooks type to the gateway using the [`register_hooks`] macro.

The hooks component is a WASM module that is loaded by the gateway at startup. If you are using Rust version 1.83 or later,
you can install the `wasm32-wasip2` target with the following command:

```bash
rustup target add wasm32-wasip2
```

For older versions of Rust, you can use the `wasm32-wasip1` target, but you must compile your hooks with the
[`cargo-component`](https://github.com/bytecodealliance/cargo-component) toolchain, which adds a compatibility
layer to the hooks module so it can be loaded by the gateway:

```bash
cargo install cargo-component
```

## Usage

Create a new rust library project with cargo:

```bash
cargo new --lib my-hooks
cd my-hooks
```

Add the `grafbase-hooks` as a dependency:

```bash
cargo add grafbase-hooks --features derive
```

Edit the `src/lib.rs` file and add the following code:

```rust
use grafbase_hooks::{grafbase_hooks, register_hooks, Context, ErrorResponse, Headers, Hooks};

struct MyHooks;

#[grafbase_hooks]
impl Hooks for MyHooks {
    fn new() -> Self
    where
        Self: Sized,
    {
        MyHooks
    }

    fn on_gateway_request(
        &mut self,
        context: Context,
        url: String,
        headers: Headers
    ) -> Result<(), ErrorResponse> {
        if let Some(ref auth_header) = headers.get("authorization") {
           context.set("auth", auth_header);
        }

        context.set("url", &url);

        Ok(())
    }
}

register_hooks!(MyHooks);
```

The example above implements the [`Hooks#on_gateway_request`] hook, which will be available in the gateway and will be called
for every request.

The [`grafbase_hooks`] attribute is used to generate the necessary code for the hooks implementation and
the [`register_hooks`] macro registers the hooks type to the gateway. The macro must be called in the library crate root.

To compile the hooks with Rust 1.83 or later:

```
cargo build --target wasm32-wasip2 --release
```

With older versions of Rust, the hooks are compiled with the `cargo-component` subcommand:

```bash
cargo component build --release
```

With Rust 1.83 or later, the compiled hooks wasm module is located in the `target/wasm32-wasip2/release` directory. With older
versions of Rust, the compiled hooks wasm module is located in the `target/wasm32-wasip1/release` directory.

You can configure the gateway to load the hooks in the `grafbase.toml` configuration file:

```toml
[hooks]
location = "path/to/my_hooks.wasm"
```
