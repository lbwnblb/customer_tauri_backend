use crate::utils::HttpClient;
use serde::{Deserialize, Serialize};

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


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

pub async fn feige_shop_info() -> Result<(), Box<dyn std::error::Error>> {
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

    let res = client.get(
        "https://fxg.jinritemai.com/center/qualification/shop/info\
     ?appid=1\
     &__token=dec6825093e192e7986d5c20fde6f1ac\
     &_bid=fxg_admin\
     &_lid=676892719333"
    ).await?;    println!("{}", res.body);

    Ok(())
}