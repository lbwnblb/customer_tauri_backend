use crate::utils::douyin::protobuf::im_proto;
use crate::utils::message::{Message, MSG_TYPE_IMAGE, MSG_TYPE_PRODUCT, MSG_TYPE_TEXT, MSG_TYPE_VIDEO, ROLE_ASSISTANT, ROLE_USER};

pub fn convert_messages(messages: Vec<im_proto::MessageBody>) -> Vec<Message> {
    messages.into_iter().filter_map(|msg| {
        let role = match msg.ext.get("s:sender_biz_role")?.as_str() {
            "CurrentServer" => ROLE_ASSISTANT,
            "Buyer" => ROLE_USER,
            _ => return None,
        }
        .to_string();
        let msg_type_str = msg.ext.get("type").map(|s| s.as_str()).unwrap_or("text");

        let (msg_type, content) = match msg_type_str {
            "text" => (MSG_TYPE_TEXT, msg.content.unwrap_or_default()),
            "file_image" => {
                let url = msg.ext.get("imageUrl").cloned().unwrap_or_default();
                (MSG_TYPE_IMAGE, url)
            }
            "template_card" => {
                let goods_id = msg.ext.get("goods_id").cloned().unwrap_or_default();
                (MSG_TYPE_PRODUCT, goods_id)
            }
            "file_video" | "video" => {
                let url = msg.ext.get("videoUrl")
                    .or_else(|| msg.ext.get("url"))
                    .cloned()
                    .unwrap_or_default();
                (MSG_TYPE_VIDEO, url)
            }
            _ => (MSG_TYPE_TEXT, msg.content.unwrap_or_default()),
        };

        Some(Message { role, content, msg_type })
    })
    .collect()
}
