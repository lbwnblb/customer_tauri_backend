use std::sync::{LazyLock, Mutex};

use tauri::{Manager, Window};

use crate::webview::creator::get_webview_ids;

/// 当前激活(显示在右侧可视区)的 08 webview ID。
/// 用于:
/// 1. 切换时知道把谁挪走
/// 2. 窗口 resize 时只把激活的那个调整到可视区,其他保持在"停车位"
static ACTIVE_08_WEBVIEW: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));

/// 获取当前激活的 08 webview ID
pub fn get_active_08() -> Option<String> {
    ACTIVE_08_WEBVIEW.lock().unwrap().clone()
}

/// 设置当前激活的 08 webview ID(仅更新状态,不动 webview)
pub fn set_active_08(id: Option<String>) {
    *ACTIVE_08_WEBVIEW.lock().unwrap() = id;
}

/// 计算右侧激活区 bounds(右侧 80% 区域,作为可视区)
pub fn active_bounds(w: f64, h: f64) -> tauri::Rect {
    tauri::Rect {
        position: tauri::LogicalPosition::new(w * 0.2, 0.0).into(),
        size: tauri::LogicalSize::new(w * 0.8, h).into(),
    }
}

/// 计算"停车位" bounds(父窗口右下角 1×1 像素)。
///
/// 关键点:
/// - **不调用 `hide()`**,所以 WebView2 的 `document.visibilityState` 保持 `"visible"`,
///   不触发后台节流(setInterval / WebSocket 等正常运行)。
/// - 位置必须在父窗口几何范围内,避免依赖"超出父窗口"的未定义行为。
/// - 用 1×1 而不是 0×0,有些平台对 0 尺寸有特殊处理,1×1 更稳。
pub fn parked_bounds(w: f64, h: f64) -> tauri::Rect {
    tauri::Rect {
        position: tauri::LogicalPosition::new((w - 1.0).max(0.0), (h - 1.0).max(0.0)).into(),
        size: tauri::LogicalSize::new(1.0, 1.0).into(),
    }
}

/// 把指定 08 webview 切换为激活状态:
/// - 目标 webview 移动到右侧可视区(全尺寸)
/// - 其他所有 08 webview 停到角落 1×1(仍 visible,后台脚本继续跑)
/// - 更新 ACTIVE_08_WEBVIEW 状态
pub fn activate_08_webview(window: &Window, target_id: &str) {
    let (w, h) = window_logical_size(window);
    let active = active_bounds(w, h);
    let parked = parked_bounds(w, h);

    for id in get_webview_ids() {
        if !id.starts_with("08") {
            continue;
        }
        if let Some(webview) = window.get_webview(&id) {
            let bounds = if id == target_id { active.clone() } else { parked.clone() };
            if let Err(e) = webview.set_bounds(bounds) {
                log::warn!("[activate_08_webview] set_bounds 失败 id={id}: {e}");
            }
        }
    }

    set_active_08(Some(target_id.to_string()));
}

/// 把除了 `keep_id` 之外的所有 08 webview 停到停车位。
/// 不动 `keep_id`(因为它可能正要被创建,新建时已经在激活位置)。
/// 用于:在新建一个 webview 之前,先把当前 active 的让位。
pub fn park_other_08_webviews(window: &Window, keep_id: &str) {
    let (w, h) = window_logical_size(window);
    let parked = parked_bounds(w, h);

    for id in get_webview_ids() {
        if !id.starts_with("08") || id == keep_id {
            continue;
        }
        if let Some(webview) = window.get_webview(&id) {
            if let Err(e) = webview.set_bounds(parked.clone()) {
                log::warn!("[park_other_08_webviews] set_bounds 失败 id={id}: {e}");
            }
        }
    }
}

/// 把所有 08 webview 停到停车位(没有激活)
pub fn park_all_08_webviews(window: &Window) {
    let (w, h) = window_logical_size(window);
    let parked = parked_bounds(w, h);

    for id in get_webview_ids() {
        if !id.starts_with("08") {
            continue;
        }
        if let Some(webview) = window.get_webview(&id) {
            let _ = webview.set_bounds(parked.clone());
        }
    }

    set_active_08(None);
}

/// 工具函数:获取窗口逻辑尺寸
fn window_logical_size(window: &Window) -> (f64, f64) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let size = window.inner_size().unwrap_or_default();
    let w = size.width as f64 / scale;
    let h = size.height as f64 / scale;
    (w, h)
}
