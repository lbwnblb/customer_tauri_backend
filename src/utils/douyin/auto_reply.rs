use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use log::{info, warn};
use prost::Message;
use tauri::{Manager, Webview};
use tokio::time::sleep;
use crate::utils::get_by_conversation;
use crate::utils::douyin::doudian_utils::{build_send_frame, build_send_payload};
use crate::utils::douyin::message_converter::convert_messages;
use crate::utils::douyin::protobuf::im_proto::MessageBody;

static PROCESSED_CLIENT_MSG_IDS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static REPLY_DEBOUNCE_MAP: LazyLock<Mutex<HashMap<String, i64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static SHARK_PRODUCT_ID_MAP: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn schedule_auto_reply(webview: Webview, message: &MessageBody) {
    let sec_sender = match message.sec_sender.clone() {
        Some(id) => id,
        None => {
            warn!("[IM] [DEBOUNCE] sec_sender 为空，跳过 message={:?}", message);
            return;
        }
    };
    let short_id = match message.conversation_short_id {
        Some(id) => id,
        None => {
            warn!("[IM] [DEBOUNCE] conversation_short_id 为空，跳过 message={:?}", message);
            return;
        }
    };
    let security_id = match message.security_conversation_id.clone() {
        Some(id) => id,
        None => {
            warn!("[IM] [DEBOUNCE] security_conversation_id 为空，跳过 message={:?}", message);
            return;
        }
    };

    let sub_id = message.sub_conversation_short_id;

    handle_auto_reply(&webview, short_id, &security_id, sub_id, sec_sender).await;
}

async fn handle_auto_reply(
    webview: &Webview,
    conversation_short_id: i64,
    security_conversation_id: &str,
    sub_conversation_short_id: Option<i64>,
    sec_sender: String,
) {
    let ts = crate::utils::timestamp_millis();
    REPLY_DEBOUNCE_MAP.lock().unwrap().insert(sec_sender.clone(), ts);

    if REPLY_DEBOUNCE_MAP.lock().unwrap().get(&sec_sender).copied() != Some(ts) {
        info!("[IM] [DEBOUNCE] sec_sender={} 有新消息到达，跳过本次自动回复", sec_sender);
        return;
    }
    let token = crate::utils::backend::get_app_token(webview).await;
    let conversation = match get_by_conversation(webview, security_conversation_id, conversation_short_id).await.map_err(|e| e.to_string()) {
        Ok(c) => c,
        Err(e) => {
            warn!("[IM] [RECV] 获取会话失败: {:?}", e);
            return;
        }
    };
    let mut read_message_index = conversation.last().and_then(|m| m.index_in_conversation).unwrap_or(0);
    let mut read_message_index_v2 = conversation.last().and_then(|m| m.index_in_conversation_v2).unwrap_or(0);
    let messages = convert_messages(conversation);
    if token.is_empty() {
        warn!("[IM] [RECV] 后端 token 未设置，跳过自动回复");
        return;
    }
    let shark_pid = SHARK_PRODUCT_ID_MAP.lock().unwrap().get(&sec_sender).cloned();
    let reply = match crate::utils::backend::send_chat_message(&token, &messages, shark_pid.as_deref(), Some(&sec_sender), crate::utils::platform_id_from_webview_id(webview.label())).await {
        Ok(r) => {
            info!("[IM] [RECV] 后端响应: {}", serde_json::to_string(&r).unwrap_or_default());
            r
        }
        Err(e) => {
            warn!("[IM] [RECV] 后端返回错误: {}", e);
            return;
        }
    };
    let reply = if reply.msg_type == 1 {
        if let Some(pid) = reply.product_id {
            let pid_str = pid.to_string();
            let webview_id = webview.label().to_string();
            if let Err(e) = crate::utils::backend::product::sync_diagnose_product(&webview_id, webview, &pid_str, &token).await {
                warn!("[IM] [RECV] sync_diagnose_product 失败: {}", e);
            }
            if let Some(window) = webview.get_window("main") {
                if let Err(e) = crate::utils::backend::product::sync_promotion_h5(&window, &webview_id, &pid_str, &token).await {
                    warn!("[IM] [RECV] sync_promotion_h5 失败: {}", e);
                }
            } else {
                warn!("[IM] [RECV] 获取 main window 失败，跳过 sync_promotion_h5");
            }
            match crate::utils::backend::send_chat_message(&token, &messages, shark_pid.as_deref(), Some(&sec_sender), crate::utils::platform_id_from_webview_id(webview.label())).await {
                Ok(r) => {
                    info!("[IM] [RECV] 后端响应(重试): {}", serde_json::to_string(&r).unwrap_or_default());
                    r
                }
                Err(e) => {
                    warn!("[IM] [RECV] 后端返回错误(重试): {}", e);
                    return;
                }
            }
        } else {
            warn!("[IM] [RECV] msg_type=1 但 product_id 为空，跳过商品同步");
            reply
        }
    } else {
        reply
    };
    let send_payload = match build_send_payload(webview, Some(conversation_short_id), reply.content, sub_conversation_short_id, Some(security_conversation_id.to_string())).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("[IM] [RECV] build_send_payload 失败: {}", e);
            return;
        }
    };
    let send_payload_bytes = send_payload.encode_to_vec();
    let mut send_frame = build_send_frame(webview);
    send_frame.payload = Some(send_payload_bytes);
    let frame_bytes = send_frame.encode_to_vec();
    let js_bytes = format!("{:?}", frame_bytes);
    let js = format!(r#"(() => {{ const ws = window.__WS_INSTANCE__; if (ws && ws.readyState === 1) {{ ws.send(new Uint8Array({}).buffer); }} }})()"#, js_bytes);
    if REPLY_DEBOUNCE_MAP.lock().unwrap().get(&sec_sender).copied() != Some(ts) {
        info!("[IM] [DEBOUNCE] sec_sender={} 有新消息到达，跳过本次自动回复", sec_sender);
        return;
    }
    webview.eval(&js).ok();

    crate::utils::douyin::mark_read::send_mark_read(webview, crate::utils::douyin::mark_read::MarkReadParams {
        conversation_short_id,
        read_message_index,
        read_message_index_v2,
        read_badge_count: 0,
        conv_unread_count: 0,
        total_unread_count: 0,
        sub_conversation_short_id,
        security_conversation_id: Some(security_conversation_id.to_string()),
    });

    info!("[IM] [RECV] 已自动回复 cid={}", conversation_short_id);
}

pub fn record_shark_product_id(webview: &Webview, message: &MessageBody) {
    let sec_sender = match message.sec_sender.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return,
    };
    if let Some(pid) = message.ext.get("shark_product_id") {
        if !pid.is_empty() {
            info!("[IM] [ENTRY] sec_sender={} shark_product_id={}", sec_sender, pid);
            SHARK_PRODUCT_ID_MAP.lock().unwrap().insert(sec_sender, pid.clone());

            let pid = pid.clone();
            let webview = webview.clone();
            tokio::spawn(async move {
                sync_product_if_missing(webview, pid).await;
            });
        }
    }
}

async fn sync_product_if_missing(webview: Webview, product_id: String) {
    let token = crate::utils::backend::get_app_token(&webview).await;
    if token.is_empty() {
        warn!("[IM] [ENTRY] token 为空，跳过商品同步 product_id={}", product_id);
        return;
    }
    let webview_id = webview.label().to_string();

    match crate::utils::backend::product::product_exists(&product_id, &token).await {
        Ok(true) => {
            info!("[IM] [ENTRY] product_id={} 已存在，跳过 sync_diagnose_product", product_id);
        }
        Ok(false) => {
            if let Err(e) = crate::utils::backend::product::sync_diagnose_product(&webview_id, &webview, &product_id, &token).await {
                warn!("[IM] [ENTRY] sync_diagnose_product 失败: {}", e);
            }
        }
        Err(e) => {
            warn!("[IM] [ENTRY] product_exists 查询失败: {}", e);
        }
    }

    match crate::utils::backend::product::product_h5_exists(&product_id, &token).await {
        Ok(true) => {
            info!("[IM] [ENTRY] product_id={} h5 已存在，跳过 sync_promotion_h5", product_id);
        }
        Ok(false) => {
            if let Some(window) = webview.get_window("main") {
                if let Err(e) = crate::utils::backend::product::sync_promotion_h5(&window, &webview_id, &product_id, &token).await {
                    warn!("[IM] [ENTRY] sync_promotion_h5 失败: {}", e);
                }
            } else {
                warn!("[IM] [ENTRY] 获取 main window 失败，跳过 sync_promotion_h5");
            }
        }
        Err(e) => {
            warn!("[IM] [ENTRY] product_h5_exists 查询失败: {}", e);
        }
    }
}

pub fn on_notify(msg: &MessageBody) -> bool {
    if let Some(client_id) = msg.ext.get("s:client_message_id") {
        let mut seen = PROCESSED_CLIENT_MSG_IDS.lock().unwrap();
        if !seen.insert(client_id.clone()) {
            info!("[IM] [RECV] 重复消息已过滤 client_id={}", client_id);
            return false;
        }
        if seen.len() > 500 {
            seen.clear();
            seen.insert(client_id.clone());
        }
    }
    true
}
