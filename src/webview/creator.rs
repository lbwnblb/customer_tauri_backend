use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use tauri::webview::WebviewBuilder;
use tauri::{Manager, Url, WebviewUrl, Window};

use crate::commands::webview_utils::{park_other_08_webviews, set_active_08};
use crate::config;
use crate::database;
use crate::scripts;
use crate::utils;

static WEBVIEW_IDS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn get_webview_ids() -> Vec<String> {
    WEBVIEW_IDS.lock().unwrap().clone()
}

pub fn has_webview(window: &Window, label: &str) -> bool {
    window.get_webview(label).is_some()
}

/// 从 WEBVIEW_IDS 中移除指定 id(供 delete_shop 等清理逻辑调用)
pub fn remove_webview_id(id: &str) {
    let mut ids = WEBVIEW_IDS.lock().unwrap();
    ids.retain(|x| x != id);
}

pub fn create_app_webview(window: &Window, w: f64, h: f64) -> Result<(), tauri::Error> {
    let id = "02_app".to_string();
    window.add_child(
        WebviewBuilder::new(&id, WebviewUrl::App("index.html".into())),
        tauri::LogicalPosition::new(0.0, 0.0),
        tauri::LogicalSize::new(w * 0.2, h),
    )?;
    WEBVIEW_IDS.lock().unwrap().push(id);
    Ok(())
}

/// 创建 platform webview。
/// 新创建的 webview 直接占据右侧可视区(成为 active),
/// 创建之前先把已有的 08 webview 停到角落。
pub fn create_platform_webview(window: &Window, w: f64, h: f64) -> Result<(), tauri::Error> {
    let id = "08pf".to_string();
    log::info!(
        "[create_platform_webview] 开始创建平台 webview, id={}, 位置=({}, {}), 尺寸=({:.2}, {:.2})",
        id,
        w * 0.2,
        0.0,
        w * 0.8,
        h
    );

    // 先把所有现有 08 webview 停车,让出激活位
    park_other_08_webviews(window, &id);

    window.add_child(
        WebviewBuilder::new(&id, WebviewUrl::App("platform.html".into())).devtools(true),
        tauri::LogicalPosition::new(w * 0.2, 0.0),
        tauri::LogicalSize::new(w * 0.8, h),
    )?;
    log::info!(
        "[create_platform_webview] webview id 已注册, id={}, 当前已注册列表: {:?}",
        id,
        WEBVIEW_IDS.lock().unwrap()
    );
    WEBVIEW_IDS.lock().unwrap().push(id.clone());

    // 新创建的就是当前激活
    set_active_08(Some(id));

    Ok(())
}

/// 创建背景 webview(应用启动时的默认底图)。
/// 创建后即设为 active,使其覆盖整个右侧区域。
pub fn create_bg_webview(window: &Window, w: f64, h: f64) -> Result<(), tauri::Error> {
    let id = "08bg".to_string();

    park_other_08_webviews(window, &id);

    window.add_child(
        WebviewBuilder::new(&id, WebviewUrl::App("bg.html".into())),
        tauri::LogicalPosition::new(w * 0.2, 0.0),
        tauri::LogicalSize::new(w * 0.8, h),
    )?;
    WEBVIEW_IDS.lock().unwrap().push(id.clone());

    set_active_08(Some(id));

    Ok(())
}

/// 创建抖音店铺 webview。
/// 新 webview 直接占据激活位,旧的 active 停到角落。
/// 注意:已存在的 webview(包括停车位上的)仍在 visible 状态,
/// 后台脚本(自动回复消息、WebSocket 心跳)持续运行,不会被节流。
pub fn create_douyin_webview(
    window: &Window,
    w: f64,
    h: f64,
    id: Option<&str>,
) -> Result<String, tauri::Error> {
    let id = id
        .map(String::from)
        .unwrap_or_else(|| format!("08_douyin_{}", utils::uuid_no_hyphen()));

    let data_dir = PathBuf::from(utils::app_data_dir()).join(&id);
    fs::create_dir_all(&data_dir)?;

    // 先把所有现有 08 webview 停车,让出激活位
    park_other_08_webviews(window, &id);

    window.add_child(
        WebviewBuilder::new(
            &id,
            WebviewUrl::External(
                Url::parse(config::FEIGE_KEFU_URL).expect("invalid feige kefu url"),
            ),
        ).initialization_script(&scripts::redirect::douyin_redirect(&id))
        .initialization_script(&scripts::feige_intercept::create_intercepted_webview())
            .initialization_script(&scripts::ws_hook::create_ws_hook())
            .initialization_script(&scripts::http_interceptor::create_http_response_hook())
            .initialization_script(&scripts::link_info_interceptor::create_link_info_hook())
        .data_directory(data_dir),
        tauri::LogicalPosition::new(w * 0.2, 0.0),
        tauri::LogicalSize::new(w * 0.8, h),
    )?;
    WEBVIEW_IDS.lock().unwrap().push(id.clone());

    set_active_08(Some(id.clone()));

    Ok(id)
}

