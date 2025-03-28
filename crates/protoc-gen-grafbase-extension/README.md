## Supported options

On methods:

- `[graphql(query)]` makes the method a GraphQL query. By default, methods are mapped to mutations.


## Limitations

- Methods with client streaming are not supported. Server streaming methods are mapped to GraphQL subscriptions.

## Prior art

- https://github.com/ysugimoto/grpc-graphql-gateway generates a graphql-go based GraphQL server that proxies to a GRPC server.
- https://github.com/danielvladco/go-proto-gql another project that does the same. Unmaintained.
