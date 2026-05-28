use crate::utils::douyin::feige_resp::REQUEST_HEADERS;
use crate::utils::http::{HttpClient, HttpError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
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
    anti_content_outer: &str,
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
        "anti_content": anti_content_outer
    });

    let client = HttpClient::new();
    let headers: Vec<(&str, String)> = vec![
        ("accept", "*/*".to_string()),
        ("accept-language", get("accept-language", "zh-CN,zh;q=0.9")),
        ("cache-control", "no-cache".to_string()),
        ("content-type", "application/json".to_string()),
        ("priority", get("priority", "u=1, i")),
        ("referer", "https://mms.pinduoduo.com/chat-merchant/index.htm".to_string()),
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

    log::info!("[PDD][send_message] request body: {}", body);

    let resp = client
        .request(
            Method::POST,
            "https://mms.pinduoduo.com/plateau/chat/send_message",
            Some(&body),
            Some(&headers_ref),
        )
        .await?;

    let result: SendMessageResp = resp.json()?;
    log::info!("[PDD][send_message] response: {:?}", result);
    Ok(result)
}

// ─────────────────────────────────────────────────────────────────────────────
// get_chat_list
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatListResp {
    pub success: bool,
    pub error_code: Option<i64>,
    pub error_msg: Option<String>,
    pub result: Option<ChatListResult>,
}

#[derive(Debug, Deserialize)]
pub struct ChatListResult {
    pub response: Option<String>,
    pub request_id: Option<i64>,
    /// "ok" on success
    #[serde(rename = "result")]
    pub result_status: Option<String>,
    pub has_more: Option<bool>,
    pub read_mark: Option<ReadMark>,
    pub messages: Option<Vec<ChatMessage>>,
    pub cs_infos: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ReadMark {
    pub user_last_read: Option<String>,
    pub min_supported_msg_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub from: Option<MessageParty>,
    pub to: Option<MessageParty>,
    /// 秒级时间戳（字符串）
    pub ts: Option<String>,
    pub content: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: Option<i32>,
    pub is_aut: Option<i32>,
    pub manual_reply: Option<i32>,
    pub status: Option<String>,
    pub is_read: Option<i32>,
    pub version: Option<i32>,
    pub msg_id: Option<String>,
    pub pre_msg_id: Option<String>,
    pub client_msg_id: Option<String>,
    #[serde(rename = "mallName")]
    pub mall_name: Option<String>,
    pub mall_context: Option<MallContext>,
    pub cs_type: Option<i32>,
    pub source_id: Option<i32>,
    pub template_name: Option<String>,
    pub no_unreply_hint: Option<i32>,
    /// type=41 等富文本消息的附加信息
    pub info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct MallContext {
    pub client_type: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MessageParty {
    pub role: Option<String>,
    pub uid: Option<String>,
    pub mall_id: Option<String>,
    pub csid: Option<String>,
    pub cs_uid: Option<String>,
}

/// 获取与指定买家的聊天消息列表。
///
/// - `uid`                : 买家 uid
/// - `start_msg_id`       : 翻页起始消息 ID，首次传 `None`
/// - `start_index`        : 起始偏移，首次传 `0`
/// - `size`               : 每页条数，建议 11
/// - `anti_content_inner` : `data.anti_content`（页面内部签名）
/// - `anti_content_outer` : 顶层 `anti_content`
/// - `request_id`         : 请求唯一 ID（毫秒时间戳）
pub async fn get_chat_list(
    webview: &Webview,
    uid: &str,
    start_msg_id: Option<i64>,
    start_index: i32,
    size: i32,
    anti_content_inner: &str,
    anti_content_outer: &str,
    request_id: i64,
) -> Result<ChatListResp, HttpError> {
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

    let body = json!({
        "data": {
            "cmd": "list",
            "request_id": request_id,
            "list": {
                "with": { "role": "user", "id": uid },
                "start_msg_id": start_msg_id,
                "start_index": start_index,
                "size": size
            },
            "notUpdateUnreplyTs": true,
            "anti_content": anti_content_inner
        },
        "client": "WEB",
        "anti_content": anti_content_outer
    });

    let client = HttpClient::new();
    let headers: Vec<(&str, String)> = vec![
        ("accept", "*/*".to_string()),
        ("accept-language", get("accept-language", "zh-CN,zh;q=0.9")),
        ("content-type", "application/json".to_string()),
        ("origin", "https://mms.pinduoduo.com".to_string()),
        ("priority", get("priority", "u=1, i")),
        ("referer", "https://mms.pinduoduo.com/chat-merchant/index.html".to_string()),
        ("sec-ch-ua", get("sec-ch-ua", "")),
        ("sec-ch-ua-mobile", get("sec-ch-ua-mobile", "?0")),
        ("sec-ch-ua-platform", get("sec-ch-ua-platform", "\"Windows\"")),
        ("sec-fetch-dest", "empty".to_string()),
        ("sec-fetch-mode", "cors".to_string()),
        ("sec-fetch-site", "same-origin".to_string()),
        ("user-agent", get("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36")),
        ("anti-content", get("anti-content", "")),
        ("cookie", cookie_str),
    ];
    let headers_ref: Vec<(&str, &str)> = headers.iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    // log::info!("[PDD][get_chat_list] uid={} start_index={} size={}", uid, start_index, size);

    let resp = client
        .request(
            Method::POST,
            "https://mms.pinduoduo.com/plateau/chat/list",
            Some(&body),
            Some(&headers_ref),
        )
        .await?;

    let result: ChatListResp = resp.json()?;
    // log::info!("[PDD][get_chat_list] response: {:?}", result);
    Ok(result)
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

// ─────────────────────────────────────────────────────────────────────────────
// get_goods_detail
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodsDetailResp {
    pub success: bool,
    pub error_code: Option<i64>,
    pub error_msg: Option<String>,
    pub result: Option<GoodsDetailResult>,
}

#[derive(Debug, Deserialize)]
pub struct GoodsDetailResult {
    pub goods_commit_id: Option<i64>,
    pub goods_id: Option<i64>,
    pub goods_sn: Option<String>,
    pub out_goods_sn: Option<String>,
    pub cat_id: Option<i64>,
    pub cats: Option<Vec<Option<String>>>,
    pub mall_id: Option<i64>,
    pub mall_name: Option<String>,
    pub goods_name: Option<String>,
    pub goods_desc: Option<String>,
    pub share_desc: Option<String>,
    pub market_price: Option<i64>,
    pub thumb_url: Option<String>,
    pub hd_thumb_url: Option<String>,
    pub goods_type: Option<i32>,
    pub goods_type_name: Option<String>,
    pub is_pre_sale: Option<i32>,
    pub is_refundable: Option<i32>,
    pub shipment_limit_second: Option<i64>,
    pub created_at: Option<i64>,
    pub status: Option<i32>,
    pub min_group_price: Option<i64>,
    pub max_group_price: Option<i64>,
    pub groups: Option<GoodsGroups>,
    pub carousel_gallery: Option<Vec<GalleryItem>>,
    pub detail_gallery: Option<Vec<GalleryItem>>,
    pub skus: Option<Vec<GoodsSku>>,
    pub commit_extension: Option<CommitExtension>,
    pub schedule_sale: Option<ScheduleSale>,
    pub extend_warranty_dto: Option<ExtendWarrantyDto>,
    pub goods_options: Option<Vec<i64>>,
    pub cat_id_1: Option<i64>,
    pub cat_id_2: Option<i64>,
    pub cat_id_3: Option<i64>,
    pub cat_id_4: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoodsGroups {
    pub single_price: Option<i64>,
    pub group_price: Option<i64>,
    pub customer_num: Option<i32>,
    pub buy_limit: Option<i64>,
    pub order_limit: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GalleryItem {
    pub url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    #[serde(rename = "type")]
    pub item_type: Option<i32>,
    pub video_url: Option<String>,
    pub file_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoodsSku {
    pub sku_id: Option<i64>,
    pub sku_sn: Option<String>,
    pub price: Option<i64>,
    pub multi_price: Option<i64>,
    pub quantity: Option<i64>,
    pub is_onsale: Option<i32>,
    pub limit_quantity: Option<i64>,
    pub weight: Option<i64>,
    pub thumb_url: Option<String>,
    pub spec: Option<Vec<serde_json::Value>>,
    pub out_sku_sn: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommitExtension {
    pub goods_commit_id: Option<i64>,
    pub production_license: Option<String>,
    pub production_standard_number: Option<String>,
    pub shelf_life: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleSale {
    pub sale_type: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ExtendWarrantyDto {
    pub extend_warranty: Option<i32>,
    pub extend_warranty_year: Option<i32>,
}

pub async fn get_goods_detail(
    webview: &Webview,
    goods_id: i64,
) -> Result<GoodsDetailResp, HttpError> {
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

    let body = json!({ "goods_id": goods_id });

    let client = HttpClient::new();
    let headers: Vec<(&str, String)> = vec![
        ("accept", "*/*".to_string()),
        ("accept-language", get("accept-language", "zh-CN,zh;q=0.9")),
        ("cache-control", "max-age=0".to_string()),
        ("content-type", "application/json".to_string()),
        ("origin", "https://mms.pinduoduo.com".to_string()),
        ("priority", get("priority", "u=1, i")),
        ("referer", "https://mms.pinduoduo.com/goods/goods_detail".to_string()),
        ("sec-ch-ua", get("sec-ch-ua", "")),
        ("sec-ch-ua-mobile", get("sec-ch-ua-mobile", "?0")),
        ("sec-ch-ua-platform", get("sec-ch-ua-platform", "\"Windows\"")),
        ("sec-fetch-dest", "empty".to_string()),
        ("sec-fetch-mode", "cors".to_string()),
        ("sec-fetch-site", "same-origin".to_string()),
        ("user-agent", get("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36")),
        ("anti-content", get("anti-content", "")),
        ("cookie", cookie_str),
    ];
    let headers_ref: Vec<(&str, &str)> = headers.iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    log::info!("[PDD][get_goods_detail] goods_id={}", goods_id);

    let resp = client
        .request(
            Method::POST,
            "https://mms.pinduoduo.com/glide/v2/mms/query/commit/on_shop/detail",
            Some(&body),
            Some(&headers_ref),
        )
        .await?;

    let result: GoodsDetailResp = resp.json()?;
    log::info!("[PDD][get_goods_detail] response: {:?}", result);
    Ok(result)
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
