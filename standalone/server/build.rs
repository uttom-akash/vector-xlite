fn main() {
    tonic_prost_build::compile_protos("../../proto/vectorxlite.proto").unwrap();
}
