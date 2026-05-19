use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::utils::HttpClient;
use serde::{Deserialize, Serialize};
use tauri::Webview;

pub static SHOP_INFO_PARAMS: LazyLock<Mutex<HashMap<String, FeigeShopInfoParams>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static REQUEST_HEADERS: LazyLock<Mutex<HashMap<String, HashMap<String, String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));



#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeigeCookie {
    pub s_v_web_id: Option<String>,
    pub passport_csrf_token: Option<String>,
    pub passport_csrf_token_default: Option<String>,
    pub x_web_secsdk_uid: Option<String>,
    pub hm_lvt_: Option<String>,
    pub hmaccount: Option<String>,
    pub passport_mfa_token: Option<String>,
    pub uid_tt: Option<String>,
    pub uid_tt_ss: Option<String>,
    pub sid_tt: Option<String>,
    pub sessionid: Option<String>,
    pub sessionid_ss: Option<String>,
    pub is_staff_user: Option<String>,
    pub has_biz_token: Option<String>,
    pub ucas_c0: Option<String>,
    pub ucas_c0_ss: Option<String>,
    pub phpsessid: Option<String>,
    pub phpsessid_ss: Option<String>,
    pub ecom_us_lt: Option<String>,
    pub ecom_us_lt_ss: Option<String>,
    pub zsgw_business_data: Option<String>,
    pub source: Option<String>,
    pub doudain_safety_did: Option<String>,
    pub csrf_session_id: Option<String>,
    pub shop_id: Option<String>,
    pub pigeon_cid: Option<String>,
    pub ffa_goods_ewid: Option<String>,
    pub ffa_goods_seraph_did: Option<String>,
    pub security_mc_1_s_sdk_crypt_sdk: Option<String>,
    pub bd_ticket_guard_client_web_domain: Option<String>,
    pub sid_guard: Option<String>,
    pub session_tlb_tag: Option<String>,
    pub sid_ucp_v1: Option<String>,
    pub ssid_ucp_v1: Option<String>,
    pub biz_trace_id: Option<String>,
    pub bd_ticket_guard_client_data: Option<String>,
    pub odin_tt: Option<String>,
    pub hm_lpvt_: Option<String>,
    pub ttwid: Option<String>,
    pub ecom_gray_shop_id: Option<String>,
    pub gfkadpd: Option<String>,
}

impl FeigeCookie {
    pub fn update_from_query(&mut self, query: &serde_json::Map<String, serde_json::Value>) {
        for (key, value) in query {
            if let Some(val) = value.as_str() {
                match key.as_str() {
                    "s_v_web_id" => self.s_v_web_id = Some(val.to_string()),
                    "passport_csrf_token" => self.passport_csrf_token = Some(val.to_string()),
                    "passport_csrf_token_default" => self.passport_csrf_token_default = Some(val.to_string()),
                    "x_web_secsdk_uid" | "x-web-secsdk-uid" => self.x_web_secsdk_uid = Some(val.to_string()),
                    "hm_lvt_" => self.hm_lvt_ = Some(val.to_string()),
                    "hmaccount" | "HMACCOUNT" => self.hmaccount = Some(val.to_string()),
                    "passport_mfa_token" => self.passport_mfa_token = Some(val.to_string()),
                    "uid_tt" => self.uid_tt = Some(val.to_string()),
                    "uid_tt_ss" => self.uid_tt_ss = Some(val.to_string()),
                    "sid_tt" => self.sid_tt = Some(val.to_string()),
                    "sessionid" => self.sessionid = Some(val.to_string()),
                    "sessionid_ss" => self.sessionid_ss = Some(val.to_string()),
                    "is_staff_user" => self.is_staff_user = Some(val.to_string()),
                    "has_biz_token" => self.has_biz_token = Some(val.to_string()),
                    "ucas_c0" => self.ucas_c0 = Some(val.to_string()),
                    "ucas_c0_ss" => self.ucas_c0_ss = Some(val.to_string()),
                    "phpsessid" | "PHPSESSID" => self.phpsessid = Some(val.to_string()),
                    "phpsessid_ss" | "PHPSESSID_SS" => self.phpsessid_ss = Some(val.to_string()),
                    "ecom_us_lt" => self.ecom_us_lt = Some(val.to_string()),
                    "ecom_us_lt_ss" => self.ecom_us_lt_ss = Some(val.to_string()),
                    "zsgw_business_data" => self.zsgw_business_data = Some(val.to_string()),
                    "source" => self.source = Some(val.to_string()),
                    "doudain_safety_did" => self.doudain_safety_did = Some(val.to_string()),
                    "csrf_session_id" => self.csrf_session_id = Some(val.to_string()),
                    "shop_id" | "SHOP_ID" => self.shop_id = Some(val.to_string()),
                    "pigeon_cid" | "PIGEON_CID" => self.pigeon_cid = Some(val.to_string()),
                    "ffa_goods_ewid" => self.ffa_goods_ewid = Some(val.to_string()),
                    "ffa_goods_seraph_did" => self.ffa_goods_seraph_did = Some(val.to_string()),
                    "security_mc_1_s_sdk_crypt_sdk" | "__security_mc_1_s_sdk_crypt_sdk" => self.security_mc_1_s_sdk_crypt_sdk = Some(val.to_string()),
                    "bd_ticket_guard_client_web_domain" => self.bd_ticket_guard_client_web_domain = Some(val.to_string()),
                    "sid_guard" => self.sid_guard = Some(val.to_string()),
                    "session_tlb_tag" => self.session_tlb_tag = Some(val.to_string()),
                    "sid_ucp_v1" => self.sid_ucp_v1 = Some(val.to_string()),
                    "ssid_ucp_v1" => self.ssid_ucp_v1 = Some(val.to_string()),
                    "biz_trace_id" => self.biz_trace_id = Some(val.to_string()),
                    "bd_ticket_guard_client_data" => self.bd_ticket_guard_client_data = Some(val.to_string()),
                    "odin_tt" => self.odin_tt = Some(val.to_string()),
                    "hm_lpvt_" => self.hm_lpvt_ = Some(val.to_string()),
                    "ttwid" => self.ttwid = Some(val.to_string()),
                    "ecom_gray_shop_id" => self.ecom_gray_shop_id = Some(val.to_string()),
                    "gfkadpd" => self.gfkadpd = Some(val.to_string()),
                    _ => {}
                }
            }
        }
    }
}


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

pub async fn feige_shop_info(webview_id: &str,webview: &Webview) -> Result<FeigeShopInfoResponse, Box<dyn std::error::Error>> {
    let cookie_str = webview
        .cookies()
        .map(|cookies| {
            cookies
                .iter()
                .map(|c| format!("{}={}", c.name(), c.value()))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default();

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



    let mut client = HttpClient::new();
    client.set_default_headers(headers);

    let res = client.get(&url).await?;

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
