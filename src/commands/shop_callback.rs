use tauri::ipc::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::{Manager, Webview};
use tauri::webview::Cookie;
use crate::database::update_shop_name;
use crate::utils::feige_resp::feige_shop_info;

pub const FEIGE_MANAGEMENT_HOST:&str = "fxg.jinritemai.com";

pub static FEIGE_MANAGEMENT_COOKIE: LazyLock<Mutex<HashMap<String, Vec<Cookie>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));



#[tauri::command]
pub async fn shop_name_callback(webview: Webview) {
    let label = webview.label().to_string();
    match webview.url() {
        Ok(url) => {
            match url.host_str() {
                None => {}
                Some(host) => {
                    if host.eq(FEIGE_MANAGEMENT_HOST) {
                        match webview.cookies() {
                            Ok(cookies) => {
                                FEIGE_MANAGEMENT_COOKIE.lock().unwrap().insert(label.clone(), cookies);
                            }
                            Err(e) => {
                                log::error!("[shop_name_callback] 获取cookie失败: {}", e);
                            }
                        }

                        
                        match feige_shop_info(&label,&webview).await {
                            Ok(info) => {
                                if let Some(data) = info.data {
                                    if let Some(shop_name) = data.shop_name {
                                        if let Err(e) = update_shop_name(&label, &shop_name) {
                                            log::error!("[shop_name_callback] 更新店铺名称失败: {}", e);
                                        } else {
                                            
                                            if url.path().contains("/ffa/mshop/homepage/index") {
                                                webview.eval(r#"
                                                window.location.href = 'https://im.jinritemai.com/pc_seller_v2/main/workspace';
                                                "#).unwrap();
                                            }
                                            
                                            match webview.get_webview("02_app") {
                                                None => {}
                                                Some(webview) => {
                                                    match webview.eval("fetchShopList()") {
                                                        Ok(()) => {
                                                            log::info!("[shop_name_callback] 获取店铺列表成功");
                                                        }
                                                        Err(e) => {
                                                            log::error!("[shop_name_callback] 获取店铺列表失败: {}", e);
                                                        }
                                                    };
                                                }
                                            };
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("[shop_name_callback] shop_info 获取失败: {}", e);
                            }
                        };
                    }
                }
            }
        }
        Err(_) => {}
    }

}


// fn chrono_now() -> String {
//     std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .map(|d| d.as_secs().to_string())
//         .unwrap_or_else(|_| "unknown".into())
// }
