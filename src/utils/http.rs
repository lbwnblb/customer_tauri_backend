use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct HttpError {
    pub kind: HttpErrorKind,
    pub message: String,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)
    }
}

impl std::error::Error for HttpError {}

impl From<reqwest::Error> for HttpError {
    fn from(e: reqwest::Error) -> Self {
        let kind = if e.is_timeout() {
            HttpErrorKind::Timeout
        } else if e.is_connect() {
            HttpErrorKind::Connection
        } else if e.is_decode() {
            HttpErrorKind::Decode
        } else {
            HttpErrorKind::Request
        };
        HttpError {
            kind,
            message: e.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpErrorKind {
    Timeout,
    Connection,
    Decode,
    Request,
    Status(StatusCode),
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_json::from_str(&self.body).map_err(|e| HttpError {
            kind: HttpErrorKind::Decode,
            message: format!("JSON 解析失败: {}", e),
        })
    }
}

pub struct HttpClient {
    client: Client,
    default_headers: HeaderMap,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(DEFAULT_TIMEOUT)
                .build()
                .unwrap_or_default(),
            default_headers: HeaderMap::new(),
        }
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_default();
        Self {
            client,
            default_headers: HeaderMap::new(),
        }
    }

    pub fn set_default_header(&mut self, key: &str, value: &str) {
        if let (Ok(k), Ok(v)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            self.default_headers.insert(k, v);
        }
    }

    pub fn set_default_headers(&mut self, headers: &[(&str, &str)]) {
        for (key, value) in headers {
            self.set_default_header(key, value);
        }
    }

    pub async fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
        self.request(Method::GET, url, None::<&()>, None).await
    }

    pub async fn post<T: Serialize + ?Sized>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<HttpResponse, HttpError> {
        self.request(Method::POST, url, Some(body), None).await
    }

    pub async fn put<T: Serialize + ?Sized>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<HttpResponse, HttpError> {
        self.request(Method::PUT, url, Some(body), None).await
    }

    pub async fn delete(&self, url: &str) -> Result<HttpResponse, HttpError> {
        self.request(Method::DELETE, url, None::<&()>, None).await
    }

    pub async fn request<T: Serialize + ?Sized>(
        &self,
        method: Method,
        url: &str,
        body: Option<&T>,
        extra_headers: Option<&[(&str, &str)]>,
    ) -> Result<HttpResponse, HttpError> {
        let mut req = self
            .client
            .request(method, url)
            .headers(self.default_headers.clone());

        if let Some(headers) = extra_headers {
            for (key, value) in headers {
                if let (Ok(k), Ok(v)) = (
                    HeaderName::from_bytes(key.as_bytes()),
                    HeaderValue::from_str(value),
                ) {
                    req = req.header(k, v);
                }
            }
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let response = req.send().await?;
        let status = response.status();
        let mut headers_map = HashMap::new();
        for (k, v) in response.headers() {
            if let Ok(s) = v.to_str() {
                headers_map.insert(k.as_str().to_string(), s.to_string());
            }
        }

        let body = response.text().await?;

        if status.is_client_error() || status.is_server_error() {
            return Err(HttpError {
                kind: HttpErrorKind::Status(status),
                message: format!("HTTP {}: {}", status.as_u16(), body),
            });
        }

        Ok(HttpResponse {
            status,
            body,
            headers: headers_map,
        })
    }
}

pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
    HttpClient::default().get(url).await
}

pub async fn get_with_timeout(url: &str, timeout_secs: u64) -> Result<HttpResponse, HttpError> {
    HttpClient::with_timeout(timeout_secs).get(url).await
}

pub async fn post_json<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
) -> Result<HttpResponse, HttpError> {
    HttpClient::default().post(url, body).await
}

pub async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, HttpError> {
    let resp = get(url).await?;
    resp.json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_post_json_server_error() -> Result<(), Box<dyn std::error::Error>> {
        let mut client = HttpClient::new();
        client.set_default_headers(&[
            ("accept", "application/json, text/plain, */*"),
            ("accept-language", "zh-CN,zh;q=0.9,en;q=0.8"),
            ("priority", "u=1, i"),
            ("referer", "https://fxg.jinritemai.com/ffa/mshop/homepage/index"),
            ("sec-ch-ua", "\"Google Chrome\";v=\"147\", \"Not.A/Brand\";v=\"8\", \"Chromium\";v=\"147\""),
            ("sec-ch-ua-mobile", "?0"),
            ("sec-ch-ua-platform", "\"Windows\""),
            ("sec-fetch-dest", "empty"),
            ("sec-fetch-mode", "cors"),
            ("sec-fetch-site", "same-origin"),
            ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36"),
            ("cookie", "s_v_web_id=verify_mp527ktm_mJeZdDCs_MtIX_4vRz_B9rL_JADHn91hXVlo; passport_csrf_token=8e3d9de665b49ca6f0c4f8d4c453d2cb; passport_csrf_token_default=8e3d9de665b49ca6f0c4f8d4c453d2cb; x-web-secsdk-uid=963b0c06-89c1-4442-82cb-40bfcaad3612; Hm_lvt_b6520b076191ab4b36812da4c90f7a5e=1778737215; HMACCOUNT=79B115A86823BFF7; passport_mfa_token=CjHOL6AFjWMcPTh41Q%2Bqyewe5uyoYUySnfbAcUo5m70rTUrpHCUBqAs8fWz22N6%2FGG%2BnGkoKPAAAAAAAAAAAAABQbDRVU9Vlv7%2Ftc9908KQrhRhe0UsbHdoQB2ac1c%2Bp3wDS9G4QK73%2Basz1EWPxmZxPoRCmwZEOGPax0WwgAiIBA5GhIWs%3D; uid_tt=5e537c13d25b05b5868d19a6edb8660b; uid_tt_ss=5e537c13d25b05b5868d19a6edb8660b; sid_tt=55343f4379f3ad5e66ac2ad532be4f77; sessionid=55343f4379f3ad5e66ac2ad532be4f77; sessionid_ss=55343f4379f3ad5e66ac2ad532be4f77; is_staff_user=false; has_biz_token=false; ucas_c0=CkEKBTEuMC4wEJuIhOzd-bSDahjmJiCvnbD5uYzFAiiwITDA9bDC743JAkDQp5vQBkjQ29fSBlCJvNuQy4i4rWhYbxIUP4IXJVZ7f19Mz3Z1JR8COryspFQ; ucas_c0_ss=CkEKBTEuMC4wEJuIhOzd-bSDahjmJiCvnbD5uYzFAiiwITDA9bDC743JAkDQp5vQBkjQ29fSBlCJvNuQy4i4rWhYbxIUP4IXJVZ7f19Mz3Z1JR8COryspFQ; PHPSESSID=c54448ebaa5311327493fee24518d98e; PHPSESSID_SS=c54448ebaa5311327493fee24518d98e; ecom_us_lt=8227995d973ab8d9eafff1e175938b230f0455342cd9c56b7d7f1955e83c2558; ecom_us_lt_ss=8227995d973ab8d9eafff1e175938b230f0455342cd9c56b7d7f1955e83c2558; zsgw_business_data=%7B%22uuid%22%3A%229ae8e350-9e15-4b63-a740-98f72d47d64b%22%2C%22platform%22%3A%22pc%22%2C%22source%22%3A%22seo.fxg.jinritemai.com%22%7D; source=seo.fxg.jinritemai.com; doudain_safety_did=3493615810962619; csrf_session_id=988c027c359ce20caf8db1a7cc7e2673; SHOP_ID=510024; PIGEON_CID=7519569113498705417; ffa_goods_ewid=3493615810962619; ffa_goods_seraph_did=3493615810962619; __security_mc_1_s_sdk_crypt_sdk=7ed749af-4401-ae7e; bd_ticket_guard_client_web_domain=2; sid_guard=55343f4379f3ad5e66ac2ad532be4f77%7C1778833114%7C5183223%7CTue%2C+14-Jul-2026+08%3A05%3A37+GMT; session_tlb_tag=sttt%7C14%7CVTQ_Q3nzrV5mrCrVMr5Pd_________-lb_ysSGPaBOlFWjSC-GMR9bURKKanpA65gCor4EpuvDM%3D; sid_ucp_v1=1.0.0-KDVmMDExYjBlMDlkYzdhMTgwOGUxNDEwNDM0MDAyOTAyZDg4MjAyNjEKGQjA9bDC743JAhDarZvQBhiwISAMOAFA6wcaAmxmIiA1NTM0M2Y0Mzc5ZjNhZDVlNjZhYzJhZDUzMmJlNGY3Nw; ssid_ucp_v1=1.0.0-KDVmMDExYjBlMDlkYzdhMTgwOGUxNDEwNDM0MDAyOTAyZDg4MjAyNjEKGQjA9bDC743JAhDarZvQBhiwISAMOAFA6wcaAmxmIiA1NTM0M2Y0Mzc5ZjNhZDVlNjZhYzJhZDUzMmJlNGY3Nw; biz_trace_id=a19eea69; bd_ticket_guard_client_data=eyJiZC10aWNrZXQtZ3VhcmQtdmVyc2lvbiI6MiwiYmQtdGlja2V0LWd1YXJkLWl0ZXJhdGlvbi12ZXJzaW9uIjoxLCJiZC10aWNrZXQtZ3VhcmQtcmVlLXB1YmxpYy1rZXkiOiJCRFhTUjd2dkRkNCtOVDhjQmVJaU53aWNoVGk2MUIwVUhiaStpUDkyTEhkTjNtcVhKVXZOMUo2TzRlWDVhTXJja3lUKzRwaDJzbFRpZHFST09DMEFTZ289IiwiYmQtdGlja2V0LWd1YXJkLXdlYi12ZXJzaW9uIjoyfQ%3D%3D; odin_tt=bf7a0b911b4cd1bd653ffaabc2dffa8d04a75b2faf56dee49e6bb3f6941885e4fbc8f88b0dbc3e7ca2869d8ed417e7b1; Hm_lpvt_b6520b076191ab4b36812da4c90f7a5e=1779166220; ttwid=1%7Cc3kWTnpqToqKzUJ-2E0d7zFuIinZw_7c3BxlUsvzIGw%7C1779166221%7C86c7d964eae06238076b50399e12e376b8a4a1a8db7462a7f055f31dfd169da6; ecom_gray_shop_id=510024; gfkadpd=4272,23756"),
        ]);

        let res = client.get("https://fxg.jinritemai.com/center/qualification/shop/info?version=0&appid=1&_bid=fxg_admin").await?;
        println!("{}", res.body);

        Ok(())
    }
}
