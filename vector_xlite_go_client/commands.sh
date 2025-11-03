protoc \
  --proto_path=./grpc_proto \
  --go_out=paths=source_relative:./go_grpc_client/pb \
  --go-grpc_out=paths=source_relative:./go_grpc_client/pb \
  grpc_proto/vectorxlite.proto