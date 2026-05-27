use tauri::{Manager, Window};

use crate::commands::webview_utils::activate_08_webview;
use crate::webview::creator::{create_account_webview, has_webview};

const ACCOUNT_WEBVIEW_ID: &str = "08_account";

#[tauri::command]
pub async fn account_query(window: Window) {
    if has_webview(&window, ACCOUNT_WEBVIEW_ID) {
        activate_08_webview(&window, ACCOUNT_WEBVIEW_ID);
        if let Some(webview) = window.get_webview(ACCOUNT_WEBVIEW_ID) {
            if let Err(e) = webview.eval("refreshAccountInfo()") {
                log::error!("[account_query] eval refreshAccountInfo 失败: {e}");
            }
        }
    } else {
        let scale = window.scale_factor().unwrap_or(1.0);
        let size = window.inner_size().unwrap_or_default();
        let w = size.width as f64 / scale;
        let h = size.height as f64 / scale;
        if let Err(e) = create_account_webview(&window, w, h) {
            log::error!("[account_query] 创建账号 webview 失败: {e}");
        }
    }
}
