use tauri::ipc::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static SHOP_CHANNELS: LazyLock<Mutex<HashMap<String, Channel<ShopTask>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopTask {
    pub webview_id: String,
    pub shop_task_type: u8,
    pub data_str: String
}
//店铺专用channel
#[tauri::command]
pub async fn add_shop_channel(channel: Channel<ShopTask>, webview_id: String) {
    log::info!("[add_shop_channel] called, webview_id={}", webview_id);
    SHOP_CHANNELS.lock().unwrap().insert(webview_id.clone(), channel);
    log::info!("[shop_channel] 已注册 channel, webview_id={}", webview_id);
}

pub fn send_shop_task(webview_id: &str, task: ShopTask) {
    if let Some(channel) = SHOP_CHANNELS.lock().unwrap().get(webview_id) {
        let _ = channel.send(task);
    }
}



// pub const TASK_TYPE_FETCH_SHOP_LIST: &str = "fetchShopList";
//

// pub fn send_task(task: Task) {
//     println!("send_task: {:?}", task);
//     if let Some(channel) = CHANNEL.lock().unwrap().as_ref() {
//         println!("准备发送 send_task: {:?}", task);
//         let _ = channel.send(task);
//
//     }
// }
//
// fn chrono_now() -> String {
//     std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .map(|d| d.as_secs().to_string())
//         .unwrap_or_else(|_| "unknown".into())
// }
