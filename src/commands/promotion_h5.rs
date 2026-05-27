use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use tauri::webview::WebviewBuilder;
use tauri::{command, Manager, Url, WebviewUrl, Webview, Window};
use tokio::sync::oneshot;

use crate::scripts::douyin::promotion_h5_interceptor::create_promotion_h5_interceptor;
use crate::utils::{app_data_dir, uuid_no_hyphen};

static PROMO_H5_PENDING: LazyLock<Mutex<HashMap<String, oneshot::Sender<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[command]
pub async fn on_promotion_pack_h5(webview: Webview, webview_label: String, body: String) {
    let sender = PROMO_H5_PENDING.lock().unwrap().remove(&webview_label);
    if let Some(tx) = sender {
        let _ = tx.send(body);
    }
    // 主动关闭隐藏 webview（成功路径）
    let _ = webview.close();
}

/// 创建一个 1×1 隐藏 webview，加载商品详情页，
/// 等待页面内 JS 拦截到 aweme/v2/shop/promotion/pack/h5 响应后回调，
/// 返回原始响应体文本。无论成功还是超时都保证 webview 已关闭。
/// shop_webview_id：调用方店铺 webview 的 label，用于共享同一 data_directory（cookies），
/// 避免因新建匿名会话触发风控。
pub async fn get_promotion_pack_h5_via_webview(
    window: &Window,
    shop_webview_id: &str,
    product_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (tx, rx) = oneshot::channel::<String>();
    let hidden_id = format!("promo_h5_{}", uuid_no_hyphen());

    PROMO_H5_PENDING.lock().unwrap().insert(hidden_id.clone(), tx);

    let data_dir = PathBuf::from(app_data_dir()).join(shop_webview_id);
    let page_url = format!(
        "https://haohuo.jinritemai.com/ecommerce/trade/detail/index.html?id={}&origin_type=doudian_product_management",
        product_id
    );

    if let Err(e) = window.add_child(
        WebviewBuilder::new(
            &hidden_id,
            WebviewUrl::External(Url::parse(&page_url)?),
        )
        .initialization_script(&create_promotion_h5_interceptor(&hidden_id))
        .data_directory(data_dir),
        tauri::LogicalPosition::new(0.0, 0.0),
        tauri::LogicalSize::new(1.0, 1.0),
    ) {
        // add_child 失败时清理 pending，避免内存泄漏
        PROMO_H5_PENDING.lock().unwrap().remove(&hidden_id);
        return Err(e.into());
    }

    log::info!("[PromoH5] 创建隐藏 webview id={} product_id={}", hidden_id, product_id);

    let result = tokio::time::timeout(Duration::from_secs(30), rx).await;

    // 无论成功/超时，都确保 webview 被关闭（on_promotion_pack_h5 已关闭则 get_webview 返回 None）
    if let Some(wv) = window.get_webview(&hidden_id) {
        let _ = wv.close();
    }

    match result {
        Ok(Ok(body)) => {
            log::info!("[PromoH5] 收到响应 id={} len={}", hidden_id, body.len());
            Ok(body)
        }
        Ok(Err(_)) => Err("promotion_h5 oneshot channel 已关闭".into()),
        Err(_) => {
            PROMO_H5_PENDING.lock().unwrap().remove(&hidden_id);
            log::warn!("[PromoH5] 等待响应超时 id={}", hidden_id);
            Err("等待 promotion_h5 响应超时（30s）".into())
        }
    }
}
