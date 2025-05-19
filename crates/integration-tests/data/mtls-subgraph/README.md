# mTLS GraphQL Subgraph

A simple GraphQL server written in Rust with Axum, Rustls, and async-graphql that requires mutual TLS (mTLS) authentication.

## Features

- GraphQL API with a simple "Hello, world" query
- Custom TLS with self-signed certificates
- Mutual TLS (mTLS) authentication required from clients
- Built with Rust, Axum, Rustls, and async-graphql

## Prerequisites

- Rust and Cargo (latest stable version)
- OpenSSL (for certificate generation)
- Docker and Docker Compose (optional, for containerized deployment)

## Installation

1. Clone this repository
2. Navigate to the project directory
3. Run `cargo build` to compile the project

## Certificate Generation

The server requires TLS certificates for both server and client authentication. Use the following commands to generate them:

```bash
# Create certificates directory
mkdir -p certs

# Generate CA certificate
openssl req -x509 -newkey rsa:4096 -keyout certs/ca-key.pem -out certs/ca-cert.pem -days 365 -nodes -subj "/CN=CA"

# Generate server certificate
openssl req -newkey rsa:4096 -keyout certs/server-key.pem -out certs/server-req.pem -nodes -subj "/CN=localhost"
openssl x509 -req -in certs/server-req.pem -days 365 -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -CAcreateserial -out certs/server-cert.pem

# Generate client certificate
openssl req -newkey rsa:4096 -keyout certs/client-key.pem -out certs/client-req.pem -nodes -subj "/CN=client"
openssl x509 -req -in certs/client-req.pem -days 365 -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -CAcreateserial -out certs/client-cert.pem
```

## Running the Server

### Using Cargo (Native)

To start the GraphQL server with mTLS natively:

```bash
cargo run
```

The server will start on `https://0.0.0.0:3000/graphql`.

### Using Docker

To build and run the server in a Docker container:

```bash
# Build the Docker image
docker build -t mtls-graphql -f Dockerfile ..

# Run the container
docker run -p 3000:3000 -v $(pwd)/certs:/app/certs mtls-graphql
```

Alternatively, you can use Docker Compose:

```bash
docker-compose up --build
```

The server will be accessible at `https://localhost:3000/graphql`.

### Helper Script

A convenience script is provided to handle certificate generation and server startup:

```bash
# Run natively
./run.sh

# Run with Docker
./run.sh --docker

# Run the test client
./run.sh --client

# Show help
./run.sh --help
```

## Testing with the Client

A test client is provided to verify the mTLS connection:

```bash
cargo run --example test_client
```

This client will send a GraphQL query to the server using the client certificates for authentication.

## Code Structure

- `src/main.rs` - Server implementation with GraphQL schema and mTLS configuration
- `examples/test_client.rs` - Test client that connects to the server using mTLS
- `certs/` - Directory containing generated certificates
- `Dockerfile` - Instructions for building the Docker container
- `docker-compose.yml` - Docker Compose configuration for easy deployment
- `run.sh` - Helper script for certificate generation and server startup

## Troubleshooting

- **Certificate Issues**: Ensure that all certificates are properly generated and located in the `certs/` directory
- **Connection Refused**: Make sure the server is running and listening on the expected port
- **TLS Handshake Failures**: Verify that both client and server are using the correct certificates

## How It Works

The server uses Rustls to configure mTLS. It requires clients to present a valid certificate signed by the CA. The GraphQL schema defines a simple query that returns "Hello, world".

The client configures its TLS connection to include its client certificate when connecting to the server, completing the mutual authentication.