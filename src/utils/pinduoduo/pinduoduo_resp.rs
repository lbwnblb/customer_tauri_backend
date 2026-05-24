use crate::utils::douyin::feige_resp::REQUEST_HEADERS;
use crate::utils::http::{HttpClient, HttpError};
use reqwest::Method;
use serde::Deserialize;
use serde_json::json;
use tauri::Webview;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFinalCredentialResp {
    pub success: bool,
    pub error_code: i64,
    pub error_msg: Option<String>,
    pub result: Option<QueryFinalCredentialResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFinalCredentialResult {
    pub mall_info: Option<MallInfo>,
    pub query_detail_result: Option<QueryDetailResult>,
    pub basic_info_status: Option<BasicInfoStatus>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MallInfo {
    pub id: i64,
    pub mall_name: Option<String>,
    pub mall_desc: Option<String>,
    pub logo: Option<String>,
    pub is_open: Option<i32>,
    pub status: Option<i32>,
    pub mall_type: Option<i32>,
    pub company_name: Option<String>,
    pub company_address: Option<String>,
    pub contact_address: Option<String>,
    pub contact_province: Option<String>,
    pub contact_city: Option<String>,
    pub contact_district: Option<String>,
    pub contact_town: Option<String>,
    pub contact_detail_address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDetailResult {
    pub import_merchant_id: Option<i64>,
    pub operator_name: Option<String>,
    pub operator_mobile: Option<String>,
    pub operator_email: Option<String>,
    pub merchant_type: Option<i32>,
    pub mall_id: Option<i64>,
    pub mall_name: Option<String>,
    pub mall_logo: Option<String>,
    pub mall_desc: Option<String>,
    pub brand_name: Option<String>,
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
    pub audit_status: Option<i32>,
    pub can_edit: Option<bool>,
    pub can_modify_key_info: Option<bool>,
    pub has_bind_bank_card: Option<i32>,
    pub has_deposited: Option<bool>,
    pub contact_address: Option<String>,
    pub id_card_number: Option<String>,
    pub id_card_expiry_time: Option<String>,
    pub id_card_front_url: Option<String>,
    pub id_card_back_url: Option<String>,
    pub modify_mall_name_remaining_count: Option<i32>,
    pub individuality: Option<Individuality>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Individuality {
    pub operator_idcard_number: Option<String>,
    pub operator_backup_name: Option<String>,
    pub operator_backup_mobile: Option<String>,
    pub responsible_person_idcard_front_img_url: Option<String>,
    pub responsible_person_idcard_back_img_url: Option<String>,
    pub id_card_begin_time: Option<String>,
    pub id_card_end_time: Option<String>,
    pub company_name: Option<String>,
    pub company_register_address: Option<String>,
    pub legal_representative_name: Option<String>,
    pub is_individual_business: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicInfoStatus {
    pub mall_logo_status: Option<i32>,
    pub mall_logo: Option<String>,
    pub mall_desc_status: Option<i32>,
    pub mall_desc: Option<String>,
    pub show_mall_desc: Option<bool>,
}

// ─────────────────────────────────────────────────────────────────────────────
// send_message
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResp {
    pub success: bool,
    pub error_code: Option<i64>,
    pub error_msg: Option<String>,
    pub result: Option<serde_json::Value>,
}

/// 发送客服消息到指定买家。
///
/// **anti_content / hash / random 必须由 JS 端提供**（浏览器环境才能生成）：
/// - `anti_content` : `window.baseUtil.getRiskCtrCrawlerInfo()`
/// - `hash`         : 内部签名（bundle 模块 622 生成，64 位 hex），可留空测试
/// - `random`       : `md5([mall_id,global_uid,uid,content,"rpZigw#&iy$!KQD8"].join("@"))[0..16]`
///                    `+ md5(randomString())[0..16]`
pub async fn send_message(
    webview: &Webview,
    uid: &str,
    content: &str,
    anti_content: &str,
    hash: &str,
    random: &str,
    request_id: i64,
) -> Result<SendMessageResp, HttpError> {
    let cookie_str = match webview.cookies() {
        Ok(cookies) => cookies
            .iter()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect::<Vec<_>>()
            .join("; "),
        Err(_) => String::new(),
    };

    let intercepted = REQUEST_HEADERS.lock().unwrap()
        .get(webview.label())
        .cloned()
        .unwrap_or_default();

    let get = |key: &str, fallback: &str| -> String {
        intercepted.get(key).cloned().unwrap_or_else(|| fallback.to_string())
    };

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let body = json!({
        "data": {
            "cmd": "send_message",
            "anti_content": anti_content,
            "request_id": request_id,
            "message": {
                "to":   { "role": "user",     "uid": uid },
                "from": { "role": "mall_cs" },
                "ts":          ts,
                "content":     content,
                "msg_id":      null,
                "type":        0,
                "is_aut":      0,
                "manual_reply": 1,
                "status":      "read",
                "is_read":     1,
                "hash":        hash
            },
            "random": random
        },
        "client": "WEB",
        "anti_content": anti_content
    });

    let client = HttpClient::new();
    let headers: Vec<(&str, String)> = vec![
        ("accept", "*/*".to_string()),
        ("accept-language", get("accept-language", "zh-CN,zh;q=0.9")),
        ("cache-control", "no-cache".to_string()),
        ("content-type", "application/json".to_string()),
        ("priority", get("priority", "u=1, i")),
        ("referer", "https://mms.pinduoduo.com/chat-merchant/index.html".to_string()),
        ("sec-ch-ua", get("sec-ch-ua", "")),
        ("sec-ch-ua-mobile", get("sec-ch-ua-mobile", "?0")),
        ("sec-ch-ua-platform", get("sec-ch-ua-platform", "\"Windows\"")),
        ("sec-fetch-dest", "empty".to_string()),
        ("sec-fetch-mode", "cors".to_string()),
        ("sec-fetch-site", "same-origin".to_string()),
        ("user-agent", get("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36")),
        ("anti-content", get("anti-content", "")),
        ("cookie", cookie_str.clone()),
    ];
    let headers_ref: Vec<(&str, &str)> = headers.iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    let resp = client
        .request(
            Method::POST,
            "https://mms.pinduoduo.com/plateau/chat/send_message",
            Some(&body),
            Some(&headers_ref),
        )
        .await?;

    resp.json()
}

pub async fn query_final_credential_new(webview: &Webview) -> Result<QueryFinalCredentialResp, HttpError> {
    let cookie_str = match webview.cookies() {
        Ok(cookies) => cookies
            .iter()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect::<Vec<_>>()
            .join("; "),
        Err(_) => String::new(),
    };

    // 取该 webview 拦截到的请求头（由 feige_intercept 脚本收集）
    let intercepted = REQUEST_HEADERS.lock().unwrap()
        .get(webview.label())
        .cloned()
        .unwrap_or_default();

    // 静态兜底：拦截到的头里没有时使用的默认值
    let get = |key: &str, fallback: &str| -> String {
        intercepted.get(key).cloned().unwrap_or_else(|| fallback.to_string())
    };

    let client = HttpClient::new();
    let headers: Vec<(&str, String)> = vec![
        ("accept", "*/*".to_string()),
        ("accept-language", get("accept-language", "zh-CN,zh;q=0.9")),
        ("cache-control", "no-cache".to_string()),
        ("priority", get("priority", "u=1, i")),
        ("referer", "https://mms.pinduoduo.com/mallcenter/info/basic".to_string()),
        ("sec-ch-ua", get("sec-ch-ua", "")),
        ("sec-ch-ua-mobile", get("sec-ch-ua-mobile", "?0")),
        ("sec-ch-ua-platform", get("sec-ch-ua-platform", "\"Windows\"")),
        ("sec-fetch-dest", "empty".to_string()),
        ("sec-fetch-mode", "cors".to_string()),
        ("sec-fetch-site", "same-origin".to_string()),
        ("user-agent", get("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36")),
        ("anti-content", get("anti-content", "")),
        ("cookie", cookie_str.clone()),
    ];
    // 过滤掉值为空的头（比如 anti-content 还没拦截到时）
    let headers_ref: Vec<(&str, &str)> = headers.iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    let resp = client
        .get_with_headers(
            "https://mms.pinduoduo.com/earth/api/mallInfo/queryFinalCredentialNew",
            &headers_ref,
        )
        .await?;

    resp.json()
}

#[cfg(test)]
mod tests {
    use crate::utils;
    use super::*;

    #[tokio::test]
    async fn test_query_final_credential_new() {
        utils::init_logger();
        // let result = query_final_credential_new().await;
        // match result {
        //     Ok(resp) => {
        //         assert!(resp.success);
        //         assert_eq!(resp.error_code, 1000000);
        //         if let Some(r) = &resp.result {
        //             if let Some(mall) = &r.mall_info {
        //                 log::info!("店铺名称: {:?}", mall.mall_name);
        //                 log::info!("店铺ID: {}", mall.id);
        //             }
        //             if let Some(detail) = &r.query_detail_result {
        //                 log::info!("经营者: {:?}", detail.operator_name);
        //             }
        //         }
        //     }
        //     Err(e) => {
        //         log::error!("请求失败: {}", e);
        //     }
        // }
    }
}
