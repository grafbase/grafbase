# Access Logs with Response Hooks

This example demonstrates how to render and write access logs from Grafbase Gateway.

## The Components of this Example

- `subgraph` contains a simple subgraph server.
- `hooks` holds the response hooks that generate and store access logs.
- `federated-schema.graphql` is the federated GraphQL schema.
- `grafbase.toml` has the configuration for the Grafbase Gateway.

## Dependencies

To run this example, you'll need:

- Grafbase Gateway version 0.12.0 or later ([install instructions](https://grafbase.com/docs/self-hosted-gateway/))
- A C compiler (e.g., clang) and pkg-config
- Rust compiler ([installation guide](https://www.rust-lang.org/learn/get-started))
- Cargo component ([installation guide](https://github.com/bytecodealliance/cargo-component?tab=readme-ov-file#installation))

For advanced users using nix with flakes support:

```
nix develop
```

## Running the Example

Start the OpenTelemetry services:

```bash
docker compose up -d
```

Run the subgraph in one terminal:

```bash
cd subgraph
cargo run --release
```

Compile the WebAssembly hook functions into a Wasm component in another terminal:

```bash
cd hooks
cargo component build --release
```

After a successful build, the component will be found at `target/wasm32-wasip1/release/hooks.wasm`. This file must exist to continue.

Finally, start the `grafbase-gateway`:

```bash
grafbase-gateway --schema federated-schema.graphql --config grafbase.toml
```

Send a GraphQL request to the federated graph:

```bash
curl -X POST 'http://127.0.0.1:5000/graphql' --data '{"query": "query { user(id: 1) { id name address { street } } }"}' -H "Content-Type: application/json"
```

The file `log/access.log` will contain exactly one row for every request, in a format similar to:

```json
{
  "method": "POST",
  "url": "/graphql",
  "status_code": 200,
  "trace_id": "45f859a51542514cbf78ba240b187f37",
  "operations": [
    {
      "name": "user",
      "document": "query {\n  user(id: 0) {\n    address {\n      street\n    }\n    id\n    name\n  }\n}\n",
      "prepare_duration_ms": 0,
      "cached_plan": false,
      "duration_ms": 2,
      "status": "Success",
      "subgraphs": [
        {
          "subgraph_name": "users",
          "method": "POST",
          "url": "http://localhost:4000/graphql",
          "responses": [
            {
              "Response": {
                "connection_time_ms": 1,
                "response_time_ms": 1,
                "status_code": 200
              }
            }
          ],
          "total_duration_ms": 1,
          "has_errors": false,
          "cached": false
        }
      ]
    }
  ]
}
```
