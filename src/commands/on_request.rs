use tauri::Webview;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct MonitorRequest {
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub method: Option<String>,
    pub url: Option<String>,
    pub query: Option<Value>,
    pub body: Option<Value>,
}
#[tauri::command]
pub fn on_request(webview: Webview, payload: MonitorRequest) {
    log::info!("[on_request][{}] {:?}", webview.label(), payload);
}
