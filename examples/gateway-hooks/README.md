# Authorization with Grafbase Gateway hooks

This is an example how to implement custom authorization hooks with Grafbase Gateway federation.
Read more on the hooks from the [gateway hooks documentation](https://grafbase.com/docs/self-hosted-gateway/hooks),
and on the directive from the [@authorized directive documentation](https://grafbase.com/docs/federation/federation-directives#authorized).

## The components of this example

- `authorized-subgraph` has a simple subgraph server
- `demo-hooks` contains the code for WebAssembly hooks as a Rust project
- `federated-schema.graphql` is the federated GraphQL schema
- `grafbase.toml` has the configuration for the Grafbase Gateway

## Dependencies

To run this example, you need the Grafbase Gateway version 0.4.0 or later, read more how to install it from:

https://grafbase.com/docs/self-hosted-gateway

Additionally, the following tools are needed:

- A C compiler, such as clang together with pkg-config (install based on your system, `cc` command is required)
- If on linux, cargo-component depends on openssl (`libssl-dev` on Debian)
- Rust compiler ([install docs](https://www.rust-lang.org/learn/get-started))
- Cargo component ([install docs](https://github.com/bytecodealliance/cargo-component?tab=readme-ov-file#installation))
- A GraphQL client, such as [Altair](https://altair-gql.sirmuel.design/)

For the advanced users using nix with flakes support:

```
nix develop
```

## Running the example

First, start the subgraph in one terminal:

```bash
cd authorized-subgraph
cargo run --release
```

Then, compile the WebAssembly hook functions into a wasm component in another terminal:

```bash
cd demo-hooks
cargo component build --release
```

After a successful build, the component can be found from `target/wasm32-wasi/release/demo_hooks.wasm`.
This file must exist for us to continue.

Finally start the `grafbase-gateway`:

```bash
grafbase-gateway --schema federated-schema.graphql --config grafbase.toml
```

A successful start of the gateway will give the following output:

```
2024-07-02T17:33:17.242780Z  INFO Grafbase Gateway 0.3.2
2024-07-02T17:33:17.259341Z  INFO loaded the provided WASM component successfully
2024-07-02T17:33:17.260585Z  INFO Waiting for engine to be ready...
2024-07-02T17:33:17.260601Z  INFO error waiting for otel reload
2024-07-02T17:33:17.260633Z  INFO GraphQL endpoint exposed at http://127.0.0.1:5000/graphql
```

Now open up the GraphQL client and start firing some queries. Read the hooks code in `demo-hooks/src/lib.rs` and adapt the header
value `x-current-user-id` accordingly to see the authorization hooks in action.

## Example query

```graphql
query {
  getUser(id: 2) {
    id
    name
    address {
      street
    }
    secret {
      socialSecurityNumber
    }
  }
  getSecret(id: 1) {
    socialSecurityNumber
  }
}
```

By changing the `x-current-user-id` header to different values, e.g. between `1` and `2` will give different requests to this query.
