use log::info;
use serde::Deserialize;
use tauri::{command, Webview};

use crate::utils::feige_resp::PIGEON_SIGN_MAP;

#[derive(Debug, Deserialize)]
struct LinkInfoResponse {
    data: Option<LinkInfoData>,
}

#[derive(Debug, Deserialize)]
struct LinkInfoData {
    #[serde(rename = "pigeon_sign")]
    pigeon_sign: Option<String>,
}

#[command]
pub async fn on_get_link_info(webview: Webview, url: String, status: u16, body: String) {
    match serde_json::from_str::<LinkInfoResponse>(&body) {
        Ok(resp) => {
            if let Some(data) = resp.data {
                if let Some(sign) = data.pigeon_sign {
                    let label = webview.label().to_string();
                    PIGEON_SIGN_MAP.lock().unwrap().insert(label, sign.clone());
                    info!("[LinkInfoInterceptor] pigeon_sign: {}", sign);
                }
            }
        }
        Err(e) => {
            info!("[LinkInfoInterceptor] JSON 解析失败: {}", e);
        }
    }
}
