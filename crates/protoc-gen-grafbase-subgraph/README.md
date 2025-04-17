# protoc-gen-grafbase-subgraph

This binary crate is a protoc plugin that generates a GraphQL subgraph to be used in concert with the Grafbase gRPC extension.

## Installation

Download the relevant binary from your platform from the [GitHub releases](https://github.com/grafbase/grafbase/releases?q=protoc-gen-grafbase-subgraph&expanded=true).

## Usage with buf

Make sure the binary is in your PATH, then configure it in your `buf.gen.yaml` file:

```yaml
version: v2
managed:
  enabled: true
plugins:
  - local: protoc-gen-grafbase-subgraph
    out: .
inputs:
  - directory: proto
```

## Usage with protoc

Make sure the binary is in your PATH, then run protoc with the `--grafbase-subgraph_out` flag. For example:

```
protoc --grafbase-subgraph_out=. proto/*.proto
```

## Limitations

- Methods with client streaming are supported, but only one message can be sent from the client side.
- All non-streaming methods are mapped to Mutation fields, and all server-streaming methods are mapped to Subscription fields. We want to implement optionally mapping methods to Query fields through a method option, please get in touch if you are interested.

## Releasing

To release a new version of the binary:

1. Update the version number in `Cargo.toml`
2. Create a tag with the format `protoc-gen-grafbase-subgraph-X.Y.Z` (e.g., `protoc-gen-grafbase-subgraph-0.2.0`)
3. Push the tag to GitHub:
   ```
   git tag protoc-gen-grafbase-subgraph-X.Y.Z
   git push origin protoc-gen-grafbase-subgraph-X.Y.Z
   ```
4. The GitHub Actions workflow will automatically build the binary for multiple platforms and create a release with the artifacts

## Prior art

- https://github.com/ysugimoto/grpc-graphql-gateway generates a graphql-go based GraphQL server that proxies to a gRPC server.
- https://github.com/danielvladco/go-proto-gql another project that does the same. Unmaintained.
