use tauri::Window;

use crate::commands::webview_utils::activate_08_webview;
use crate::webview::creator::{create_redeem_webview, has_webview};

const REDEEM_WEBVIEW_ID: &str = "08_redeem";

#[tauri::command]
pub async fn redeem_query(window: Window) {
    if has_webview(&window, REDEEM_WEBVIEW_ID) {
        activate_08_webview(&window, REDEEM_WEBVIEW_ID);
    } else {
        let scale = window.scale_factor().unwrap_or(1.0);
        let size = window.inner_size().unwrap_or_default();
        let w = size.width as f64 / scale;
        let h = size.height as f64 / scale;
        if let Err(e) = create_redeem_webview(&window, w, h) {
            log::error!("[redeem_query] 创建兑换码 webview 失败: {e}");
        }
    }
}
