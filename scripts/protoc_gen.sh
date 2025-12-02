#!/bin/bash
# Generate Go protobuf files for VectorXLite gRPC client
#
# Prerequisites:
#   - protoc (Protocol Buffer compiler)
#   - protoc-gen-go: go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
#   - protoc-gen-go-grpc: go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
#
# Run this from the vector-db-rs root directory:
#   cd /path/to/vector-db-rs
#   ./vector_xlite_go_client/commands.sh

set -e

echo "Generating Go protobuf files..."

protoc \
  --proto_path=./grpc_proto \
  --go_out=paths=source_relative:./vector_xlite_go_client/pb \
  --go-grpc_out=paths=source_relative:./vector_xlite_go_client/pb \
  grpc_proto/vectorxlite.proto

echo "Done! Generated files:"
ls -la ./vector_xlite_go_client/pb/
