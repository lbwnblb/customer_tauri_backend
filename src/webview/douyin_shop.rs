use tauri::Window;

use crate::database::save_shop_webview;
use crate::webview::creator::{create_douyin_webview, create_pinduoduo_webview};

/// 打开一个新的抖音店铺 webview。
/// `create_douyin_webview` 内部已经处理:
/// - 把当前激活的 08 webview 停到角落(不调用 hide,保持 visible 状态)
/// - 新创建的 webview 直接占据可视区,成为 active
pub fn open_douyin_shop(window: &Window) -> Result<(), String> {
    let scale = window.scale_factor().unwrap_or(1.0);
    let size = window.inner_size().unwrap();
    let w = size.width as f64 / scale;
    let h = size.height as f64 / scale;

    let id = create_douyin_webview(&window, w, h, None)
        .map_err(|e| format!("创建抖音 webview 失败: {e}"))?;

    save_shop_webview(&id, "未登录")
        .map_err(|e| format!("保存 webview id 到数据库失败: {e}"))
}

/// 打开一个新的拼多多店铺 webview。
pub fn open_pinduoduo_shop(window: &Window) -> Result<(), String> {
    let scale = window.scale_factor().unwrap_or(1.0);
    let size = window.inner_size().unwrap();
    let w = size.width as f64 / scale;
    let h = size.height as f64 / scale;

    let id = create_pinduoduo_webview(&window, w, h, None)
        .map_err(|e| format!("创建拼多多 webview 失败: {e}"))?;

    save_shop_webview(&id, "未登录")
        .map_err(|e| format!("保存 webview id 到数据库失败: {e}"))
}
