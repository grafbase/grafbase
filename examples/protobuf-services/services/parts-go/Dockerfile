# Build stage
FROM golang:1.21-alpine AS builder

# Install build dependencies
RUN apk add --no-cache git

WORKDIR /app

# Copy go mod files
COPY services/parts-go/go.mod services/parts-go/go.sum ./

# Download dependencies
RUN go mod download

# Copy source code
COPY services/parts-go/ ./

# Copy proto files
COPY proto /proto

# Build the application
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o main .

# Production stage
FROM alpine:latest

# Install ca-certificates for HTTPS
RUN apk --no-cache add ca-certificates

WORKDIR /root/

# Copy the binary from builder
COPY --from=builder /app/main .
COPY --from=builder /proto /proto

# Expose gRPC port
EXPOSE 50052

# Run the service
CMD ["./main"]