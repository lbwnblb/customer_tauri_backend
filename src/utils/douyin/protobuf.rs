use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use log::{info, warn};
use prost::Message;
use tauri::Webview;
use crate::utils::douyin::doudian_utils::{TICKET_NOTIFY_MAP};
use crate::utils::douyin::protobuf::im_proto::{MessageBody};

pub mod im_proto {
    include!(concat!(env!("OUT_DIR"), "/dy_im_proto.rs"));
}

pub static SEND_REQUEST_MAP: LazyLock<Mutex<HashMap<String, im_proto::Request>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static TICKET_MAP: LazyLock<Mutex<HashMap<i64, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));




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
    pub const GET_CONVERSATION_INFO_LIST_V2_BODY: i32 = 610;

    /// 标记已读
    pub const MARK_CONVERSATION_READ_V3: i32 = 2002;

    /// 拉取历史消息
    pub const GET_HISTORY_MESSAGE: i32 = 200;

    /// 拉取会话列表
    pub const GET_CONVERSATION_LIST: i32 = 203;

    /// 拉取消息（通用）
    pub const GET_MESSAGE: i32 = 301;
    pub const GET_TICKET: i32 = 2005;

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

pub mod ext_type{
    pub const ALLOCATED_SERVICE: &str = "allocated_service";
}


pub mod message_type {
    pub const MESSAGE_TYPE_1000: i32 = 1000;
    pub const MESSAGE_TYPE_50002:i32 = 50002;
}

/// 打印消息中所有可能记录"用户从哪个商品进来"的字段，用于探查 ext key。
/// 在 cmd=500 新消息通知里对每条消息调用一次。
fn dump_entry_product(label: &str, message: &im_proto::MessageBody) {
    let short_id  = message.conversation_short_id.unwrap_or(0);
    let msg_type  = message.message_type.unwrap_or(0);
    let role      = message.ext.get("s:sender_biz_role").map(|s| s.as_str()).unwrap_or("?");
    let dy_type   = message.ext.get("type").map(|s| s.as_str()).unwrap_or("?");
    let content   = message.content.as_deref().unwrap_or("");

    // 已知可能携带来源商品的 key
    const PRODUCT_KEYS: &[&str] = &[
        "source_goods_id",
        "goods_id",
        "product_id",
        "s:source_goods_id",
        "source_type",
        "origin_type",
        "source_page",
        "enter_from",
        "promotion_id",
        "item_id",
        "sku_id",
    ];

    let found: Vec<String> = PRODUCT_KEYS
        .iter()
        .filter_map(|k| message.ext.get(*k).map(|v| format!("{}={}", k, v)))
        .collect();

    info!(
        "[ENTRY_PRODUCT] webview={} cid={} msg_type={} role={} dy_type={} | product_keys=[{}] | content={:?} | full_ext={:?}",
        label, short_id, msg_type, role, dy_type,
        found.join(", "),
        &content[..content.len().min(200)],
        message.ext
    );
}


pub async fn feige_im_recv(webview: &Webview, bytes: &[u8]) {
    let webview_id = webview.label().to_string();
    let  frame = match im_proto::Frame::decode(bytes) {
        Ok(f) => f,
        Err(e) => {
            info!("[IM] [RECV] Frame 解码失败: {}", e);
            return;
        }
    };
    match frame.payload {
        None => {}
        Some(payload) => {
            match im_proto::Response::decode(payload.as_slice()) {
                Ok(response) => {
                    // info!("[IM] [RECV] Response:{:?}",response);
                    match response.cmd {
                        None => {}
                        Some(cmd) => {
                            match cmd {
                                cmd_status::NEW_MSG_NOTIFY=>{
                                    //新消息通知
                                    match response.body {
                                        None => {
                                            warn!("[IM] [RECV] Response.Body 为空")
                                        }
                                        Some(ref body) => {
                                            match body.has_new_message_notify {
                                                None => {
                                                    warn!("[IM] [RECV] Response.Body.NewMessageNotify 为空")
                                                }
                                                Some(ref has_new_message_notify) => {
                                                    // has_new_message_notify
                                                    // info!("[IM] [RECV] response新消息通知:{:?}",response);
                                                    match has_new_message_notify.message {
                                                        None => {}
                                                        Some(ref message) => {
                                                            // 诊断：打印每条消息的入口商品相关字段
                                                            // dump_entry_product(&webview_id, message);
                                                            match message.message_type {
                                                                None => {}
                                                                Some(ref message_type) => {
                                                                    let ext = &message.ext;
                                                                    match ext.get("s:sender_biz_role") {
                                                                        None => {
                                                                            // warn!("[IM] [RECV] message.ext.s:sender_biz_role 为空{:?}",has_new_message_notify);
                                                                        }
                                                                        Some(sender_biz_role) => {
                                                                            match sender_biz_role.as_str() {
                                                                                "Buyer"=>{
                                                                                    match message.message_type {
                                                                                        None => {}
                                                                                        Some( message_type) => {
                                                                                            match message_type {
                                                                                                message_type::MESSAGE_TYPE_1000=>{
                                                                                                    match ext.get("type") {
                                                                                                        None => {}
                                                                                                        Some(dy_type) => {
                                                                                                            if !dy_type.contains(ext_type::ALLOCATED_SERVICE) {
                                                                                                                // info!("用户消息,{:?}",message.content);
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                },
                                                                                                message_type::MESSAGE_TYPE_50002 => {
                                                                                                    super::auto_reply::record_shark_product_id(webview, message);
                                                                                                    if super::auto_reply::on_notify(message) {
                                                                                                        super::auto_reply::schedule_auto_reply(webview.clone(), message).await;
                                                                                                        // info!("MESSAGE_TYPE_50002 message:{:?}",message);
                                                                                                    }
                                                                                                }
                                                                                                _ => {}
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                                &_ => {}
                                                                            }
                                                                        }
                                                                    };

                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                cmd_status::GET_CONVERSATION_INFO_LIST_V2_BODY=>{
                                    if let Some(ref body) = response.body {
                                        if let Some(ref conv_list) = body.get_conversation_info_list_v2_body {
                                            let mut ticket_map = TICKET_MAP.lock().unwrap();
                                            let notify_map = TICKET_NOTIFY_MAP.lock().unwrap();
                                            for conv in &conv_list.conversation_info_list {
                                                if let (Some(short_id), Some(ref ticket)) = (conv.conversation_short_id, &conv.ticket) {
                                                    info!("[IM] [RECV] GET_CONVERSATION_INFO_LIST_V2 ticket cached: cid={} ticket={}", short_id, ticket);
                                                    ticket_map.insert(short_id, ticket.clone());
                                                    if let Some(notify) = notify_map.get(&short_id) {
                                                        notify.notify_one();
                                                    }
                                                }
                                            }
                                        } else {
                                            warn!("[IM] [RECV] GET_CONVERSATION_INFO_LIST_V2 body 为空");
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(e) => {
                    info!("[IM] [RECV] Response 解码失败: {}", e);
                }
            };
        }
    }

}
pub async fn feige_im_send(webview: &Webview, bytes: &[u8]) {
    let webview_id = webview.label().to_string();
    let frame = match im_proto::Frame::decode(bytes) {
        Ok(f) => f,
        Err(e) => {
            info!("[IM] [SEND] Frame 解码失败: {}", e);
            return;
        }
    };
    match frame.payload {
        None => {}
        Some(payload) => {
            match im_proto::Request::decode(payload.as_slice()) {
                Ok(request) => {
                    match request.cmd {
                        None => {}
                        Some(cmd) => {
                            match cmd {
                                cmd_status::GET_CONVERSATION_INFO_LIST_V2_BODY => {
                                    // info!("[IM] [SEND] GET_CONVERSATION_INFO_LIST_V2_BODY seqid={} logid={} service={} method={} headers={:?} payload_encoding={:?} payload_type={:?}", frame.seqid, frame.logid, frame.service, frame.method, frame.headers, frame.payload_encoding, frame.payload_type);
                                    // info!("[IM] [SEND] GET_CONVERSATION_INFO_LIST_V2_BODY request={:?}", request);
                                }
                                _=> {}
                            }
                        }
                    }
                }
                Err(e) => {
                    info!("[IM] [SEND] Request 解码失败: {}", e);
                }
            };
        }
    }
}

pub fn parse_response(bytes: &[u8]) -> Result<im_proto::Response, prost::DecodeError> {
    im_proto::Response::decode(bytes)
}