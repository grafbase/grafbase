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

## Custom options

To make use of custom options, you will need to import the option definitions. They are defined in the `grafbase.options` package, available in the "options.proto" file in this project. Also note that since this module imports ["google/protobuf/descriptor.proto"](https://github.com/protocolbuffers/protobuf/blob/8228ee42b512cc330971e61bc9b86935a59f3477/src/google/protobuf/descriptor.proto), that one has to be present in your project as well.

With `buf`, you can make use of the [inputs.git_repo](https://buf.build/docs/configuration/v2/buf-gen-yaml/#git_repo) option.

### Mapping RPC methods to Query fields

By default, RPC methods are mapped to fields on Mutation. But you can also map them to fields on Query:

```protobuf
import "grafbase/options.proto";

service SearchService {
  rpc Search(SearchRequest) returns (SearchResponse) {
    option (grafbase.graphql.is_query_field) = true;
  }
}
```

### Default all service methods to Query fields

```protobuf
import "grafbase/options.proto";

service SearchService {
  option (grafbase.graphql.default_to_query_fields) = true;

  rpc Search(SearchRequest) returns (SearchResponse) {
    option (grafbase.graphql.method_field_directives) = "@lookup";
  }
}
```

Analogous to the "is_query_field" option above, there is also a "is_mutation_field" option to map RPC methods to fields on Mutation when your service defaults to Query:

```protobuf
import "grafbase/options.proto";

service SearchService {
  option (grafbase.graphql.default_to_query_fields) = true;

  rpc Search(SearchRequest) returns (SearchResponse) {
    option (grafbase.graphql.is_mutation_field) = true;
  }
}
```

### Adding GraphQL directives on types, fields and enum values

```protobuf
import "grafbase/options.proto";

message MyMessage {
  option (grafbase.graphql.output_object_directives) = "@key(fields: \"id\")";

  string id = 1 [(grafbase.graphql.output_field_directives) = "@deprecated"];
}

enum Color {
  option (grafbase.graphql.enum_directives) = "@deprecated";

  RED = 0 [(grafbase.graphql.enum_value_directives) = "@deprecated @tag(name: \"private\")"];
  GREEN = 1 [(grafbase.graphql.enum_value_directives) = "@deprecated"];
  BLUE = 2 [(grafbase.graphql.enum_value_directives) = "@deprecated"];
}
```

## Limitations

- Methods with client streaming are supported, but only one message can be sent from the client side.

## Contributing

### Releasing

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
