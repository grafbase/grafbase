# Build stage
FROM golang:1.25-alpine AS builder

# Install build dependencies
RUN apk add --no-cache git protobuf-dev

# Install protoc-gen-go and protoc-gen-go-grpc
RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@latest && \
    go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

WORKDIR /app

# Copy proto files for generation
COPY proto/ /proto/

# Copy only necessary source files (not generated ones)
COPY services/parts-go/go.mod services/parts-go/go.sum ./
COPY services/parts-go/main.go ./
COPY services/parts-go/grafbase/options.pb.go ./grafbase/

# Add replace directive for local grafbase package
RUN echo 'replace github.com/grafbase/grafbase/examples/grpc-composite-schemas/services/parts-go/grafbase => ./grafbase' >> go.mod

# Generate parts protobuf code
RUN protoc --go_out=. --go-grpc_out=. \
    --go_opt=paths=source_relative \
    --go-grpc_opt=paths=source_relative \
    --go_opt=Mgrafbase/options.proto=github.com/grafbase/grafbase/examples/grpc-composite-schemas/services/parts-go/grafbase \
    --go-grpc_opt=Mgrafbase/options.proto=github.com/grafbase/grafbase/examples/grpc-composite-schemas/services/parts-go/grafbase \
    -I /proto/ \
    /proto/parts.proto && \
    sed -i 's/^package parts$/package main/' parts.pb.go parts_grpc.pb.go

# Download dependencies
RUN go mod download

# Build the application
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o main .

# Production stage
FROM alpine:latest

# Install ca-certificates for HTTPS
RUN apk --no-cache add ca-certificates

WORKDIR /root/

# Copy the binary from builder
COPY --from=builder /app/main .

# Expose gRPC port
EXPOSE 50052

# Run the service
CMD ["./main"]
