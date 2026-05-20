fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();

    std::env::set_var("PROTOC", protoc);  // 告诉 prost-build 用这个 protoc

    prost_build::compile_protos(
        &["src/proto/dy_im.proto"],
        &["src/proto/"]
    ).unwrap();
    tauri_build::build()
}
