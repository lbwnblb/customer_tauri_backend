use super::constants::{PLATFORM_DOUYIN, PLATFORM_PINDUODUO, PLATFORM_UNKNOWN, PLATFORM_DOUYIN_CN, PLATFORM_PINDUODUO_CN, PLATFORM_UNKNOWN_CN};

pub fn is_douyin_platform(id: &str) -> bool {
    let parts: Vec<&str> = id.split('_').collect();
    parts.get(1) == Some(&PLATFORM_DOUYIN)
}

pub fn is_pinduoduo_platform(id: &str) -> bool {
    let parts: Vec<&str> = id.split('_').collect();
    parts.get(1) == Some(&PLATFORM_PINDUODUO)
}

pub fn get_platform_from_id(id: &str) -> String {
    let parts: Vec<&str> = id.split('_').collect();
    match parts.get(1) {
        Some(&PLATFORM_DOUYIN) => PLATFORM_DOUYIN_CN.to_string(),
        Some(&PLATFORM_PINDUODUO) => PLATFORM_PINDUODUO_CN.to_string(),
        _ => PLATFORM_UNKNOWN_CN.to_string(),
    }
}

/// 从 webview id 中提取平台英文标识符（"douyin" / "pinduoduo"），未知平台返回 None。
pub fn platform_id_from_webview_id(webview_id: &str) -> Option<&'static str> {
    let part = webview_id.split('_').nth(1)?;
    match part {
        PLATFORM_DOUYIN => Some(PLATFORM_DOUYIN),
        PLATFORM_PINDUODUO => Some(PLATFORM_PINDUODUO),
        _ => None,
    }
}
