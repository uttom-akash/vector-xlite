fn main() {
    tonic_prost_build::compile_protos("../../proto/vectorxlite/v1/vectorxlite.proto").unwrap();
}
