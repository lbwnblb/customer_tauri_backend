use log::info;
use prost::Message;
use tauri::Webview;

pub mod im_proto {
    include!(concat!(env!("OUT_DIR"), "/dy_im_proto.rs"));
}


pub mod cmd_status {
    // ==================== 客户端 → 服务端 ====================

    /// 真正发消息（文本/图片/卡片等）
    pub const SEND_MESSAGE: i32 = 100;

    /// P2P 发消息
    pub const SEND_MESSAGE_P2P: i32 = 2031;

    /// 用户行为上报
    pub const SEND_USER_ACTION: i32 = 410;

    /// "正在输入..."
    pub const SEND_INPUT_STATUS: i32 = 411;

    /// 收到推送后的 ACK 回执
    pub const CLIENT_ACK: i32 = 2010;

    /// 标记已读
    pub const MARK_CONVERSATION_READ_V3: i32 = 2002;

    /// 拉取历史消息
    pub const GET_HISTORY_MESSAGE: i32 = 200;

    /// 拉取会话列表
    pub const GET_CONVERSATION_LIST: i32 = 203;

    /// 拉取消息（通用）
    pub const GET_MESSAGE: i32 = 301;

    /// 拉取消息扩展
    pub const GET_MESSAGE_EXT: i32 = 2035;

    /// 拉取消息补充
    pub const GET_MESSAGE_SUPPLEMENT: i32 = 2043;

    /// 删除消息
    pub const DELETE_MESSAGE: i32 = 701;

    /// 撤回消息
    pub const RECALL_MESSAGE: i32 = 702;

    // ==================== 服务端 → 客户端（推送） ====================

    /// 新消息通知——核心，顾客发来的消息走这里
    pub const NEW_MSG_NOTIFY: i32 = 500;

    /// P2P 新消息推送
    pub const NEW_P2P_MSG_NOTIFY: i32 = 504;
}

pub mod sender_role {
    /// 顾客
    pub const CUSTOMER: &str = "1";

    /// 客服（自己）
    pub const AGENT: &str = "2";

    /// 系统 / 机器人
    pub const SYSTEM: &str = "3";
}

pub async fn feige_im_proto(webview: &Webview, bytes: &[u8]) {
    let frame = match im_proto::Frame::decode(bytes) {
        Ok(f) => f,
        Err(e) => {
            info!("[IM] Frame 解码失败: {}", e);
            return;
        }
    };

    let payload = match &frame.payload {
        Some(p) => p,
        None => return,
    };

    let response = match im_proto::Response::decode(payload.as_slice()) {
        Ok(r) => r,
        Err(e) => {
            info!("[IM] Response 解码失败: {}", e);
            return;
        }
    };

    if response.cmd != Some(500) {
        return;
    }

    let body = match &response.body {
        Some(b) => b,
        None => return,
    };

    let notify = match &body.has_new_message_notify {
        Some(n) => n,
        None => return,
    };

    let msg = match &notify.message {
        Some(m) => m,
        None => return,
    };

    info!(
        "[IM] cmd=500 conversation_id={:?} sender={:?} security_conversation_id={:?} security_sender={:?} message_type={:?} content={}",
        msg.conversation_id,
        msg.sender,
        msg.security_conversation_id,
        msg.security_sender,
        msg.message_type,
        msg.content.as_deref().unwrap_or(""),
    );

    if msg.content.as_deref().unwrap_or("").is_empty() {
        let security_conv_id = msg.security_conversation_id.clone().unwrap_or_default();
        let conv_short_id = msg.conversation_short_id.unwrap_or(0);
        let conv_id = msg.conversation_id.clone().unwrap_or_default();
        let anchor_index = msg.index_in_conversation.unwrap_or(0);

        info!("[IM] cmd=500 content 为空，拉取消息...");

        match crate::utils::feige_resp::get_by_conversation(
            webview,
            &conv_id,
            &security_conv_id,
            conv_short_id,
            anchor_index,
        )
        .await
        {
            Ok(messages) => {
                info!("[IM] 拉取到 {} 条消息", messages.len());
            }
            Err(e) => {
                info!("[IM] 拉取消息失败: {}", e);
            }
        }
    }
}

pub fn parse_response(bytes: &[u8]) -> Result<im_proto::Response, prost::DecodeError> {
    im_proto::Response::decode(bytes)
}