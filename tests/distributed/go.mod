module github.com/uttom-akash/vector-xlite/tests/distributed

go 1.22

replace github.com/uttom-akash/vector-xlite/distributed/cluster => ../../distributed/cluster

require github.com/uttom-akash/vector-xlite/distributed/cluster v0.0.0-00010101000000-000000000000

require (
	github.com/hashicorp/raft v1.7.1 // indirect
	google.golang.org/grpc v1.69.2 // indirect
	google.golang.org/protobuf v1.35.2 // indirect
)
