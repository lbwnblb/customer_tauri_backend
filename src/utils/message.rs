use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(rename = "type")]
    pub msg_type: u8,
}

/// 角色常量
pub const ROLE_ASSISTANT: &str = "assistant";
pub const ROLE_USER: &str = "user";

/// 消息类型常量
pub const MSG_TYPE_TEXT: u8 = 0;    // 文本消息
pub const MSG_TYPE_IMAGE: u8 = 1;   // 图片消息
pub const MSG_TYPE_PRODUCT: u8 = 2; // 商品消息
pub const MSG_TYPE_VIDEO: u8 = 3;   // 视频消息
