fn main() {
    tonic_prost_build::compile_protos("../grpc_proto/vectorxlite.proto").unwrap();
}
