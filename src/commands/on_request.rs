use std::collections::HashMap;
use tauri::Webview;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::utils::douyin::feige_resp::{FeigeShopInfoParams, SHOP_INFO_PARAMS, REQUEST_HEADERS};
use crate::utils::is_douyin_platform;

#[derive(Debug, Deserialize, Serialize)]
pub struct MonitorRequest {
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub method: Option<String>,
    pub url: Option<String>,
    pub query: Option<Value>,
    pub headers: Option<HashMap<String, String>>,  // ← 新增
    pub body: Option<Value>,
}
#[tauri::command]
pub fn on_request(webview: Webview, payload: MonitorRequest) {
    let label = webview.label().to_string();

    if is_douyin_platform(&label) {
        if let Some(Value::Object(ref query)) = payload.query {
            let mut map = SHOP_INFO_PARAMS.lock().unwrap();
            let params = map.entry(label.clone()).or_default();
            params.update_from_query(query);
        }
    }

    if let Some(headers) = payload.headers {
        let mut map = REQUEST_HEADERS.lock().unwrap();
        let entry = map.entry(label).or_default();
        for (k, v) in headers {
            if !v.is_empty() {
                entry.insert(k, v);
            }
        }
    }
}
