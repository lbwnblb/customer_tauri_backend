use tauri::{Manager, Window};

use crate::webview::creator::{create_platform_webview, has_webview};
use crate::commands::webview_utils::activate_08_webview;

// 必须用 async：同步 command 跑在 blocking 线程池，
// 在那里调 add_child 会触发 wry 的线程检查导致 crash。
// 官方对动态多 webview 的支持要求 async command。
#[tauri::command]
pub async fn select_platform(window: Window) {
    if has_webview(&window, "08pf") {
        // 已存在:切换激活(08pf 移到可视区,其他停到角落)
        activate_08_webview(&window, "08pf");
    } else {
        // 不存在:创建(创建函数内部会处理停车并设为 active)
        let scale = window.scale_factor().unwrap_or(1.0);
        let size = window.inner_size().unwrap();
        let w = size.width as f64 / scale;
        let h = size.height as f64 / scale;
        if let Err(e) = create_platform_webview(&window, w, h) {
            log::error!("[select_platform] 创建 platform webview 失败: {e}");
        }
    }
}
