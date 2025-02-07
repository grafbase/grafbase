# Grafbase Gateway SDK for Extensions

This crate provides building blocks for creating [Grafbase Gateway](https://grafbase.com/docs/reference/gateway/installation) extensions.

## Usage

**Extensions are still under development. Expect issues if you try them out before we complete development.**

Initialize a new project with the [Grafbase CLI](https://grafbase.com/docs/reference/grafbase-cli):

```bash,no_run
grafbase extension init --type auth/resolver my-extension
```

This creates a new project with the necessary files and dependencies to get you started. Edit the `src/lib.rs` file to add your extension logic. The Grafbase Gateway initializes the struct `TestProject` once during the initial extension call. The Grafbase Gateway keeps extensions in a connection pool and reuses the struct for multiple requests. Because an extension is single-threaded, we keep multiple instances in the gateway memory to handle multiple requests concurrently.

The initialization gets a list of schema directives containing the schema directives from the federated schema, defined in the schema file. Often you need configuration to initialize the extension, which the schema directive provides.

The `ResolverExtension` derive macro generates the necessary code to initialize the extension, and guides you to implement two traits: [`Extension`] and [`Resolver`]. The `Extension` trait initializes the extension, and the `Resolver` trait implements the extension logic to resolve a field:

## Building

You can build your extension with the Grafbase CLI. For this to work, you must have a working [rustup](https://rustup.rs/) installation:

```bash,ignore
grafbase extension build
```

This compiles your extension and creates two files:

```ignore
build/
├── manifest.json
└── test_project.wasm
```

## Checking

The Grafbase CLI provides a way to test your extension's implementation:

```bash,ignore
grafbase extension check
```

This builds the extension and initializes it from the Grafbase Gateway.

## Publishing

To publish the extension to the Grafbase extensions repository, use the Grafbase CLI:

```bash,ignore
grafbase extension publish
```
