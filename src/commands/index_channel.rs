use tauri::ipc::Channel;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static CHANNEL: LazyLock<Mutex<Option<Channel<Task>>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub r#type: u8,
    pub task_type: String,
}

pub const TASK_TYPE_FETCH_SHOP_LIST: &str = "fetchShopList";

#[tauri::command]
pub async fn start_backend_channel(channel: Channel<Task>) {
    *CHANNEL.lock().unwrap() = Some(channel);
}

pub fn send_task(task: Task) {
    log::info!("send_task: {:?}", task);
    if let Some(channel) = CHANNEL.lock().unwrap().as_ref() {
        log::info!("准备发送 send_task: {:?}", task);
        let _ = channel.send(task);

    }
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".into())
}
