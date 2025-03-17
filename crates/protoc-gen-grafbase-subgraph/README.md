# protoc-gen-grafbase-subgraph

This binary crate is a protoc plugin that generates a GraphQL subgraph to be used in concert with the Grafbase gRPC extension.

## Limitations

- Methods with client streaming are supported, but only one message can be sent from the client side.

## Prior art

- https://github.com/ysugimoto/grpc-graphql-gateway generates a graphql-go based GraphQL server that proxies to a gRPC server.
- https://github.com/danielvladco/go-proto-gql another project that does the same. Unmaintained.
