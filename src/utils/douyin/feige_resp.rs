use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use log::info;
use prost::Message;
use crate::commands::shop_callback::FEIGE_MANAGEMENT_COOKIE;
use crate::utils::douyin::protobuf::im_proto;
use crate::utils::HttpClient;
use serde::{Deserialize, Serialize};
use tauri::{Webview, Window};

const FEIGE_BASE_URL: &str = "https://fxg.jinritemai.com";

pub static SHOP_INFO_PARAMS: LazyLock<Mutex<HashMap<String, FeigeShopInfoParams>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static REQUEST_HEADERS: LazyLock<Mutex<HashMap<String, HashMap<String, String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static PIGEON_SIGN_MAP: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// WebSocket 连接时从 URL 拿到的 IM token（protobuf Request.token 字段）
pub static IM_TOKEN_MAP: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// WebSocket 连接时从 URL 拿到的 device_id（protobuf Request.device_id 字段）
pub static IM_DEVICE_ID_MAP: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));





#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeigeShopInfoParams {
    pub version: Option<String>,
    pub appid: Option<String>,
    #[serde(rename = "__token")]
    pub token: Option<String>,
    #[serde(rename = "_bid")]
    pub bid: Option<String>,
    #[serde(rename = "_lid")]
    pub lid: Option<String>,
    #[serde(rename = "verifyFp")]
    pub verify_fp: Option<String>,
    pub fp: Option<String>,
    #[serde(rename = "msToken")]
    pub ms_token: Option<String>,
    #[serde(rename = "a_bogus")]
    pub a_bogus: Option<String>,
}

impl Default for FeigeShopInfoParams {
    fn default() -> Self {
        Self {
            version: Some("0".to_string()),
            appid: Some("1".to_string()),
            token: Some("dec6825093e192e7986d5c20fde6f1ac".to_string()),
            bid: Some("fxg_admin".to_string()),
            lid: Some("676892719333".to_string()),
            verify_fp: Some("verify_mp527ktm_mJeZdDCs_MtIX_4vRz_B9rL_JADHn91hXVlo".to_string()),
            fp: Some("verify_mp527ktm_mJeZdDCs_MtIX_4vRz_B9rL_JADHn91hXVlo".to_string()),
            ms_token: Some("8Kb8SFlPhwPTVEQeTCelJriT4ZRpJVt9XZrT9w9EhVxj8CplumOALgWL4lv-uwtgmLndepMGchnvAH36U8yZXJKN-VICLEJxN3K7Y8UCFeDMjmsUGQJ7yCMmsW-pF-cDiqN1-tQYeYhK552ZtijPxdxn7Xi2d4-IXGqv6re0asIKGAr23fvHwd4=".to_string()),
            a_bogus: Some("Ev0nDey7Op8fa3CGuOnI75xli2LMrPWyK1T/RFazH1cpPhFaq01FbBcsjoLCm52hX8BwNHV7GjlAYxVcYHT0Ze9kKmkvSkty1s5CV8fLZqiZGMU8DqWsS8kzww0z05wia5VUi1fUhUGHZnOWDZQm/-lyHA8CQ5gZFq9ykqYbOIGVZ0LlEZnlPdGZOhGqLD==".to_string()),
        }
    }
}

impl FeigeShopInfoParams {
    pub fn update_from_query(&mut self, query: &serde_json::Map<String, serde_json::Value>) {
        for (key, value) in query {
            if let Some(val) = value.as_str() {
                match key.as_str() {
                    "version" => self.version = Some(val.to_string()),
                    "appid" => self.appid = Some(val.to_string()),
                    "__token" => self.token = Some(val.to_string()),
                    "_bid" => self.bid = Some(val.to_string()),
                    "_lid" => self.lid = Some(val.to_string()),
                    "verifyFp" => self.verify_fp = Some(val.to_string()),
                    "fp" => self.fp = Some(val.to_string()),
                    "msToken" => self.ms_token = Some(val.to_string()),
                    "a_bogus" => self.a_bogus = Some(val.to_string()),
                    _ => {}
                }
            }
        }
    }

    pub fn to_query_string(&self) -> String {
        let mut params: Vec<String> = Vec::new();
        if let Some(ref v) = self.version { params.push(format!("version={}", v)); }
        if let Some(ref v) = self.appid { params.push(format!("appid={}", v)); }
        if let Some(ref v) = self.token { params.push(format!("__token={}", v)); }
        if let Some(ref v) = self.bid { params.push(format!("_bid={}", v)); }
        if let Some(ref v) = self.lid { params.push(format!("_lid={}", v)); }
        if let Some(ref v) = self.verify_fp { params.push(format!("verifyFp={}", v)); }
        if let Some(ref v) = self.fp { params.push(format!("fp={}", v)); }
        if let Some(ref v) = self.ms_token { params.push(format!("msToken={}", v)); }
        if let Some(ref v) = self.a_bogus { params.push(format!("a_bogus={}", v)); }
        params.join("&")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeShopLogo {
    pub file_id: Option<String>,
    pub url: Option<String>,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeShopNameSection {
    pub not_allow_update: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeQualStatusInfo {
    pub task_id: Option<String>,
    pub status: Option<i32>,
    pub audit_status: Option<i32>,
    pub reject_reasons: Option<Vec<String>>,
    pub submit_time: Option<String>,
    pub audit_time: Option<String>,
    pub build_source: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeShopForm {
    pub shop_form_enum: Option<i32>,
    pub shop_form_code: Option<String>,
    pub shop_form_cn: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeAddressItem {
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeShopAddress {
    pub province: Option<FeigeAddressItem>,
    pub city: Option<FeigeAddressItem>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeShopSettleDay {
    pub detail_url: Option<String>,
    pub delay_day_list: Option<Vec<i32>>,
    pub delay_day_display: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeShopInfoData {
    pub qual_id: Option<String>,
    pub shop_type: Option<i32>,
    pub shop_name: Option<String>,
    pub shop_logo: Option<FeigeShopLogo>,
    pub brand_src: Option<i32>,
    pub portrait_auth_term: Option<String>,
    pub taxfree_licence_term: Option<String>,
    pub shop_name_section: Option<FeigeShopNameSection>,
    pub is_auto_generated_shop_logo: Option<bool>,
    pub qual_status_info: Option<FeigeQualStatusInfo>,
    pub shop_form: Option<FeigeShopForm>,
    pub shop_actual_address: Option<FeigeShopAddress>,
    pub personal_business_scope: Option<Vec<i32>>,
    pub allow_store_reuse_qual: Option<i32>,
    pub shop_has_brand35: Option<bool>,
    pub poi_id: Option<String>,
    pub custom_id: Option<String>,
    pub shop_settle_day: Option<FeigeShopSettleDay>,
    pub operate_status: Option<i32>,
    pub operate_status_str: Option<String>,
    pub shop_open_time: Option<i64>,
    pub shop_open_time_str: Option<String>,
    pub allow_update_main_category: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeExtra {
    pub log_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeShopInfoResponse {
    pub data: Option<FeigeShopInfoData>,
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub extra: Option<FeigeExtra>,
}

fn get_webview_cookie(webview: &Webview) -> String {
    webview
        .cookies()
        .map(|cookies| {
            cookies
                .iter()
                .map(|c| format!("{}={}", c.name(), c.value()))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}

fn get_management_cookie(webview_id: &str) -> String {
    FEIGE_MANAGEMENT_COOKIE
        .lock()
        .unwrap()
        .get(webview_id)
        .map(|cookies| {
            cookies
                .iter()
                .map(|c| format!("{}={}", c.name(), c.value()))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}

pub async fn feige_shop_info(webview_id: &str,webview: &Webview) -> Result<FeigeShopInfoResponse, Box<dyn std::error::Error>> {
    let cookie_str = get_webview_cookie(webview);

    let query_string = SHOP_INFO_PARAMS
        .lock()
        .unwrap()
        .get(webview_id)
        .map(|p| p.to_query_string())
        .unwrap_or_default();

    let url = format!(
        "https://fxg.jinritemai.com/center/qualification/shop/info?{}",
        query_string
    );

    let (accept, accept_language, priority, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, sec_fetch_dest, sec_fetch_mode, sec_fetch_site, user_agent) = {
        let headers_map = REQUEST_HEADERS.lock().unwrap();
        let webview_headers = headers_map.get(webview_id);

        let accept = webview_headers
            .and_then(|h| h.get("accept"))
            .cloned()
            .unwrap_or_else(|| "application/json, text/plain, */*".to_string());
        let accept_language = webview_headers
            .and_then(|h| h.get("accept-language"))
            .cloned()
            .unwrap_or_else(|| "zh-CN,zh;q=0.9,en;q=0.8".to_string());
        let priority = webview_headers
            .and_then(|h| h.get("priority"))
            .cloned()
            .unwrap_or_else(|| "u=1, i".to_string());
        let sec_ch_ua = webview_headers
            .and_then(|h| h.get("sec-ch-ua"))
            .cloned()
            .unwrap_or_else(|| "\"Google Chrome\";v=\"147\", \"Not.A/Brand\";v=\"8\", \"Chromium\";v=\"147\"".to_string());
        let sec_ch_ua_mobile = webview_headers
            .and_then(|h| h.get("sec-ch-ua-mobile"))
            .cloned()
            .unwrap_or_else(|| "?0".to_string());
        let sec_ch_ua_platform = webview_headers
            .and_then(|h| h.get("sec-ch-ua-platform"))
            .cloned()
            .unwrap_or_else(|| "\"Windows\"".to_string());
        let sec_fetch_dest = webview_headers
            .and_then(|h| h.get("sec-fetch-dest"))
            .cloned()
            .unwrap_or_else(|| "empty".to_string());
        let sec_fetch_mode = webview_headers
            .and_then(|h| h.get("sec-fetch-mode"))
            .cloned()
            .unwrap_or_else(|| "cors".to_string());
        let sec_fetch_site = webview_headers
            .and_then(|h| h.get("sec-fetch-site"))
            .cloned()
            .unwrap_or_else(|| "same-origin".to_string());
        let user_agent = webview_headers
            .and_then(|h| h.get("user-agent"))
            .cloned()
            .unwrap_or_else(|| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".to_string());

        (accept, accept_language, priority, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, sec_fetch_dest, sec_fetch_mode, sec_fetch_site, user_agent)
    };

    let headers: &[(&str, &str)] = &[
        ("accept", &accept),
        ("accept-language", &accept_language),
        ("priority", &priority),
        ("referer", "https://fxg.jinritemai.com/ffa/mshop/homepage/index"),
        ("sec-ch-ua", &sec_ch_ua),
        ("sec-ch-ua-mobile", &sec_ch_ua_mobile),
        ("sec-ch-ua-platform", &sec_ch_ua_platform),
        ("sec-fetch-dest", &sec_fetch_dest),
        ("sec-fetch-mode", &sec_fetch_mode),
        ("sec-fetch-site", &sec_fetch_site),
        ("user-agent", &user_agent),
        ("cookie", &cookie_str),
    ];



    let res = HttpClient::new().get_with_headers(&url, headers).await?;

    if res.is_success() {
        match res.json::<FeigeShopInfoResponse>() {
            Ok(parsed) => Ok(parsed),
            Err(e) => {
                log::error!("[feige_shop_info] JSON 解析失败: {}", e);
                Err(e.into())
            }
        }
    } else {
        log::error!("[feige_shop_info] 请求失败, url: {}, webview_id: {}, status: {}", url, webview_id, res.status);
        Err(format!("HTTP 请求失败, status: {}", res.status).into())
    }
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnoseAlertDialog {
    pub title: Option<String>,
    pub content: Option<String>,
    pub switch_dialog: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnoseIgnoreOperation {
    pub name: Option<String>,
    pub key: Option<String>,
    pub status: Option<String>,
    pub reason: Option<String>,
    pub app_link: Option<String>,
    pub alert_dialog: Option<DiagnoseAlertDialog>,
    pub jump_url: Option<String>,
    pub hover: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnoseFieldProblem {
    pub key: Option<String>,
    pub name: Option<String>,
    pub problem_key: Option<i32>,
    pub problem_name: Option<String>,
    pub suggestion: Option<String>,
    pub base_score_suggestion: Option<String>,
    pub is_ignored: Option<bool>,
    pub is_support_ignore: Option<bool>,
    pub similar_product_list: Option<serde_json::Value>,
    pub title_typo: Option<String>,
    pub is_base_score: Option<bool>,
    pub problem_affect: Option<Vec<String>>,
    pub diagnose_target: Option<String>,
    pub ignore_operation: Option<DiagnoseIgnoreOperation>,
    pub is_quality_score: Option<bool>,
    pub detail_link: Option<String>,
    pub item_rcmd_effect: Option<serde_json::Value>,
    pub problem_category: Option<String>,
    pub jump_link: Option<String>,
    pub is_app_support: Option<bool>,
    pub problem_score: Option<String>,
    pub conversion_revenue_msg: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnoseListOperation {
    pub name: Option<String>,
    pub key: Option<String>,
    pub status: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnoseRecommendTagInfo {
    pub tag_type: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnoseRecommend {
    pub name: Option<String>,
    pub props_map: Option<serde_json::Value>,
    pub suggest_seo_title_words: Option<Vec<String>>,
    pub recommend_tag_info_list: Option<Vec<DiagnoseRecommendTagInfo>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnoseProductDetailData {
    pub product_id: Option<i64>,
    pub product_name: Option<String>,
    pub product_status: Option<i32>,
    pub product_check_status: Option<i32>,
    pub img: Option<String>,
    pub problem_num_to_improve: Option<i32>,
    pub field_problem: Option<Vec<DiagnoseFieldProblem>>,
    pub list_operation: Option<Vec<DiagnoseListOperation>>,
    pub is_checking: Option<bool>,
    pub diagnose_recommend: Option<DiagnoseRecommend>,
    pub quality_score: Option<i32>,
    pub quality_level: Option<String>,
    pub suggest_field_problem: Option<Vec<DiagnoseFieldProblem>>,
    pub conversion_revenue_msg: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetDiagnoseProductDetailResponse {
    pub errno: Option<i32>,
    pub st: Option<i32>,
    pub msg: Option<String>,
    pub code: Option<i32>,
    pub data: Option<DiagnoseProductDetailData>,
    pub page: Option<i32>,
    pub total: Option<i32>,
    pub size: Option<i32>,
}

pub async fn get_diagnose_product_detail(
    webview_id: &str,
    webview: &Webview,
    product_id: &str,
) -> Result<GetDiagnoseProductDetailResponse, Box<dyn std::error::Error>> {
    let cookie_str = get_management_cookie(webview_id);

    let query_string = SHOP_INFO_PARAMS
        .lock()
        .unwrap()
        .get(webview_id)
        .map(|p| p.to_query_string())
        .unwrap_or_default();

    let url = format!(
        "{}/product_diagnose/tproduct/get_diagnose_product_detail?product_id={}&{}",
        FEIGE_BASE_URL, product_id, query_string
    );

    let referer = format!(
        "https://fxg.jinritemai.com/ffa/g/create?product_id={}",
        product_id
    );

    let (accept, accept_language, priority, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, sec_fetch_dest, sec_fetch_mode, sec_fetch_site, user_agent) = {
        let headers_map = REQUEST_HEADERS.lock()?;
        let h = headers_map.get(webview_id);

        let accept = h.and_then(|h| h.get("accept")).cloned()
            .unwrap_or_else(|| "application/json, text/plain, */*".to_string());
        let accept_language = h.and_then(|h| h.get("accept-language")).cloned()
            .unwrap_or_else(|| "zh-CN,zh;q=0.9,en;q=0.8".to_string());
        let priority = h.and_then(|h| h.get("priority")).cloned()
            .unwrap_or_else(|| "u=1, i".to_string());
        let sec_ch_ua = h.and_then(|h| h.get("sec-ch-ua")).cloned()
            .unwrap_or_else(|| "\"Google Chrome\";v=\"148\", \"Not.A/Brand\";v=\"8\", \"Chromium\";v=\"148\"".to_string());
        let sec_ch_ua_mobile = h.and_then(|h| h.get("sec-ch-ua-mobile")).cloned()
            .unwrap_or_else(|| "?0".to_string());
        let sec_ch_ua_platform = h.and_then(|h| h.get("sec-ch-ua-platform")).cloned()
            .unwrap_or_else(|| "\"Windows\"".to_string());
        let sec_fetch_dest = h.and_then(|h| h.get("sec-fetch-dest")).cloned()
            .unwrap_or_else(|| "empty".to_string());
        let sec_fetch_mode = h.and_then(|h| h.get("sec-fetch-mode")).cloned()
            .unwrap_or_else(|| "cors".to_string());
        let sec_fetch_site = h.and_then(|h| h.get("sec-fetch-site")).cloned()
            .unwrap_or_else(|| "same-origin".to_string());
        let user_agent = h.and_then(|h| h.get("user-agent")).cloned()
            .unwrap_or_else(|| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36".to_string());

        (accept, accept_language, priority, sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform, sec_fetch_dest, sec_fetch_mode, sec_fetch_site, user_agent)
    };

    let headers: &[(&str, &str)] = &[
        ("accept", &accept),
        ("accept-language", &accept_language),
        ("cookie", &cookie_str),
        ("priority", &priority),
        ("referer", &referer),
        ("sec-ch-ua", &sec_ch_ua),
        ("sec-ch-ua-mobile", &sec_ch_ua_mobile),
        ("sec-ch-ua-platform", &sec_ch_ua_platform),
        ("sec-fetch-dest", &sec_fetch_dest),
        ("sec-fetch-mode", &sec_fetch_mode),
        ("sec-fetch-site", &sec_fetch_site),
        ("user-agent", &user_agent),
        ("x-tt-from-appid", "ffa-goods"),
        ("x-tt-from-end", "PC"),
        ("x-tt-from-page", "https://fxg.jinritemai.com/ffa/g/create"),
        ("x-tt-from-version", "1.0.1.3849"),
    ];

    let res = HttpClient::new().get_with_headers(&url, headers).await?;

    if res.is_success() {
        match res.json::<GetDiagnoseProductDetailResponse>() {
            Ok(parsed) => Ok(parsed),
            Err(e) => {
                log::error!("[get_diagnose_product_detail] JSON 解析失败: {}", e);
                Err(e.into())
            }
        }
    } else {
        log::error!(
            "[get_diagnose_product_detail] 请求失败, product_id={}, status={}",
            product_id, res.status
        );
        Err(format!("HTTP 请求失败, status: {}", res.status).into())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionH5Extra {
    pub fatal_item_ids: Option<Vec<serde_json::Value>>,
    pub logid: Option<String>,
    pub now: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionH5LogPb {
    pub impr_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionPrice {
    pub min_price: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionPriceInfo {
    pub price: Option<PromotionPrice>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionTitleInfo {
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionBottomLabel {
    pub render_type: Option<i32>,
    pub label_type: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionBasicInfoData {
    pub price_info: Option<PromotionPriceInfo>,
    pub title_info: Option<PromotionTitleInfo>,
    pub bottom_label_list: Option<Vec<PromotionBottomLabel>>,
    pub product_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionShopLinkButton {
    pub text: Option<String>,
    pub background: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionGradeInfo {
    pub is_new_style: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionShopBasicInfo {
    pub shop_logo: Option<String>,
    pub shop_name: Option<String>,
    pub shop_link_button: Option<PromotionShopLinkButton>,
    pub brand_icon: Option<String>,
    pub new_brand_icon: Option<String>,
    pub grade_info: Option<PromotionGradeInfo>,
    pub shop_extra_info: Option<serde_json::Value>,
    pub shop_icon_list: Option<serde_json::Value>,
    pub background: Option<String>,
    pub status_icon_info: Option<serde_json::Value>,
    pub account_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionShopInfo {
    pub basic_info: Option<PromotionShopBasicInfo>,
    pub style_type: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionCombinationInfo {
    pub combination_jump: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionMediaContent {
    pub url: Option<String>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub combination_info: Option<PromotionCombinationInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionMedia {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub content_list: Option<Vec<PromotionMediaContent>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionHeadFigureData {
    pub media_list: Option<Vec<PromotionMedia>>,
    pub container_size: Option<i32>,
    pub style_type: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionHeaderContent {
    pub background: Option<String>,
    pub service_tag: Option<serde_json::Value>,
    pub content_color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionSafetyPopupItem {
    pub left_icon: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionCom {
    #[serde(rename = "type")]
    pub com_type: Option<String>,
    pub text: Option<String>,
    pub color: Option<String>,
    pub url: Option<String>,
    pub border_radius: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionContentItem {
    pub render_type: Option<i32>,
    pub label_type: Option<i32>,
    pub link: Option<String>,
    pub coms: Option<Vec<PromotionCom>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionSafetyContent {
    pub safety_popup_content: Option<Vec<PromotionSafetyPopupItem>>,
    pub service_type: Option<String>,
    pub content_list: Option<Vec<PromotionContentItem>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionLogisticsContent {
    pub title: Option<String>,
    pub content_list: Option<Vec<PromotionContentItem>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionProductSupportInfoData {
    pub header_content: Option<PromotionHeaderContent>,
    pub safety_content: Option<PromotionSafetyContent>,
    pub logistics_content: Option<PromotionLogisticsContent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionTitleBar {
    pub link: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionCommentSection {
    pub title_bar: Option<PromotionTitleBar>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionCommentData {
    pub good_comment: Option<PromotionCommentSection>,
    pub buyer_show: Option<PromotionCommentSection>,
    pub shop_extra_comment: Option<PromotionCommentSection>,
    pub good_evaluation: Option<serde_json::Value>,
    pub style_type: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionSmallButton {
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub subscript: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionTextInfo {
    pub content: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionBigButton {
    pub button_type: Option<i32>,
    pub button_status: Option<String>,
    pub text_info: Option<PromotionTextInfo>,
    pub sub_text_info: Option<serde_json::Value>,
    pub bg_color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionFooterData {
    pub small_button_list: Option<Vec<PromotionSmallButton>>,
    pub big_button_list: Option<Vec<PromotionBigButton>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionH5Data {
    pub basic_info_data: Option<PromotionBasicInfoData>,
    pub shop_info: Option<PromotionShopInfo>,
    pub head_figure_data: Option<PromotionHeadFigureData>,
    pub product_support_info_data: Option<PromotionProductSupportInfoData>,
    pub comment_data: Option<PromotionCommentData>,
    pub shop_coupon_data: Option<serde_json::Value>,
    pub footer_data: Option<PromotionFooterData>,
    pub init_popup_data: Option<serde_json::Value>,
    pub pack_detail: Option<serde_json::Value>,
    pub page_data: Option<serde_json::Value>,
    pub redirect_location_v2: Option<String>,
    pub style_type: Option<i32>,
    pub collapse_detail: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromotionPackH5Response {
    pub extra: Option<PromotionH5Extra>,
    pub fallback_page: Option<serde_json::Value>,
    pub log_pb: Option<PromotionH5LogPb>,
    pub status_code: Option<i32>,
    pub promotion_h5: Option<PromotionH5Data>,
    pub detail_url: Option<String>,
}

pub async fn get_promotion_pack_h5(
    window: &Window,
    webview_id: &str,
    product_id: &str,
) -> Result<PromotionPackH5Response, Box<dyn std::error::Error>> {
    let body =
        crate::commands::promotion_h5::get_promotion_pack_h5_via_webview(window, webview_id, product_id)
            .await?;

    match serde_json::from_str::<PromotionPackH5Response>(&body) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            log::error!(
                "[get_promotion_pack_h5] JSON 解析失败: {}, 响应前500字节: {}",
                e,
                &body[..body.len().min(500)]
            );
            Err(e.into())
        }
    }
}

pub async fn get_by_conversation(
    webview: &Webview,
    security_conversation_id: &str,
    conversation_short_id: i64,

) -> Result<Vec<im_proto::MessageBody>, Box<dyn std::error::Error>> {
    let cookie_str = get_webview_cookie(webview);
    let pigeon_sign = PIGEON_SIGN_MAP
        .lock()
        .unwrap()
        .get(webview.label())
        .cloned()
        .unwrap_or_default();

    if pigeon_sign.is_empty() {
        info!("[get_by_conversation] PIGEON_SIGN_MAP 中未找到 webview={} 的 pigeon_sign", webview.label());
    }

    let conv_body = im_proto::MessagesInConversationRequestBody {
        conversation_id: Some("".to_string()),
        conversation_type: Some(10),
        conversation_short_id: Some(conversation_short_id),
        direction: Some(1),
        anchor_index: Some(0),
        limit: Some(14),
        security_conversation_id: Some(security_conversation_id.to_string()),
        ..Default::default()
    };

    let mut body = im_proto::RequestBody::default();
    body.messages_in_conversation_body = Some(conv_body);

    let mut headers = HashMap::new();
    headers.insert("pigeon_source".to_string(), "web".to_string());
    headers.insert("PIGEON_BIZ_TYPE".to_string(), "2".to_string());
    headers.insert("pigeon_sign".to_string(), pigeon_sign);
    headers.insert("user_agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".to_string());
    headers.insert("referer".to_string(), "https://im.jinritemai.com/pc_seller/".to_string());

    let mut request = im_proto::Request::default();
    request.cmd = Some(301);
    request.sequence_id = Some(10020);
    request.sdk_version = Some("0.0.0-fix-fix-inbox-array-202632312465".to_string());
    request.refer = Some(3);
    request.inbox_type = Some(3);
    request.build_number = Some("0e70fed:fix/fix_inbox_array".to_string());
    request.body = Some(body);
    request.device_platform = Some("web".to_string());
    request.headers = headers;
    request.auth_type = Some(2);

    // ⬇️ 关键：从全局 Map 取 token 和 device_id，没有这两个字段服务端会返回 150 参数错误
    let im_token = IM_TOKEN_MAP
        .lock()
        .unwrap()
        .get(webview.label())
        .cloned()
        .unwrap_or_default();
    let device_id = IM_DEVICE_ID_MAP
        .lock()
        .unwrap()
        .get(webview.label())
        .cloned()
        .unwrap_or_default();

    if im_token.is_empty() {
        info!("[get_by_conversation] ⚠️ IM_TOKEN_MAP 中未找到 webview={} 的 token", webview.label());
    }
    if device_id.is_empty() {
        info!("[get_by_conversation] ⚠️ IM_DEVICE_ID_MAP 中未找到 webview={} 的 device_id", webview.label());
    }

    request.token = Some(im_token);
    request.device_id = Some(device_id);

    let url = format!("{}/pigeon_im/v1/message/get_by_conversation", FEIGE_BASE_URL);

    let resp_bytes = HttpClient::new()
        .request_raw(
            reqwest::Method::POST,
            &url,
            request.encode_to_vec(),
            Some(&[
                ("content-type", "application/x-protobuf"),
                ("accept", "application/x-protobuf"),
                ("cookie", &cookie_str),
                ("origin", "https://im.jinritemai.com"),
                ("referer", "https://im.jinritemai.com/"),
            ]),
        )
        .await?;

    // info!("[get_by_conversation] resp len: {}", resp_bytes.len());
    // info!("[get_by_conversation] resp hex: {}", hex::encode(&resp_bytes));

    let response = im_proto::Response::decode(resp_bytes.as_slice())?;
    // println!("[get_by_conversation] response: {:?}", response);

    if response.status_code != Some(0) {
        info!("[get_by_conversation] 请求失败, url: {}, request: {:?}", url, request);
    }

    let response_body = response.body.ok_or("Response 缺少 body")?;
    let conv_body = response_body
        .messages_in_conversation_body
        .ok_or("缺少 messages_in_conversation_body")?;


    let messages: Vec<_> = conv_body.messages.into_iter().filter(|msg| {
        msg.message_type == Some(1000)
            && msg.ext.get("s:sender_biz_role")
                .map(|r| r == "Buyer" || r == "CurrentServer")
                .unwrap_or(false)
            && msg.ext.get("type").map(|t| t != "allocated_service").unwrap_or(true)
    }).collect();

    for msg in &messages {
        let role = msg.ext.get("s:sender_biz_role").map(|s| s.as_str()).unwrap_or("unknown");
        let msg_type = msg.ext.get("type").map(|s| s.as_str()).unwrap_or("");
        let content = msg.content.as_deref().unwrap_or("");

        match msg_type {
            "template_card" => {
                let goods_id = msg.ext.get("goods_id").map(|s| s.as_str()).unwrap_or("");
                let img = msg.ext.get("static_data")
                    .and_then(|sd| serde_json::from_str::<serde_json::Value>(sd).ok())
                    .and_then(|v| {
                        v.get("sale_goods")?.get(0)?.get("img")?.as_str().map(|s| s.to_string())
                    })
                    .unwrap_or_default();
                info!(
                    "[get_by_conversation] role={} type={} goods_id={} img={}",
                    role, msg_type, goods_id, img
                );
            }
            "file_image" => {
                let image_url = msg.ext.get("imageUrl").map(|s| s.as_str()).unwrap_or("");
                info!(
                    "[get_by_conversation] role={} type={} imageUrl={}",
                    role, msg_type, image_url
                );
            }
            _ => {
                info!(
                    "[get_by_conversation] role={} type={} content={}",
                    role, msg_type, content
                );
            }
        }
    }

    Ok(messages)
}