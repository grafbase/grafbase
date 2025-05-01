# gRPC extension example

This example demonstrates how to use the Grafbase gRPC extension and protoc-gen-grafbase-subgraph to set up a virtual subgraph exposing a gRPC service. Note that arbitrarily many services can be exposed by an arbitrary number of subgraphs, but we will stick to a single service and a single subgraph in this example.

## Structure

- The `server/` directory contains the gRPC server implementation. We use the tonic framework, but it could be any gRPC server.
- The `proto/` directory contains the protobuf definitions for the gRPC service.
- The `schema.graphql` file contains the GraphQL definition of the virtual subgraph. It is generated with the [protoc-gen-grafbase-subgraph protoc plugin][protoc-plugin]

## Running the example

- [Install](https://grafbase.com/docs/reference/grafbase-cli#installation) the Grafbase CLI.
- Start the gRPC server: `docker compose up grpc-server`.
- Start the Grafbase development server: `grafbase dev`.
- Query the GraphQL API: `http://localhost:5000`.

Execute a mutation with the gRPC fields:

```graphql
mutation {
  routeguide_RouteGuide_GetFeature(input:  {
     latitude: 409146138
     longitude: -746188906
  }) {
    location {
      latitude
      longitude
    }
    name
  }
}
```

## Generating the subgraph schema

### Using protoc

- Install the [protoc plugin][protoc-plugin] (see [GitHub releases](https://github.com/grafbase/grafbase/releases/tag/protoc-gen-grafbase-subgraph-0.1.1)).
- Run `protoc --grafbase-subgraph_out=. ./proto/route_guide.proto -I ./proto/`

### Using buf

- Install the [protoc plugin][protoc-plugin] (see [GitHub releases](https://github.com/grafbase/grafbase/releases/tag/protoc-gen-grafbase-subgraph-0.1.1)).
- Run `buf generate`.

See the [buf.yaml](./buf.yaml) and [buf.gen.yaml](./buf.gen.yaml) files for configuration details.

[protoc-plugin]: https://github.com/grafbase/grafbase/tree/main/crates/protoc-gen-grafbase-subgraph
