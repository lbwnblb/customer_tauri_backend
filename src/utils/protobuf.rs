use prost::Message;

pub mod im_proto {
    include!(concat!(env!("OUT_DIR"), "/dy_im_proto.rs"));
}
pub fn feige_im_proto(hex_str: &str) {
    let bytes = hex::decode(hex_str).unwrap();

    // 第一层：Frame
    let frame = im_proto::Frame::decode(bytes.as_slice()).unwrap();
    println!("=== Frame ===");
    println!("seqid: {}", frame.seqid);
    println!("logid: {}", frame.logid);
    println!("service: {}", frame.service);
    println!("method: {}", frame.method);
    for h in &frame.headers {
        println!("header: {} = {}", h.key, h.value);
    }
    println!("payload_encoding: {:?}", frame.payload_encoding);
    println!("payload_type: {:?}", frame.payload_type);

    // 第二层：Response（从 payload 里解）
    if let Some(payload) = &frame.payload {
        match im_proto::Response::decode(payload.as_slice()) {
            Ok(response) => {
                println!("\n=== Response ===");
                println!("cmd: {:?}", response.cmd);
                println!("sequence_id: {:?}", response.sequence_id);
                println!("status_code: {:?}", response.status_code);
                println!("error_desc: {:?}", response.error_desc);
                println!("inbox_type: {:?}", response.inbox_type);
                println!("log_id: {:?}", response.log_id);

                // 第三层：ResponseBody（自动递归解出来）
                if let Some(body) = &response.body {
                    println!("\n=== ResponseBody ===");
                    println!("{:#?}", body);
                }
            }
            Err(e) => {
                println!("Response 解码失败: {}", e);
            }
        }
    }
}