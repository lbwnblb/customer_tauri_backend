use tauri::{Manager, WindowBuilder};
use crate::webview::creator::{create_app_webview, create_bg_webview};

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
        .invoke_handler(tauri::generate_handler![commands::greet::greet, commands::on_request::on_request, commands::shop::shop_account, commands::shop::add_shop, commands::shop::delete_shop, commands::shop::shop_list, commands::shop::select_shop, commands::platform::select_platform, commands::index_channel::start_backend_channel, commands::shop_channel::add_shop_channel, commands::ws_handler::on_ws, commands::ws_handler::on_ws_binary, commands::http_response_intercepted::on_http_response_intercepted, commands::get_link_info::on_get_link_info])
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
            create_app_webview(&window, w, h)?;


            create_bg_webview(&window, w, h)?;

            window::on_window_resized(&window);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}