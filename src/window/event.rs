use tauri::{Manager, Window, WindowEvent};

use crate::commands::webview_utils::{active_bounds, get_active_08, parked_bounds};
use crate::webview::creator::get_webview_ids;

/// 监听窗口 resize:
/// - `02_app` 固定在左侧 20% 区域
/// - 当前激活的 08 webview 调到右侧 80% 可视区
/// - 其他 08 webview 始终保持在 1×1 停车位(跟随父窗口右下角移动)
///
/// 注意:停车位的 webview 也要跟随 resize 重新计算位置,
/// 否则父窗口缩小后它们可能跑出父窗口范围之外。
pub fn on_window_resized(window: &Window) {
    let win = window.clone();

    window.on_window_event(move |event| {
        if let WindowEvent::Resized(_) = event {
            let scale = win.scale_factor().unwrap_or(1.0);
            let size = win.inner_size().unwrap();
            let w = size.width as f64 / scale;
            let h = size.height as f64 / scale;

            let active_id = get_active_08();
            let active = active_bounds(w, h);
            let parked = parked_bounds(w, h);

            for id in get_webview_ids() {
                let Some(webview) = win.get_webview(&id) else {
                    continue;
                };

                if id.starts_with("00") {
                    // 登录页,全窗口
                    let _ = webview.set_position(tauri::LogicalPosition::new(0.0, 0.0));
                    let _ = webview.set_size(tauri::LogicalSize::new(w, h));
                } else if id.starts_with("02") {
                    // 左侧导航,固定 20% 宽
                    let _ = webview.set_position(tauri::LogicalPosition::new(0.0, 0.0));
                    let _ = webview.set_size(tauri::LogicalSize::new(w * 0.2, h));
                } else if id.starts_with("08") {
                    // 右侧:active 占可视区,其他保持停车位
                    let bounds = if Some(&id) == active_id.as_ref() {
                        active.clone()
                    } else {
                        parked.clone()
                    };
                    let _ = webview.set_bounds(bounds);
                }
            }
        }
    });
}
