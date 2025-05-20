FROM rust:1.87-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config protobuf-compiler libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the necessary files
COPY proto/ /app/proto/
COPY server/ /app/server/

# Build the server
WORKDIR /app/server
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the built binary and data
COPY --from=builder /app/server/target/release/server /app/server
COPY --from=builder /app/server/db.json /app/db.json

# Expose port
EXPOSE 10000

# Run server
CMD ["/app/server"]
