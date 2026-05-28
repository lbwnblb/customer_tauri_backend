use tauri::{Emitter, Manager, Window};

use crate::commands::webview_utils::activate_08_webview;
use crate::webview::creator::{create_qa_webview, has_webview};

const QA_WEBVIEW_ID: &str = "08_qa";

#[tauri::command]
pub async fn add_qa(window: Window) {
    if has_webview(&window, QA_WEBVIEW_ID) {
        activate_08_webview(&window, QA_WEBVIEW_ID);
        window.app_handle().emit("refresh-shops", ()).unwrap();
    } else {
        let scale = window.scale_factor().unwrap_or(1.0);
        let size = window.inner_size().unwrap_or_default();
        let w = size.width as f64 / scale;
        let h = size.height as f64 / scale;
        if let Err(e) = create_qa_webview(&window, w, h) {
            log::error!("[add_qa] 创建问答 webview 失败: {e}");
        }
    }
}
