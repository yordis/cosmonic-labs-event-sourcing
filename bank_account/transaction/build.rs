fn main() {
    prost_build::compile_protos(&["../proto/bank.proto"], &["../proto/"]).unwrap();
} 