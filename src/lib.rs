use tauri::WindowBuilder;
use crate::webview::creator::create_auth_webview;

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
        .invoke_handler(tauri::generate_handler![commands::greet::greet, commands::on_request::on_request, commands::shop::shop_account, commands::shop::add_shop, commands::shop::delete_shop, commands::shop::shop_list, commands::shop::select_shop, commands::platform::select_platform,  commands::shop_callback::shop_name_callback, commands::ws_handler::on_ws, commands::ws_handler::dy_ws_recv, commands::ws_handler::dy_ws_send, commands::ws_handler::pdd_ws_recv, commands::ws_handler::pdd_ws_send, commands::http_response_intercepted::on_http_response_intercepted, commands::get_link_info::on_get_link_info, commands::pdd_send::pdd_send_message, commands::pdd_send::pdd_crypto_callback, commands::auth::on_login_success, commands::auth::on_logout, commands::auth::check_token, commands::promotion_h5::on_promotion_pack_h5, commands::account::account_query, commands::redeem::redeem_query, commands::qa::add_qa])
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
            create_auth_webview(&window, w, h)?;

            window::on_window_resized(&window);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}