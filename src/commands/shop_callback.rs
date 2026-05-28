use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tauri::{Manager, Webview};
use tauri::webview::Cookie;
use crate::commands::shop::notify_shop_list_update;
use crate::database::{update_shop_name, upsert_webview_shop_id};
use crate::utils::douyin::doudian_utils::get_current_shop_id;
use crate::utils::douyin::feige_resp::feige_shop_info;
use crate::utils::pinduoduo::pinduoduo_resp::query_final_credential_new;
use crate::utils::{is_douyin_platform, is_pinduoduo_platform};

pub static FEIGE_MANAGEMENT_COOKIE: LazyLock<Mutex<HashMap<String, Vec<Cookie>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

async fn handle_douyin_callback(webview: &Webview, label: &str) {
    let url = match webview.url() {
        Ok(url) => url,
        Err(_) => return,
    };

    if url.path().contains("/login/common") {
        let _ = update_shop_name(label, "未登录");
        if let Some(app_webview) = webview.get_webview("02_app") {
            notify_shop_list_update(&app_webview);
        }
        return;
    }

    match webview.cookies() {
        Ok(cookies) => {
            FEIGE_MANAGEMENT_COOKIE.lock().unwrap().insert(label.to_string(), cookies);
        }
        Err(_) => {}
    }

    match feige_shop_info(label, webview).await {
        Ok(info) => {
            if let Some(data) = info.data {
                if let Some(shop_name) = data.shop_name {
                    if let Err(_) = update_shop_name(label, &shop_name) {

                    } else {
                        if let Ok(shop_id) = get_current_shop_id(webview) {
                            let _ = upsert_webview_shop_id(label, &shop_id);
                        }
                        if url.path().contains("/ffa/mshop/homepage/index") {
                            webview.eval(r#"
                            window.location.href = 'https://im.jinritemai.com/pc_seller_v2/main/workspace';
                            "#).unwrap();
                        }

                        if let Some(app_webview) = webview.get_webview("02_app") {
                            notify_shop_list_update(&app_webview);
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
}

async fn handle_pinduoduo_callback(webview: &Webview, label: &str) {
    let url = match webview.url() {
        Ok(url) => url,
        Err(_) => return,
    };

    if url.path().contains("/login") {
        let _ = update_shop_name(label, "未登录");
        if let Some(app_webview) = webview.get_webview("02_app") {
            notify_shop_list_update(&app_webview);
        }
        return;
    }

    match query_final_credential_new(webview).await {
        Ok(resp) => {
            if let Some(result) = resp.result {
                if let Some(mall_info) = result.mall_info {
                    if let Some(mall_name) = mall_info.mall_name {
                        if let Err(_) = update_shop_name(label, &mall_name) {
                        } else {
                            let _ = upsert_webview_shop_id(label, &mall_info.id.to_string());
                            if url.path().contains("/home") {
                                webview.eval(r#"
                                window.location.href = 'https://mms.pinduoduo.com/chat-merchant/index.htm';
                                "#).unwrap();
                            }

                            if let Some(app_webview) = webview.get_webview("02_app") {
                                notify_shop_list_update(&app_webview);
                            }
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
}

#[tauri::command]
pub async fn shop_name_callback(webview: Webview) {
    let label = webview.label().to_string();
    if is_douyin_platform(&label) {
        handle_douyin_callback(&webview, &label).await;
    }
    if is_pinduoduo_platform(&label) {
        handle_pinduoduo_callback(&webview, &label).await;
    }
}


// fn chrono_now() -> String {
//     std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .map(|d| d.as_secs().to_string())
//         .unwrap_or_else(|_| "unknown".into())
// }
