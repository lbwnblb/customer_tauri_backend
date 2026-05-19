use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 前端统一发过来的结构，event 区分类型，其余字段按需取
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsEvent {
    pub event: String,           // connect | open | close | error | message
    pub timestamp: u64,
    // connect / open / close / error 携带
    pub url: Option<String>,
    pub token: Option<String>,
    pub aid: Option<String>,
    pub device_id: Option<String>,
    // close 携带
    pub code: Option<u16>,
    pub reason: Option<String>,
    // message 携带
    pub direction: Option<String>,   // incoming | outgoing
    pub data_type: Option<String>,   // text | binary
    pub payload: Option<String>,
}

#[tauri::command]
pub fn on_ws(event: WsEvent) {
    match event.event.as_str() {
        "connect" => {
            info!("[WS] 连接建立: {}", event.url.as_deref().unwrap_or(""));
        }
        "open" => {
            info!("[WS] 连接已打开");
        }
        "close" => {
            info!("[WS] 连接关闭: code={:?} reason={:?}", event.code, event.reason);
        }
        "error" => {
            info!("[WS] 连接错误");
        }
        "message" => {
            let dir = event.direction.as_deref().unwrap_or("?");
            let dtype = event.data_type.as_deref().unwrap_or("?");
            let payload = event.payload.as_deref().unwrap_or("");

            // 只关心收到的消息
            if dir == "incoming" {
                info!("[WS] ← {} ({}) {} bytes", dir, dtype, payload.len());
                // TODO: 在这里解析 protobuf、匹配规则、触发自动化
            }
        }
        other => {
            info!("[WS] 未知事件: {}", other);
        }
    }
}