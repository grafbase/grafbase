# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev protobuf-dev

WORKDIR /app

# Copy Cargo files
COPY services/locations-rust/Cargo.toml services/locations-rust/Cargo.lock* ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build && \
    rm -rf src

# Copy source code
COPY services/locations-rust/src ./src
COPY services/locations-rust/build.rs ./

# Copy proto files
COPY proto /proto

# Build the application
RUN cargo build

# Production stage
FROM alpine:latest

# Install runtime dependencies
RUN apk --no-cache add ca-certificates

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/debug/locations-service .
COPY --from=builder /proto /proto

# Expose gRPC port
EXPOSE 50053

# Run the service
CMD ["./locations-service"]
