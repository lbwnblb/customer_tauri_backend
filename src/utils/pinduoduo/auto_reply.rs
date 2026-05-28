use log::{info, warn};
use tauri::Webview;
use crate::commands::pdd_send::send_pdd_auto_reply;
use crate::utils::message::{Message, MSG_TYPE_PRODUCT};
use crate::utils::{timestamp_millis, PLATFORM_PINDUODUO};
use crate::utils::douyin::auto_reply::REPLY_DEBOUNCE_MAP;
use crate::utils::pinduoduo::message_converter::convert_messages;
use crate::utils::pinduoduo::pinduoduo_resp::{get_chat_list, get_goods_detail};

pub async fn handle_buyer_message(webview: Webview, buyer_uid: String) {

    let request_id = timestamp_millis();

    REPLY_DEBOUNCE_MAP.lock().unwrap().insert(buyer_uid.clone(),request_id);

    let (anti_inner, anti_outer) = match crate::commands::pdd_send::fetch_pdd_crypto(&webview, &buyer_uid, "").await {
        Ok(v) => v,
        Err(e) => {
            warn!("[PDD][History] uid={} 获取加密参数失败: {}", buyer_uid, e);
            (String::new(), String::new())
        }
    };

    match get_chat_list(&webview, &buyer_uid, None, 0, 11, &anti_inner, &anti_outer, request_id).await {
        Ok(resp) => {
            if let Some(result) = resp.result {
                if let Some(raw_messages) = result.messages {
                    let shark_product_id = raw_messages.iter()
                        .filter(|m| m.template_name.as_deref() == Some("user_source"))
                        .find_map(|m| {
                            m.info.as_ref()
                                .and_then(|v| v.get("goods_info"))
                                .and_then(|v| v.get("goods_id"))
                                .and_then(|v| {
                                    v.as_i64().map(|id| id.to_string())
                                        .or_else(|| v.as_str().map(|s| s.to_string()))
                                })
                        });
                    let messages = convert_messages(raw_messages);
                    for msg in &messages {
                        info!("[PDD][History] [{}] type={}: {}", msg.role, msg.msg_type, msg.content);
                        if msg.msg_type == MSG_TYPE_PRODUCT && !msg.content.is_empty() {
                            if let Ok(goods_id) = msg.content.parse::<i64>() {
                                sync_goods_if_missing(&webview, goods_id).await;
                            }
                        }
                    }
                    handle_auto_reply(&webview, &buyer_uid, &messages, shark_product_id.as_deref(),request_id).await;
                }
            } else {
                warn!("[PDD][History] uid={} 获取失败: {:?}", buyer_uid, resp.error_msg);
            }
        }
        Err(e) => warn!("[PDD][History] uid={} 请求错误: {}", buyer_uid, e),
    }
}

async fn handle_auto_reply(webview: &Webview, buyer_uid: &str, messages: &[Message], shark_product_id: Option<&str>,request_id: i64) {
    if !crate::database::ai_reply::get_ai_reply_enabled(webview.label()).unwrap_or(true) {
        info!("[PDD][AutoReply] AI回复已关闭，跳过自动回复 webview={}", webview.label());
        return;
    }
    let buyer_uid_clone = buyer_uid.to_string();
    let token = crate::utils::backend::get_app_token(webview).await;
    if token.is_empty() {
        warn!("[PDD][AutoReply] token 为空，跳过自动回复 uid={}", buyer_uid);
        return;
    }
    let shop_id = crate::database::webview_shop_id::get_platform_shop_id(webview.label())
        .unwrap_or(None);
    let reply = match crate::utils::backend::send_chat_message(&token, messages, shark_product_id, Some(buyer_uid), Some(PLATFORM_PINDUODUO), shop_id.as_deref()).await {
        Ok(r) => {
            info!("[PDD][AutoReply] 后端响应: {}", serde_json::to_string(&r).unwrap_or_default());
            r
        }
        Err(e) => {
            warn!("[PDD][AutoReply] 后端返回错误: {}", e);
            return;
        }
    };
    let reply = if reply.msg_type == 1 {
        if let Some(pid) = reply.product_id {
            match get_goods_detail(webview, pid).await {
                Ok(resp) if resp.success => {
                    if let Some(result) = resp.result {
                        match crate::utils::backend::pdd_product::sync_pdd_product(&result, &token).await {
                            Ok(id) => info!("[PDD][AutoReply] 已同步商品 goods_id={}", id),
                            Err(e) => warn!("[PDD][AutoReply] sync_pdd_product 失败: {}", e),
                        }
                    }
                }
                Ok(resp) => warn!("[PDD][AutoReply] get_goods_detail 失败: {:?}", resp.error_msg),
                Err(e) => warn!("[PDD][AutoReply] get_goods_detail 请求错误: {}", e),
            }
            match crate::utils::backend::send_chat_message(&token, messages, shark_product_id, Some(buyer_uid), Some(PLATFORM_PINDUODUO), shop_id.as_deref()).await {
                Ok(r) => {
                    info!("[PDD][AutoReply] 后端响应(重试): {}", serde_json::to_string(&r).unwrap_or_default());
                    r
                }
                Err(e) => {
                    warn!("[PDD][AutoReply] 后端返回错误(重试): {}", e);
                    return;
                }
            }
        } else {
            warn!("[PDD][AutoReply] msg_type=1 但 product_id 为空，跳过商品同步");
            reply
        }
    } else {
        reply
    };
    if reply.content.is_empty() {
        warn!("[PDD][AutoReply] 回复内容为空，跳过发送 uid={}", buyer_uid);
        return;
    }
    if REPLY_DEBOUNCE_MAP.lock().unwrap().get(buyer_uid_clone.as_str()).copied() != Some(request_id) {
        info!("[IM] [DEBOUNCE] sec_sender={} 有新消息到达，跳过本次自动回复", buyer_uid_clone);
        return;
    }
    let _ = send_pdd_auto_reply(webview, buyer_uid, &reply.content).await;
    info!("[PDD][AutoReply] 已自动回复 uid={}", buyer_uid);
}

async fn sync_goods_if_missing(webview: &Webview, goods_id: i64) {
    let token = crate::utils::backend::get_app_token(webview).await;
    if token.is_empty() {
        warn!("[PDD][Goods] token 为空，跳过商品同步 goods_id={}", goods_id);
        return;
    }
    match crate::utils::backend::pdd_product::pdd_product_exists(goods_id, &token).await {
        Ok(true) => {
            info!("[PDD][Goods] goods_id={} 已存在，跳过同步", goods_id);
        }
        Ok(false) => {
            match get_goods_detail(webview, goods_id).await {
                Ok(resp) if resp.success => {
                    if let Some(result) = resp.result {
                        match crate::utils::backend::pdd_product::sync_pdd_product(&result, &token).await {
                            Ok(id) => info!("[PDD][Goods] 已同步 goods_id={}", id),
                            Err(e) => warn!("[PDD][Goods] sync_pdd_product 失败: {}", e),
                        }
                    }
                }
                Ok(resp) => warn!("[PDD][Goods] get_goods_detail 失败: {:?}", resp.error_msg),
                Err(e) => warn!("[PDD][Goods] get_goods_detail 请求错误: {}", e),
            }
        }
        Err(e) => warn!("[PDD][Goods] pdd_product_exists 查询失败: {}", e),
    }
}
