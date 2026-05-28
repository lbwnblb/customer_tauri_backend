use tauri::Window;
use crate::database::get_all_shops;
use crate::utils::{is_douyin_platform, is_pinduoduo_platform};
use crate::webview::creator::{create_douyin_webview, create_pinduoduo_webview, has_webview};

#[tauri::command]
pub async fn one_click_login(window: Window) {
    let shops = match get_all_shops() {
        Ok(s) => s,
        Err(e) => {
            log::error!("[one_click_login] 获取店铺列表失败: {e}");
            return;
        }
    };

    let scale = window.scale_factor().unwrap_or(1.0);
    let size = window.inner_size().unwrap();
    let w = size.width as f64 / scale;
    let h = size.height as f64 / scale;

    for shop in shops {
        if has_webview(&window, &shop.id) {
            log::info!("[one_click_login] 店铺 webview 已存在: {}", shop.id);
            continue;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        if is_douyin_platform(&shop.id) {
            match create_douyin_webview(&window, w, h, Some(&shop.id)) {
                Ok(_) => log::info!("[one_click_login] 创建抖音 webview 成功: {}", shop.id),
                Err(e) => log::error!("[one_click_login] 创建抖音 webview 失败: {} - {e}", shop.id),
            }
        } else if is_pinduoduo_platform(&shop.id) {
            match create_pinduoduo_webview(&window, w, h, Some(&shop.id)) {
                Ok(_) => log::info!("[one_click_login] 创建拼多多 webview 成功: {}", shop.id),
                Err(e) => log::error!("[one_click_login] 创建拼多多 webview 失败: {} - {e}", shop.id),
            }
        } else {
            log::warn!("[one_click_login] 未知平台 shop_id: {}", shop.id);
        }
    }
}
