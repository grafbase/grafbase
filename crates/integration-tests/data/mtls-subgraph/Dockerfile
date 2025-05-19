FROM rust:1-bullseye AS builder

WORKDIR /usr/src/app

# Copy over manifests and build dependencies separately to utilize Docker caching
COPY Cargo.toml Cargo.lock ./
# Create a dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() { println!(\"dummy\"); }" > src/main.rs
RUN cargo build --release
# Remove the dummy build artifacts
RUN rm -rf src target

# Copy the actual source code and build the application
COPY src ./src
RUN cargo build --release

# Create a smaller runtime image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/mtls-subgraph /app/mtls-subgraph

# Create certificates directory
RUN mkdir -p /app/certs

# Copy TLS certificates
COPY certs/* /app/certs/

# Expose the port the server listens on
EXPOSE 3000

# Run the server
CMD ["/app/mtls-subgraph"]
