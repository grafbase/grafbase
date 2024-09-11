# Blocking Requests with the Gateway Request Hook

This example shows how to block requests on the Grafbase Gateway using the `on-gateway-request` hook.

## Components of This Example

- **subgraph**: Contains a simple subgraph server.
- **request-hook**: Holds the code for the request hook WebAssembly component.
- **federated-schema.graphql**: Defines the federated GraphQL schema.
- **grafbase.toml**: Includes the configuration for the Grafbase Gateway.

## Dependencies

To run this example, you need the following:

- Grafbase Gateway version 0.12.0 or later ([installation instructions](https://grafbase.com/docs/self-hosted-gateway/))
- A C compiler (e.g., clang) and pkg-config
- Rust compiler ([installation guide](https://www.rust-lang.org/learn/get-started))
- Cargo component ([installation guide](https://github.com/bytecodealliance/cargo-component?tab=readme-ov-file#installation))

For advanced users with nix and flakes support, run the following:

```bash
nix develop
```

## Running the Example

1. **Run the Subgraph**: Open one terminal and navigate to the subgraph directory. Start the subgraph server:

   ```bash
   cd subgraph
   cargo run --release
   ```

2. **Compile the WebAssembly Hook**: Open another terminal and navigate to the request-hook directory. Compile the WebAssembly hook functions into a Wasm component:

   ```bash
   cd request-hook
   cargo component build --release
   ```

   After a successful build, you will find the component at `target/wasm32-wasip1/request-hook/hooks.wasm`. This file must exist to proceed.

3. **Start the Grafbase Gateway**: Finally, start the Grafbase Gateway with the following command:

   ```bash
   grafbase-gateway --schema federated-schema.graphql --config grafbase.toml
   ```

4. **Send a GraphQL Request**: Test the system by sending a GraphQL request to the federated graph:

   ```bash
   curl -X POST 'http://127.0.0.1:5000/graphql' \
     --data '{"query": "query { user(id: 1) { id name address { street } } }"}' \
     -H "Content-Type: application/json" \
     -H "x-custom: secret"
   ```

   By changing the value of the `x-custom` header, you can control whether the response returns data or an error.
