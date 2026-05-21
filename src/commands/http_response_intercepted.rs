use log::{error, info};
use prost::DecodeError;
use tauri::command;
use crate::utils::protobuf::im_proto::Response;
use crate::utils::protobuf::parse_response;

/// Tauri command，接收二进制响应数据
///
/// JS 端传 Uint8Array，Tauri 自动映射为 Vec<u8>，零拷贝无 base64
#[command]
pub async fn on_http_response_intercepted(url: String, status: u16, body: Vec<u8>) {
    println!("[HttpInterceptor] 收到拦截数据:");
    println!("  URL:    {}", url);
    println!("  Status: {}", status);
    match parse_response(body.as_slice()) {
        Ok( response) => {
            // 假设 response 是 Response 类型
if let Some(body) = &response.body {
    if let Some(conv_body) = &body.messages_in_conversation_body {
        for msg in &conv_body.messages {
            // 打印消息类型和内容
            println!("[消息] type: {:?}, content: {:?}", msg.message_type, msg.content);
            
            // 或者格式化打印更多信息
            println!(
                "message_type={:?}, content={}, server_message_id={:?}, create_time={:?}",
                msg.message_type,
                msg.content.as_deref().unwrap_or(""),
                msg.server_message_id,
                msg.create_time
            );
        }
    }
    if let Some(range_body) = &body.get_message_info_by_index_v2_range_body {
        for info in &range_body.infos {
            if let Some(msg) = &info.body {
                if let Some(content) = &msg.content {
                    println!("{}", content);
                }
            }
        }
    }
}
        }
        Err(e) => {
            error!("解析响应数据错误: {}", e);
        }
    };
}