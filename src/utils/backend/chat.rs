use reqwest::Client;
use serde::Serialize;

use crate::utils::message::{Message, Reply};
use super::{url, BackendResp};

#[derive(Serialize)]
struct ChatRequest<'a> {
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    shark_product_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sec_sender: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shop_id: Option<&'a str>,
}

#[derive(Debug)]
pub enum ChatError {
    BadRequest(String),
    Unauthorized(String),
    RateLimited(String),
    ServerError(String),
    Http(String),
}

impl std::fmt::Display for ChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatError::BadRequest(m) => write!(f, "请求错误: {m}"),
            ChatError::Unauthorized(m) => write!(f, "未授权: {m}"),
            ChatError::RateLimited(m) => write!(f, "请求过于频繁: {m}"),
            ChatError::ServerError(m) => write!(f, "服务器异常: {m}"),
            ChatError::Http(m) => write!(f, "请求失败: {m}"),
        }
    }
}

impl std::error::Error for ChatError {}

pub async fn send_chat_message(token: &str, messages: &[Message], shark_product_id: Option<&str>, sec_sender: Option<&str>, platform_id: Option<&str>, shop_id: Option<&str>) -> Result<Reply, ChatError> {
    let req = ChatRequest { messages, shark_product_id, sec_sender, platform_id, shop_id };
    log::info!("[chat] 请求参数: {}", serde_json::to_string(&req).unwrap_or_default());

    let resp = Client::new()
        .post(url("/api/v1/chat/message"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&req)
        .send()
        .await
        .map_err(|e| ChatError::Http(e.to_string()))?;

    let status = resp.status().as_u16();
    let body: BackendResp = resp
        .json()
        .await
        .map_err(|e| ChatError::Http(format!("响应解析失败: {e}")))?;

    log::info!("[chat] 响应 status={status} code={} message={} data={:?}", body.code, body.message, body.data);

    match status {
        200 => {
            let raw = body.data.ok_or_else(|| ChatError::Http("响应缺少 data 字段".into()))?;
            serde_json::from_value(raw).map_err(|e| ChatError::Http(format!("data 解析失败: {e}")))
        }
        400 => Err(ChatError::BadRequest(body.message)),
        401 => Err(ChatError::Unauthorized(body.message)),
        429 => Err(ChatError::RateLimited(body.message)),
        500 => Err(ChatError::ServerError(body.message)),
        other => Err(ChatError::Http(format!("未预期的状态码 {other}: {}", body.message))),
    }
}
