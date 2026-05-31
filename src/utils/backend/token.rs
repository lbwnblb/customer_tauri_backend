use reqwest::Client;
use serde::Deserialize;

use super::{url, BackendResp};

#[derive(Debug, Deserialize)]
pub struct TokenVerifyData {
    pub valid: bool,
    pub user_id: serde_json::Value,
    pub expire_at: serde_json::Value,
}

#[derive(Debug)]
pub enum VerifyTokenError {
    Expired,
    BadToken,
    Http(String),
}

impl std::fmt::Display for VerifyTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyTokenError::Expired => write!(f, "令牌已过期或不存在"),
            VerifyTokenError::BadToken => write!(f, "缺少或格式错误的令牌"),
            VerifyTokenError::Http(e) => write!(f, "请求失败: {e}"),
        }
    }
}

impl std::error::Error for VerifyTokenError {}

pub async fn verify_token(token: &str) -> Result<TokenVerifyData, VerifyTokenError> {
    let client = Client::builder()
        .http1_only()
        .build()
        .map_err(|e| VerifyTokenError::Http(e.to_string()))?;
    let resp = client
        .get(url("/api/v1/auth/token/verify"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| {
            let mut msg = e.to_string();
            let mut src = std::error::Error::source(&e);
            while let Some(s) = src {
                msg.push_str(&format!(": {s}"));
                src = s.source();
            }
            VerifyTokenError::Http(msg)
        })?;

    let status = resp.status().as_u16();
    let body: BackendResp = resp
        .json()
        .await
        .map_err(|e| VerifyTokenError::Http(format!("响应解析失败: {e}")))?;

    match status {
        200 => {
            let raw = body.data.ok_or_else(|| VerifyTokenError::Http("响应缺少 data 字段".into()))?;
            serde_json::from_value(raw)
                .map_err(|e| VerifyTokenError::Http(format!("data 解析失败: {e}")))
        }
        401 => {
            if body.message.contains("过期") || body.message.contains("不存在") {
                Err(VerifyTokenError::Expired)
            } else {
                Err(VerifyTokenError::BadToken)
            }
        }
        other => Err(VerifyTokenError::Http(format!(
            "未预期的状态码 {other}: {}",
            body.message
        ))),
    }
}
