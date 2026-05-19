use tauri::Webview;

#[tauri::command]
pub fn on_request(webview: Webview, payload: String) {
    log::info!("[on_request][{}] {}", webview.label(), payload);
}
