use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::utils::protobuf::feige_im_proto;
use tauri::{Webview, ipc::{InvokeBody, Request}};

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
pub fn on_ws_binary(webview: Webview, request: Request) {
    let bytes: Vec<u8> = match request.body() {
        InvokeBody::Raw(data) => data.clone(),
        InvokeBody::Json(_val) => {
            info!("[WS] on_ws_binary 收到非 Raw body，跳过");
            return;
        }
    };

    tauri::async_runtime::spawn(async move {
        feige_im_proto(&webview, &bytes).await;
    });
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
            info!("[WS] 收到消息: {}", event.payload.as_deref().unwrap_or(""));
        }
        other => {
            info!("[WS] 未知事件: {}", other);
        }
    }
}