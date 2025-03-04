# gRPC Extension

This extension enables you to make gRPC calls using HTTP/2 transport in your Grafbase schema. It supports JSON-based gRPC requests and responses.

## Usage

```graphql
type Query {
  # Example of a gRPC call
  getUser(id: ID!): User @grpc(
    endpoint: "https://grpc.example.com"
    service: "users.UserService"
    method: "GetUser"
    request: {
      "userId": "{{id}}"
    }
    headers: {
      "authorization": "Bearer token"
    }
    responsePath: "user"
    timeout: 5000
  )
}
```

## Configuration

The `@grpc` directive supports the following arguments:

- `endpoint` (required): The base URL of the gRPC service
- `service` (required): The gRPC service name
- `method` (required): The gRPC method name
- `request` (optional): JSON template for the request body. Supports variable interpolation using `{{variableName}}` syntax
- `headers` (optional): Additional headers to include in the request
- `responsePath` (optional): Dot-notation path to extract specific fields from the response
- `timeout` (optional): Request timeout in milliseconds

## Variable Interpolation

The extension supports variable interpolation in the request template. Use the `{{variableName}}` syntax to reference field arguments:

```graphql
type Mutation {
  createUser(name: String!, email: String!): User @grpc(
    endpoint: "https://grpc.example.com"
    service: "users.UserService"
    method: "CreateUser"
    request: {
      "name": "{{name}}",
      "email": "{{email}}"
    }
  )
}
```

## Response Path Extraction

Use the `responsePath` argument to extract specific fields from the response:

```graphql
type Query {
  getUserProfile(id: ID!): Profile @grpc(
    endpoint: "https://grpc.example.com"
    service: "users.UserService"
    method: "GetUserProfile"
    request: {
      "userId": "{{id}}"
    }
    responsePath: "data.profile"  # Extracts the profile object from data.profile
  )
}
```

## Error Handling

The extension will return an error if:
- The gRPC request fails (non-200 status code)
- The response cannot be parsed as JSON
- The specified response path cannot be found in the response

## Development

To build the extension:

```bash
cargo build --release
```

To run tests:

```bash
cargo test
``` 