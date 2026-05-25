use tauri::{Manager, WindowBuilder};
use crate::utils::app_data::app_data_dir_home_index;
use crate::utils::backend::verify_token;
use crate::utils::cookie::get_cookies;
use crate::webview::creator::{create_app_webview, create_auth_webview, create_platform_webview};

mod commands;
mod config;
mod database;
mod scripts;
mod utils;
mod webview;
mod window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::init_logger();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::greet::greet, commands::on_request::on_request, commands::shop::shop_account, commands::shop::add_shop, commands::shop::delete_shop, commands::shop::shop_list, commands::shop::select_shop, commands::platform::select_platform,  commands::shop_callback::shop_name_callback, commands::ws_handler::on_ws, commands::ws_handler::dy_ws_recv, commands::ws_handler::dy_ws_send, commands::ws_handler::pdd_ws_recv, commands::ws_handler::pdd_ws_send, commands::http_response_intercepted::on_http_response_intercepted, commands::get_link_info::on_get_link_info, commands::pdd_send::pdd_send_message, commands::pdd_send::pdd_crypto_callback, commands::auth::on_login_success, commands::auth::on_logout])
        .setup(|app| {
            let monitor = app.primary_monitor()?.expect("找不到主显示器");
            let screen_size = monitor.size();
            let width = screen_size.width as f64 * 0.65;
            let height = screen_size.height as f64 * 0.5;

            let window = WindowBuilder::new(app, "main")
                .title("我的应用")
                .inner_size(width, height)
                .center()
                .build()?;


            let scale_factor = window.scale_factor()?;
            let size = window.inner_size()?;
            let w = size.width as f64 / scale_factor;
            let h = size.height as f64 / scale_factor;
            let token = get_cookies(&app_data_dir_home_index())
                .ok()
                .and_then(|cookies| {
                    cookies.into_iter()
                        .find(|c| c.name == "Authorization")
                        .map(|c| c.value)
                })
                .filter(|v| !v.is_empty());

            let needs_auth = match token {
                None => {
                    log::info!("[startup] 未找到 Authorization cookie，跳转登录页");
                    true
                }
                Some(token) => {
                    match tauri::async_runtime::block_on(verify_token(&token)) {
                        Ok(data) => {
                            log::info!(
                                "[startup] token 有效, user_id={}, expire_at={}",
                                data.user_id,
                                data.expire_at
                            );
                            false
                        }
                        Err(e) => {
                            log::warn!("[startup] token 验证失败: {e}，跳转登录页");
                            true
                        }
                    }
                }
            };

            if needs_auth {
                create_auth_webview(&window, w, h)?;
            } else {
                create_app_webview(&window, w, h)?;
                create_platform_webview(&window, w, h)?;
            }

            window::on_window_resized(&window);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}