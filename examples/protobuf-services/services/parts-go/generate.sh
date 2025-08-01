#!/bin/bash
# This script generates the Go protobuf code for the parts service
# Run this from the parts-go directory

# Install protoc-gen-go and protoc-gen-go-grpc if not already installed
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# Generate the protobuf code
protoc --go_out=. --go-grpc_out=. \
  --go_opt=paths=source_relative \
  --go-grpc_opt=paths=source_relative \
  --go_opt=Mgrafbase/options.proto=github.com/grafbase/grafbase/examples/protobuf-services/services/parts-go/grafbase \
  --go-grpc_opt=Mgrafbase/options.proto=github.com/grafbase/grafbase/examples/protobuf-services/services/parts-go/grafbase \
  ../../proto/parts.proto \
  -I ../../proto/

echo "Generated parts.pb.go and parts_grpc.pb.go"