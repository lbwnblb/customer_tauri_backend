use tauri::ipc::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::Webview;
use crate::database::update_shop_name;
use crate::utils::feige_resp::feige_shop_info;
use crate::commands::index_channel::{send_task, Task, TASK_TYPE_FETCH_SHOP_LIST};

static SHOP_CHANNELS: LazyLock<Mutex<HashMap<String, Channel<ShopTask>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopTask {
    pub shop_task_type: u8,
    pub data_str: String
}
//店铺专用channel
#[tauri::command]
pub async fn add_shop_channel(webview: Webview,channel: Channel<ShopTask>, webview_id: String) {
    let label = webview.label().to_string();

    match feige_shop_info(&label, &webview).await {
        Ok(info) => {
            if let Some(data) = info.data {
                if let Some(shop_name) = data.shop_name {
                    if let Err(e) = update_shop_name(&label, &shop_name) {
                        log::error!("[add_shop_channel] 更新店铺名称失败: {}", e);
                    } else {
                        send_task(Task {
                            r#type: 1,
                            task_type: TASK_TYPE_FETCH_SHOP_LIST.to_string(),
                        });
                    }
                }
            }
        }
        Err(e) => {
            log::error!("[add_shop_channel] shop_info 获取失败: {}", e);
        }
    };

    log::info!("[add_shop_channel] called, webview_id={}", webview_id);
    SHOP_CHANNELS.lock().unwrap().insert(webview_id.clone(), channel);
    log::info!("[shop_channel] 已注册 channel, webview_id={}", webview_id);
}

pub fn send_shop_task(webview_id: &str, task: ShopTask) {
    if let Some(channel) = SHOP_CHANNELS.lock().unwrap().get(webview_id) {
        let _ = channel.send(task);
    }
}
// fn chrono_now() -> String {
//     std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .map(|d| d.as_secs().to_string())
//         .unwrap_or_else(|_| "unknown".into())
// }
