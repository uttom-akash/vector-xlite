module github.com/uttom-akash/go_examples

go 1.25.3

replace github.com/uttom-akash/vector-xlite/go_grpc_client => ../../vector_xlite_go_client

require github.com/uttom-akash/vector-xlite/go_grpc_client v0.0.0-00010101000000-000000000000

require (
	golang.org/x/net v0.42.0 // indirect
	golang.org/x/sys v0.34.0 // indirect
	golang.org/x/text v0.27.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20250804133106-a7a43d27e69b // indirect
	google.golang.org/grpc v1.76.0 // indirect
	google.golang.org/protobuf v1.36.10 // indirect
)
