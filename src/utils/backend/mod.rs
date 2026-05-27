pub mod token;
pub mod chat;
pub mod product;

use std::sync::{LazyLock, Mutex};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::{Manager, Webview};

use super::env::base_url;
use super::http::{HttpClient, HttpError, HttpResponse};

pub use token::{verify_token, TokenVerifyData, VerifyTokenError};
pub use chat::{send_chat_message, ChatError};

// ── 全局后端 token ─────────────────────────────────────────────────────────

static APP_TOKEN: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

pub fn set_app_token(token: &str) {
    *APP_TOKEN.lock().unwrap() = token.to_string();
}

pub async fn get_app_token(webview: &Webview) -> String {
    {
        let cached = APP_TOKEN.lock().unwrap();
        if !cached.is_empty() {
            return cached.clone();
        }
    }

    let Some(app_wv) = webview.get_webview("02_app") else {
        log::warn!("[get_app_token] 02_app webview 不存在，返回空 token");
        return String::new();
    };

    let token = match app_wv.cookies() {
        Ok(cookies) => cookies
            .iter()
            .find(|c| c.name() == "Authorization")
            .map(|c| c.value().to_string())
            .unwrap_or_default(),
        Err(e) => {
            log::warn!("[get_app_token] 获取 cookies 失败: {e}");
            return String::new();
        }
    };

    if !token.is_empty() {
        set_app_token(&token);
    }

    token
}

// ── 通用后端响应包装 ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(super) struct BackendResp {
    pub code: u16,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

// ── URL 工具 ───────────────────────────────────────────────────────────────

pub(super) fn url(path: &str) -> String {
    let base = base_url().trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

// ── 通用 HTTP 客户端 ───────────────────────────────────────────────────────

pub struct BackendClient {
    inner: HttpClient,
}

impl BackendClient {
    pub fn new() -> Self {
        Self {
            inner: HttpClient::default(),
        }
    }

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

pub fn client() -> BackendClient {
    BackendClient::new()
}
