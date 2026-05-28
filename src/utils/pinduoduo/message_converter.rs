use crate::utils::pinduoduo::pinduoduo_resp::ChatMessage;
use crate::utils::message::{Message, MSG_TYPE_IMAGE, MSG_TYPE_PRODUCT, MSG_TYPE_TEXT, MSG_TYPE_VIDEO, ROLE_ASSISTANT, ROLE_USER};

pub fn convert_messages(mut messages: Vec<ChatMessage>) -> Vec<Message> {
    messages.reverse();
    messages.into_iter().filter_map(|msg| {
        let role = match msg.from.as_ref().and_then(|f| f.role.as_deref()) {
            Some("mall_cs") => ROLE_ASSISTANT,
            Some("user") => ROLE_USER,
            _ => return None,
        }
        .to_string();

        // 过滤系统通知消息（type=31 等非对话消息）
        if matches!(msg.msg_type, Some(31)) {
            return None;
        }

        let (msg_type, content) = if msg.template_name.as_deref().filter(|s| !s.is_empty()) == Some("user_goods_card") {
            let goods_id = msg.info.as_ref()
                .and_then(|v| v.get("goodsID"))
                .and_then(|v| v.as_i64().map(|id| id.to_string())
                    .or_else(|| v.as_str().map(|s| s.to_string())))
                .or_else(|| msg.content.clone())
                .unwrap_or_default();
            (MSG_TYPE_PRODUCT, goods_id)
        } else {
            let mt = match msg.msg_type.unwrap_or(0) {
                1 => MSG_TYPE_IMAGE,
                2 => MSG_TYPE_PRODUCT,
                3 => MSG_TYPE_VIDEO,
                41 => MSG_TYPE_PRODUCT,
                _ => MSG_TYPE_TEXT,
            };
            let content = if mt == MSG_TYPE_PRODUCT {
                msg.info.as_ref()
                    .and_then(|v| v.get("goodsID"))
                    .and_then(|v| v.as_i64().map(|id| id.to_string())
                        .or_else(|| v.as_str().map(|s| s.to_string())))
                    .or_else(|| msg.content.clone())
                    .unwrap_or_default()
            } else {
                msg.content.unwrap_or_default()
            };
            (mt, content)
        };

        Some(Message { role, content, msg_type })
    })
    .collect()
}
