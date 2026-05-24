fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    std::env::set_var("PROTOC", protoc);

    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &["proto/dy_im_proto.proto","proto/titan.proto"],
            &["proto/"]
        ).unwrap();
    tauri_build::build()
}