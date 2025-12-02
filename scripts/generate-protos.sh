#!/bin/bash
# Generate Go protobuf files for VectorXLite
#
# Prerequisites:
#   - protoc (Protocol Buffer compiler)
#   - protoc-gen-go: go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
#   - protoc-gen-go-grpc: go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
#
# Run this from the vector-db-rs root directory:
#   cd /path/to/vector-db-rs
#   ./scripts/generate-protos.sh

set -e

echo "Generating Go protobuf files..."

# Generate VectorXLite gRPC protos (standalone client)
echo "Generating VectorXLite protos..."
protoc \
  --proto_path=./proto \
  --go_out=paths=source_relative:./standalone/clients/go/pb \
  --go-grpc_out=paths=source_relative:./standalone/clients/go/pb \
  proto/vectorxlite/v1/vectorxlite.proto

echo "Done! Generated VectorXLite files:"
ls -la ./standalone/clients/go/pb/

# Generate Cluster protos (distributed cluster)
echo "Generating Cluster protos..."
protoc \
  --proto_path=./proto \
  --go_out=paths=source_relative:./distributed/cluster/pkg/pb \
  --go-grpc_out=paths=source_relative:./distributed/cluster/pkg/pb \
  proto/cluster/v1/cluster.proto

echo "Done! Generated Cluster files:"
ls -la ./distributed/cluster/pkg/pb/

echo "All proto files generated successfully!"
