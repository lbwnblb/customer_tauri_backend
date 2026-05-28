use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use serde::Serialize;
use tauri::{Manager, Webview, Window};
use tokio::sync::oneshot;

use crate::utils::pinduoduo::pinduoduo_resp::send_message;

struct PddCryptoParams {
    anti_content: String,
    anti_content_outer: String,
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
    anti_content_outer: String,
    hash: String,
    random: String,
    request_id: i64,
) {
    let sender = PDD_CRYPTO_CALLBACKS.lock().unwrap().remove(&callback_id);
    if let Some(tx) = sender {
        let _ = tx.send(PddCryptoParams { anti_content, anti_content_outer, hash, random, request_id });
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PddSendResult {
    pub success: bool,
    pub error_msg: Option<String>,
}

/// 通过 webview JS 计算 anti_content / anti_content_outer（浏览器指纹，与 uid/content 无关）。
/// 返回 (anti_content_inner, anti_content_outer)，失败时返回 Err。
pub async fn fetch_pdd_crypto(webview: &Webview, uid: &str, content: &str) -> Result<(String, String), String> {
    let callback_id = crate::utils::uuid_no_hyphen();
    let (tx, rx) = oneshot::channel::<PddCryptoParams>();

    PDD_CRYPTO_CALLBACKS
        .lock()
        .unwrap()
        .insert(callback_id.clone(), tx);

    let js = format!(
        r#"(function(){{var u={uid},c={content},id={cb},n=0;function t(){{if(window.__pddComputeCrypto){{window.__pddComputeCrypto(u,c,id);}}else if(++n<45){{setTimeout(t,200);}}else{{console.error('[pdd] crypto hook 未就绪，已等待 9s');}}}}t();}})();"#,
        uid     = serde_json::to_string(uid).unwrap(),
        content = serde_json::to_string(content).unwrap(),
        cb      = serde_json::to_string(&callback_id).unwrap(),
    );
    webview.eval(&js).map_err(|e| e.to_string())?;

    let params = tokio::time::timeout(std::time::Duration::from_secs(10), rx)
        .await
        .map_err(|_| "等待加密参数超时".to_string())?
        .map_err(|_| "crypto channel 已关闭".to_string())?;

    Ok((params.anti_content, params.anti_content_outer))
}

/// 内部调用：通过 webview 内 JS 计算加密参数后发送消息（供自动回复等场景使用）。
pub async fn send_pdd_auto_reply(webview: &Webview, uid: &str, content: &str) -> Result<(), String> {
    let callback_id = crate::utils::uuid_no_hyphen();
    let (tx, rx) = oneshot::channel::<PddCryptoParams>();

    PDD_CRYPTO_CALLBACKS
        .lock()
        .unwrap()
        .insert(callback_id.clone(), tx);

    let js = format!(
        r#"(function(){{var u={uid},c={content},id={cb},n=0;function t(){{if(window.__pddComputeCrypto){{window.__pddComputeCrypto(u,c,id);}}else if(++n<45){{setTimeout(t,200);}}else{{console.error('[pdd] crypto hook 未就绪，已等待 9s');}}}}t();}})();"#,
        uid     = serde_json::to_string(uid).unwrap(),
        content = serde_json::to_string(content).unwrap(),
        cb      = serde_json::to_string(&callback_id).unwrap(),
    );
    webview.eval(&js).map_err(|e| e.to_string())?;

    let params = tokio::time::timeout(std::time::Duration::from_secs(10), rx)
        .await
        .map_err(|_| "等待加密参数超时".to_string())?
        .map_err(|_| "crypto channel 已关闭".to_string())?;

    let result = send_message(
        webview,
        uid,
        content,
        &params.anti_content,
        &params.anti_content_outer,
        &params.hash,
        &params.random,
        params.request_id,
    )
    .await
    .map(|_| ())
    .map_err(|e| e.to_string());

    if result.is_ok() {
        inject_sent_message(webview, uid, content);
    }

    result
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
        r#"(function(){{var u={uid},c={content},id={cb},n=0;function t(){{if(window.__pddComputeCrypto){{window.__pddComputeCrypto(u,c,id);}}else if(++n<45){{setTimeout(t,200);}}else{{console.error('[pdd] crypto hook 未就绪，已等待 9s');}}}}t();}})();"#,
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
        &params.anti_content_outer,
        &params.hash,
        &params.random,
        params.request_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    if resp.success {
        inject_sent_message(&webview, &uid, &content);
    }

    Ok(PddSendResult {
        success: resp.success,
        error_msg: if resp.success { None } else { resp.error_msg },
    })
}

/// 发送成功后直接把消息注入 PDD 的 Vuex store，使 UI 立即回显。
///
/// PDD 正常发送走 WebSocket (PcSocketUtil.sendMsg) + 乐观更新，
/// 我们绕过走 REST API，服务端不会把"已发"消息推回发送方，
/// 所以必须手动把消息 commit 进 store。
fn inject_sent_message(webview: &Webview, uid: &str, content: &str) {
    let js = format!(
        r#"(function(){{
    var uid = {uid}, content = {content};
    // 非 PC 模式渲染用 _textProcessContent（saveMessage 生成），需要手动计算
    var escaped = content
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/\n/g, '<br>');
    var newMsg = {{
        from: {{role: "mall_cs"}},
        to:   {{role: "user", uid: String(uid)}},
        ts:   String(Math.floor(Date.now() / 1000)),
        content: content,
        rawContent: content,
        _textProcessContent: escaped,
        type: 0,
        is_aut: 0,
        is_read: 1,
        status: "read",
        msg_id: String(Date.now())
    }};

    // 从 DOM 找 Vue 根实例
    var vm = null;
    var candidates = [
        document.getElementById('app'),
        document.querySelector('.app-wrapper'),
        document.body && document.body.firstElementChild
    ];
    for (var ci = 0; ci < candidates.length && !vm; ci++) {{
        var el = candidates[ci];
        while (el && !el.__vue__) el = el.firstElementChild;
        if (el && el.__vue__ && el.__vue__.$store) vm = el.__vue__;
    }}
    if (!vm) {{ console.warn('[pdd-inject] no vue store'); return; }}

    var store = vm.$store;
    var chatList = (store.state && store.state.chatList) || [];
    var chat = null;
    for (var i = 0; i < chatList.length; i++) {{
        if (chatList[i].userInfo && String(chatList[i].userInfo.uid) === String(uid)) {{
            chat = chatList[i]; break;
        }}
    }}
    if (!chat) {{ console.warn('[pdd-inject] chat not found for uid:', uid); return; }}

    var msgList = (chat.messageList || []).slice();
    msgList.push(newMsg);
    store.commit("UPDATE_CHAT_LIST_ONE", Object.assign({{}}, chat, {{
        messageList: msgList,
        lastMessage: newMsg,
        unRead: 0,
        unReplyTs: 0
    }}));

    // 滚动到底部
    vm.$bus && vm.$bus.$emit && vm.$bus.$emit('chatSendMessage', {{}});
    console.log('[pdd-inject] ok uid:', uid);
}})();"#,
        uid     = serde_json::to_string(uid).unwrap(),
        content = serde_json::to_string(content).unwrap(),
    );
    let _ = webview.eval(&js);
}
