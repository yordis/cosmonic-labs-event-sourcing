fn main() {
    prost_build::compile_protos(&["../aggregate/proto/bank.proto"], &["../aggregate/proto/"]).unwrap();
} 