use tauri::{Manager, Window};

use crate::webview::creator::{create_app_webview, create_platform_webview, remove_webview_id};

/// 登录成功后由 auth.html 调用。
///
/// 流程：
/// 1. 创建正常工作区（02_app + 08pf）
#[tauri::command]
pub async fn on_login_success(window: Window) {
    log::info!("[on_login_success] 收到登录成功回调");

    // 关闭 auth webview
    if let Some(auth_wv) = window.get_webview("00_auth") {
        if let Err(e) = auth_wv.close() {
            log::error!("[on_login_success] 关闭 auth webview 失败: {e}");
        }
    }
    remove_webview_id("00_auth");

    let scale = window.scale_factor().unwrap_or(1.0);
    let size = window.inner_size().unwrap_or_default();
    let w = size.width as f64 / scale;
    let h = size.height as f64 / scale;

    if let Err(e) = create_app_webview(&window, w, h) {
        log::error!("[on_login_success] 创建 app webview 失败: {e}");
        return;
    }

    if let Err(e) = create_platform_webview(&window, w, h) {
        log::error!("[on_login_success] 创建 platform webview 失败: {e}");
    }
}

/// 从前端传入 token，验证有效性。
/// 有效时直接触发 on_login_success 完成跳转；无效时返回 Err。
#[tauri::command]
pub async fn check_token(token: String, window: Window) -> Result<(), String> {
    match crate::utils::backend::verify_token(&token).await {
        Ok(data) => {
            log::info!("[check_token] token 有效, user_id={}", data.user_id);
            crate::utils::backend::set_app_token(&token);
            tauri::async_runtime::spawn(async move {
                on_login_success(window).await;
            });
            Ok(())
        }
        Err(e) => {
            log::warn!("[check_token] token 验证失败: {e}");
            Err(e.to_string())
        }
    }
}

/// 主动登出：清除本地 token 并切换回登录页。
///
/// 流程：
/// 1. 清除本地 token
/// 2. 关闭所有非 auth webview
/// 3. 创建 auth webview
#[tauri::command]
pub async fn on_logout(window: Window) {
    log::info!("[on_logout] 执行登出");

    // 关闭所有已注册 webview（02_app、08* 等）
    for id in crate::webview::creator::get_webview_ids() {
        if let Some(wv) = window.get_webview(&id) {
            if let Err(e) = wv.close() {
                log::warn!("[on_logout] 关闭 webview {id} 失败: {e}");
            }
        }
        remove_webview_id(&id);
    }

    let scale = window.scale_factor().unwrap_or(1.0);
    let size = window.inner_size().unwrap_or_default();
    let w = size.width as f64 / scale;
    let h = size.height as f64 / scale;

    if let Err(e) = crate::webview::creator::create_auth_webview(&window, w, h) {
        log::error!("[on_logout] 创建 auth webview 失败: {e}");
    }
}
