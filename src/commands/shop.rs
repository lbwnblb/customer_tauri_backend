use tauri::{Manager, Webview, Window};
use serde::{Deserialize, Serialize};
use crate::database::{get_all_shops, delete_shop_webview, ShopWebview};
use crate::utils::{app_data_dir, get_platform_from_id, is_douyin_platform, is_pinduoduo_platform, PLATFORM_DOUYIN, PLATFORM_PINDUODUO};
use crate::commands::webview_utils::{activate_08_webview, get_active_08};
use crate::webview::creator::{create_douyin_webview, create_pinduoduo_webview, create_platform_webview, get_webview_ids, has_webview, remove_webview_id};

#[derive(Deserialize)]
pub struct ShopPlatform {
    platform: String
}

#[derive(Serialize)]
pub struct Shop {
    id: String,
    name: String,
    platform: String,
}

#[tauri::command]
pub fn shop_list() -> Vec<Shop> {
    match get_all_shops() {
        Ok(shops) => shops.into_iter().map(|shop| Shop {
            id: shop.id.clone(),
            name: shop.shop_name,
            platform: get_platform_from_id(&shop.id),
        }).collect(),
        Err(_) => vec![],
    }
}

fn delete_cookie_dir_async(shop_id: String) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let cookie_dir = format!("{}\\{}", app_data_dir(), shop_id);
        if let Err(e) = std::fs::remove_dir_all(&cookie_dir) {
            log::error!("[delete_shop] 删除 cookie 目录失败: {} - {}", cookie_dir, e);
        }
    });
}

#[tauri::command]
pub async fn delete_shop(window: Window, shop_id: String) {
    // 如果删除的是当前激活的,选下一个候选作为新的 active
    // 候选优先级:其他店铺 webview > 08pf > 新建 08pf
    let was_active = get_active_08().as_deref() == Some(shop_id.as_str());

    if let Some(webview) = window.get_webview(&shop_id) {
        let _ = webview.close();
    }

    delete_cookie_dir_async(shop_id.clone());
    // 从注册表里移除
    remove_webview_id(&shop_id);

    // 如果删的是当前激活的,挑下一个激活
    if was_active {
        let remaining: Vec<_> = get_webview_ids()
            .into_iter()
            .filter(|id| id != "02_app" && id != "08bg" && id.starts_with("08"))
            .collect();

        if let Some(next_id) = remaining.first() {
            // 优先切到剩余的店铺/平台 webview
            activate_08_webview(&window, next_id);
        } else if has_webview(&window, "08pf") {
            activate_08_webview(&window, "08pf");
        } else {
            // 兜底:创建一个 platform webview(它会自动成为 active)
            let scale = window.scale_factor().unwrap_or(1.0);
            let size = window.inner_size().unwrap();
            let w = size.width as f64 / scale;
            let h = size.height as f64 / scale;
            if let Err(e) = create_platform_webview(&window, w, h) {
                log::error!("[delete_shop] 创建 platform webview 失败: {e}");
            }
        }
    }

    match delete_shop_webview(&shop_id) {
        Ok(_) => {}
        Err(e) => log::error!("[delete_shop] 删除店铺失败: {e}"),
    }
}

#[tauri::command]
pub fn shop_account() {
    log::info!("店铺账号");
}

#[tauri::command]
pub async fn select_shop(window: Window, shop_id: String) {
    if has_webview(&window, &shop_id) {
        // 已存在:用 activate 切换 z 序(把它移到可视区,其他停到角落)
        activate_08_webview(&window, &shop_id);
    } else {
        // 不存在:创建一个新的(创建函数内部会处理停车)
        let scale = window.scale_factor().unwrap_or(1.0);
        let size = window.inner_size().unwrap();
        let w = size.width as f64 / scale;
        let h = size.height as f64 / scale;

        if is_douyin_platform(&shop_id) {
            match create_douyin_webview(&window, w, h, Some(&shop_id)) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("[select_shop] 创建抖音 webview 失败: {e}");
                }
            }
        } else if is_pinduoduo_platform(&shop_id) {
            match create_pinduoduo_webview(&window, w, h, Some(&shop_id)) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("[select_shop] 创建拼多多 webview 失败: {e}");
                }
            }
        }
    }
}

pub(crate) fn notify_shop_list_update(webview: &Webview) {
    match webview.eval("fetchShopList()") {
        Ok(()) => {
            log::info!("[shop_name_callback] 获取店铺列表成功");
        }
        Err(e) => {
            log::error!("[shop_name_callback] 获取店铺列表失败: {}", e);
        }
    };
}

// 必须用 async：同步 command 跑在 blocking 线程池，
// 在那里调 add_child 会触发 wry 的线程检查导致 crash。
// 官方对动态多 webview 的支持要求 async command。
#[tauri::command]
pub async fn add_shop(window: Window, platform: ShopPlatform) {
    log::info!("添加店铺: {}", platform.platform);

    match platform.platform.as_str() {
        s if s == PLATFORM_DOUYIN => {
            log::info!("选择抖音小店");
            match crate::webview::douyin_shop::open_douyin_shop(&window) {
                Ok(_) => {
                    if let Some(wv) = window.get_webview("02_app") {
                        notify_shop_list_update(&wv);
                    }
                }
                Err(e) => log::error!("[add_shop] {e}"),
            }
        }
        s if s == PLATFORM_PINDUODUO => {
            log::info!("选择拼多多");
            match crate::webview::douyin_shop::open_pinduoduo_shop(&window) {
                Ok(_) => {
                    if let Some(wv) = window.get_webview("02_app") {
                        notify_shop_list_update(&wv);
                    }
                }
                Err(e) => log::error!("[add_shop] {e}"),
            }
        }
        _ => {
            log::warn!("未知平台");
        }
    }
}
