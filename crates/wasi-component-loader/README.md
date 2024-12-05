# Wasm Component Loader for the Grafbase Gateway

Adds support for loading WebAssembly components in the Grafbase Gateway. The Wasm file has to be in a form of WASI Preview 2 component. Compile all examples with the `wasm32-wasip2` target. See the examples for simple guest components, which are all tested in CI for this crate and work together with the host library.

You can create a hooks component using the [grafbase-hooks](/grafbase/grafbase/tree/main/crates/grafbase-hooks) crate.

The component defines the functions the guest is interested to plug into. If the host cannot find that exact function from the guest, the host hook will be a no-op.
