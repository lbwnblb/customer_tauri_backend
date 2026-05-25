use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::env::base_url;
use super::http::{HttpClient, HttpError, HttpResponse};

// ── 通用后端响应包装 ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct BackendResp {
    pub code: u16,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

// ── token 验证 ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenVerifyData {
    pub valid: bool,
    pub user_id: serde_json::Value,
    pub expire_at: serde_json::Value,
}

#[derive(Debug)]
pub enum VerifyTokenError {
    /// 401 — 令牌已过期或不存在
    Expired,
    /// 401 — 缺少或格式错误的令牌
    BadToken,
    /// 网络 / 解析错误
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

/// 验证 token 有效性。
///
/// - 200 → `Ok(TokenVerifyData)`
/// - 401 令牌过期/不存在 → `Err(VerifyTokenError::Expired)`
/// - 401 格式错误/缺失   → `Err(VerifyTokenError::BadToken)`
pub async fn verify_token(token: &str) -> Result<TokenVerifyData, VerifyTokenError> {
    let resp = Client::new()
        .get(url("/api/v1/auth/token/verify"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| VerifyTokenError::Http(e.to_string()))?;

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

fn url(path: &str) -> String {
    let base = base_url().trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

pub struct BackendClient {
    inner: HttpClient,
}

impl BackendClient {
    pub fn new() -> Self {
        Self {
            inner: HttpClient::default(),
        }
    }

    /// 携带固定请求头（如 Authorization token）。
    pub fn with_token(mut self, token: &str) -> Self {
        self.inner.set_default_header("Authorization", &format!("Bearer {token}"));
        self
    }

    pub async fn get(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.inner.get(&url(path)).await
    }

    pub async fn get_typed<T: DeserializeOwned>(&self, path: &str) -> Result<T, HttpError> {
        self.inner.get(&url(path)).await?.json()
    }

    pub async fn post<T: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<HttpResponse, HttpError> {
        self.inner.post(&url(path), body).await
    }

    pub async fn post_typed<B: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R, HttpError> {
        self.inner.post(&url(path), body).await?.json()
    }

    pub async fn put<T: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<HttpResponse, HttpError> {
        self.inner.put(&url(path), body).await
    }

    pub async fn delete(&self, path: &str) -> Result<HttpResponse, HttpError> {
        self.inner.delete(&url(path)).await
    }
}

/// 全局共享实例，无需 token 时直接使用。
pub fn client() -> BackendClient {
    BackendClient::new()
}
