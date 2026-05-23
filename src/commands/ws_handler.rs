use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::utils::protobuf::{feige_im_recv, feige_im_send};
use crate::utils::feige_resp::{IM_TOKEN_MAP, IM_DEVICE_ID_MAP};
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

fn extract_raw_body(request: &Request) -> Option<Vec<u8>> {
    match request.body() {
        InvokeBody::Raw(data) => Some(data.clone()),
        InvokeBody::Json(_) => {
            info!("[WS] 收到非 Raw body，跳过");
            None
        }
    }
}

#[tauri::command]
pub fn on_ws_recv(webview: Webview, request: Request) {
    let Some(bytes) = extract_raw_body(&request) else { return };
    tauri::async_runtime::spawn(async move {
        feige_im_recv(&webview, &bytes).await;
    });
}

#[tauri::command]
pub fn on_ws_send(webview: Webview, request: Request) {
    let Some(bytes) = extract_raw_body(&request) else { return };
    tauri::async_runtime::spawn(async move {
        feige_im_send(&webview, &bytes).await;
    });
}

#[tauri::command]
pub fn on_ws(webview: Webview, event: WsEvent) {
    match event.event.as_str() {
        "connect" => {
            info!("[WS] 连接建立: {}", event.url.as_deref().unwrap_or(""));
            // 存储 token 和 device_id，供 get_by_conversation 使用
            if let Some(token) = &event.token {
                if !token.is_empty() {
                    IM_TOKEN_MAP.lock().unwrap().insert(webview.label().to_string(), token.clone());
                    info!("[WS] 已存储 IM token for webview={}", webview.label());
                }
            }
            if let Some(device_id) = &event.device_id {
                if !device_id.is_empty() {
                    IM_DEVICE_ID_MAP.lock().unwrap().insert(webview.label().to_string(), device_id.clone());
                    info!("[WS] 已存储 device_id={} for webview={}", device_id, webview.label());
                }
            }
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