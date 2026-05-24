use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use serde::Serialize;
use tauri::{Manager, Window};
use tokio::sync::oneshot;

use crate::utils::pinduoduo::pinduoduo_resp::send_message;

struct PddCryptoParams {
    anti_content: String,
    hash: String,
    random: String,
    request_id: i64,
}

static PDD_CRYPTO_CALLBACKS: LazyLock<Mutex<HashMap<String, oneshot::Sender<PddCryptoParams>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// JS webview 计算完加密参数后回调 Rust
#[tauri::command]
pub async fn pdd_crypto_callback(
    callback_id: String,
    anti_content: String,
    hash: String,
    random: String,
    request_id: i64,
) {
    let sender = PDD_CRYPTO_CALLBACKS.lock().unwrap().remove(&callback_id);
    if let Some(tx) = sender {
        let _ = tx.send(PddCryptoParams { anti_content, hash, random, request_id });
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PddSendResult {
    pub success: bool,
    pub error_msg: Option<String>,
}

/// 前端调用此命令发送拼多多客服消息。
/// Rust 通过 webview.eval 触发 JS 计算 anti_content / hash / random，
/// JS 回调 pdd_crypto_callback，再由 Rust 发起 HTTP 请求。
#[tauri::command]
pub async fn pdd_send_message(
    window: Window,
    webview_label: String,
    uid: String,
    content: String,
) -> Result<PddSendResult, String> {
    let webview = window
        .get_webview(&webview_label)
        .ok_or_else(|| format!("webview {} 不存在", webview_label))?;

    let callback_id = crate::utils::uuid_no_hyphen();
    let (tx, rx) = oneshot::channel::<PddCryptoParams>();

    PDD_CRYPTO_CALLBACKS
        .lock()
        .unwrap()
        .insert(callback_id.clone(), tx);

    let js = format!(
        "if (window.__pddComputeCrypto) {{ window.__pddComputeCrypto({uid}, {content}, {cb}); }} else {{ console.error('[pdd] crypto hook 未就绪'); }}",
        uid     = serde_json::to_string(&uid).unwrap(),
        content = serde_json::to_string(&content).unwrap(),
        cb      = serde_json::to_string(&callback_id).unwrap(),
    );
    webview.eval(&js).map_err(|e| e.to_string())?;

    let params = tokio::time::timeout(std::time::Duration::from_secs(10), rx)
        .await
        .map_err(|_| "等待加密参数超时（10 s）".to_string())?
        .map_err(|_| "crypto 回调 channel 已关闭".to_string())?;

    log::info!(
        "[pdd_send] uid={} request_id={} hash_len={} anti_len={}",
        uid,
        params.request_id,
        params.hash.len(),
        params.anti_content.len()
    );

    let resp = send_message(
        &webview,
        &uid,
        &content,
        &params.anti_content,
        &params.hash,
        &params.random,
        params.request_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(PddSendResult {
        success: resp.success,
        error_msg: if resp.success { None } else { resp.error_msg },
    })
}
