fn main() {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    prost_build::compile_protos(&["proto/dy_im_proto.proto"], &["proto/"]).unwrap();
    tauri_build::build()
}
