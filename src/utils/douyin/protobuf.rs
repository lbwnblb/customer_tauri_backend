use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::sync::{Arc, LazyLock, Mutex};
use std::sync::atomic::Ordering;
use log::{info, warn};
use prost::{DecodeError, Message};
use tauri::Webview;
use crate::utils::{app_data, get_by_conversation};
use crate::utils::douyin::doudian_utils::{build_get_conversation_info_list_v2_frame, build_get_conversation_info_list_v2_payload, build_send_frame, build_send_payload, SEQUENCE_ID, TICKET_NOTIFY_MAP};
use crate::utils::douyin::protobuf::im_proto::{GetConversationInfoListV2ResponseBody, MessageBody, NewMessageNotify, Request, Response, ResponseBody};

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
                                                                                                    if on_notify(message) {
                                                                                                        let conversation_short_id = message.conversation_short_id.clone();
                                                                                                        // message.
                                                                                                        //准备回复消息
                                                                                                        let send_payload = match build_send_payload(webview, conversation_short_id, message.conversation_type, format!("我在:{}",SEQUENCE_ID.load(Ordering::Relaxed)), message.sub_conversation_short_id, message.security_conversation_id.clone()).await {
                                                                                                            Ok(payload) => payload,
                                                                                                            Err(e) => {
                                                                                                                log::error!("build_send_payload failed: {}", e);
                                                                                                                return;
                                                                                                            }
                                                                                                        };
                                                                                                        info!("[IM] [RECV] MESSAGE_TYPE_50002  send_payload:{:?}",send_payload);
                                                                                                         // Request::se
                                                                                                        let send_payload = send_payload.encode_to_vec();
                                                                                                        let mut send_frame = build_send_frame(webview);
                                                                                                        info!("[IM] [RECV] MESSAGE_TYPE_50002  send_frame:{:?}",send_frame);
                                                                                                        send_frame.payload = Some(send_payload.to_vec());
                                                                                                        let frame_bytes = send_frame.encode_to_vec();
                                                                                                        // 转成 JS 能识别的字节数组字面量
                                                                                                        let js_bytes = format!("{:?}", frame_bytes); // "[1, 2, 3, ...]"

                                                                                                        let js = format!(
                                                                                                            r#"
                                                                                                                (() => {{
                                                                                                                    const ws = window.__WS_INSTANCE__;
                                                                                                                    if (ws && ws.readyState === 1) {{
                                                                                                                        ws.send(new Uint8Array({}).buffer);
                                                                                                                    }}
                                                                                                                }})()
                                                                                                                "#,
                                                                                                            js_bytes
                                                                                                        );
                                                                                                        webview.eval(&js).ok();

                                                                                                    }
                                                                                                    // info!("[IM] [RECV] conversation_short_id 消息通知:{:?}  index_cmd_message:{:?}",message.conversation_short_id,has_new_message_notify.cmd_message_index);
                                                                                                    // match get_by_conversation(webview, message.security_conversation_id.clone().unwrap().as_str(), message.conversation_short_id.unwrap()).await {
                                                                                                    //     Ok(conversation) => {
                                                                                                    //
                                                                                                    //     }
                                                                                                    //     Err(e) => {
                                                                                                    //         warn!("[IM] [RECV] 获取会话失败:{:?}",e);
                                                                                                    //     }
                                                                                                    // };
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
                                    // info!("[IM] [RECV] GET_CONVERSATION_INFO_LIST_V2_BODY:{:?}",response.body);
                                    match response.body {
                                        None => {}
                                        Some(body) => {
                                            match body.get_conversation_info_list_v2_body {
                                                None => {}
                                                Some(get_conversation_info_list_v2_body) => {
                                                    for conversationinfov2 in get_conversation_info_list_v2_body.conversation_info_list {

                                                        match conversationinfov2.conversation_short_id {
                                                            None => {}
                                                            Some(conversation_short_id) => {
                                                                let conversation_short_id_str = conversation_short_id.to_string();
                                                                if let Some(ref ticket) = conversationinfov2.ticket {
                                                                    TICKET_MAP.lock().unwrap().insert(conversation_short_id, ticket.clone());
                                                                    if let Some(notify) = TICKET_NOTIFY_MAP.lock().unwrap().get(&conversation_short_id) {
                                                                        notify.notify_one();
                                                                    }
                                                                }
                                                                let dir = app_data::response_cmd_610_get_conversation_info_list_v2_body();
                                                                let _ = fs::create_dir_all(&dir);
                                                                let file_path = format!("{}/{}.json", dir, conversation_short_id_str);
                                                                match serde_json::to_string_pretty(&conversationinfov2) {
                                                                    Ok(json) => {
                                                                        if let Err(e) = fs::write(&file_path, &json) {
                                                                            warn!("[IM] [RECV] 写入会话信息失败 {}: {}", file_path, e);
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        warn!("[IM] [RECV] 序列化会话信息失败: {}", e);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
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
fn on_notify(msg: &MessageBody) -> bool {
    // 带 LLM 增值字段的是第二条补充推送，直接丢弃
    if msg.ext.contains_key("llm_intent_label") {
        return false; // 忽略
    }
    true // 正常处理
}
pub fn new_send_request(cmd: i32, seqid: i64, body: im_proto::RequestBody) -> im_proto::Request {
    im_proto::Request {
        cmd: Some(cmd),
        sequence_id: Some(seqid),
        sdk_version: None,
        token: None,
        refer: None,
        inbox_type: None,
        build_number: None,
        body: Some(body),
        device_id: None,
        channel: None,
        device_platform: None,
        device_type: None,
        os_version: None,
        version_code: None,
        headers: HashMap::new(),
        config_id: None,
        token_info: None,
        auth_type: None,
        msg_trace: None,
        retry_count: None,
        biz: None,
        access: None,
    }
}

pub fn merge_request(existing: &mut im_proto::Request, incoming: im_proto::Request) {
    if incoming.cmd.is_some() { existing.cmd = incoming.cmd; }
    if incoming.sequence_id.is_some() { existing.sequence_id = incoming.sequence_id; }
    if incoming.sdk_version.is_some() { existing.sdk_version = incoming.sdk_version; }
    if incoming.token.is_some() { existing.token = incoming.token; }
    if incoming.refer.is_some() { existing.refer = incoming.refer; }
    if incoming.inbox_type.is_some() { existing.inbox_type = incoming.inbox_type; }
    if incoming.build_number.is_some() { existing.build_number = incoming.build_number; }
    if incoming.device_id.is_some() { existing.device_id = incoming.device_id; }
    if incoming.channel.is_some() { existing.channel = incoming.channel; }
    if incoming.device_platform.is_some() { existing.device_platform = incoming.device_platform; }
    if incoming.device_type.is_some() { existing.device_type = incoming.device_type; }
    if incoming.os_version.is_some() { existing.os_version = incoming.os_version; }
    if incoming.version_code.is_some() { existing.version_code = incoming.version_code; }
    if incoming.config_id.is_some() { existing.config_id = incoming.config_id; }
    if incoming.auth_type.is_some() { existing.auth_type = incoming.auth_type; }
    if incoming.retry_count.is_some() { existing.retry_count = incoming.retry_count; }
    if incoming.biz.is_some() { existing.biz = incoming.biz; }
    if incoming.access.is_some() { existing.access = incoming.access; }
    // 逐 key 合并 headers
    for (k, v) in incoming.headers {
        existing.headers.insert(k, v);
    }
    // 子消息精细合并
    if let Some(incoming_body) = incoming.body {
        merge_request_body(existing.body.get_or_insert_with(Default::default), incoming_body);
    }
    if let Some(incoming_token_info) = incoming.token_info {
        let t = existing.token_info.get_or_insert_with(Default::default);
        if incoming_token_info.mark_id.is_some() { t.mark_id = incoming_token_info.mark_id; }
        if incoming_token_info.r#type.is_some() { t.r#type = incoming_token_info.r#type; }
        if incoming_token_info.app_id.is_some() { t.app_id = incoming_token_info.app_id; }
        if incoming_token_info.user_id.is_some() { t.user_id = incoming_token_info.user_id; }
        if incoming_token_info.timestamp.is_some() { t.timestamp = incoming_token_info.timestamp; }
    }
    if let Some(incoming_trace) = incoming.msg_trace {
        let t = existing.msg_trace.get_or_insert_with(Default::default);
        if incoming_trace.path.is_some() { t.path = incoming_trace.path; }
        for (k, v) in incoming_trace.metrics {
            t.metrics.insert(k, v);
        }
    }
}

fn merge_request_body(existing: &mut im_proto::RequestBody, incoming: im_proto::RequestBody) {
    if let Some(v) = incoming.send_message_body {
        merge_send_message_body(existing.send_message_body.get_or_insert_with(Default::default), v);
    }
}

fn merge_send_message_body(existing: &mut im_proto::SendMessageRequestBody, incoming: im_proto::SendMessageRequestBody) {
    if incoming.conversation_id.is_some() { existing.conversation_id = incoming.conversation_id; }
    if incoming.conversation_type.is_some() { existing.conversation_type = incoming.conversation_type; }
    if incoming.conversation_short_id.is_some() { existing.conversation_short_id = incoming.conversation_short_id; }
    if incoming.content.is_some() { existing.content = incoming.content; }
    if incoming.message_type.is_some() { existing.message_type = incoming.message_type; }
    if incoming.ticket.is_some() { existing.ticket = incoming.ticket; }
    if incoming.client_message_id.is_some() { existing.client_message_id = incoming.client_message_id; }
    if incoming.ignore_badge_count.is_some() { existing.ignore_badge_count = incoming.ignore_badge_count; }
    if incoming.unuse_field1.is_some() { existing.unuse_field1 = incoming.unuse_field1; }
    if incoming.sub_conversation_short_id.is_some() { existing.sub_conversation_short_id = incoming.sub_conversation_short_id; }
    if incoming.security_conversation_id.is_some() { existing.security_conversation_id = incoming.security_conversation_id; }
    if incoming.security_to_user_id.is_some() { existing.security_to_user_id = incoming.security_to_user_id; }
    if !incoming.mentioned_users.is_empty() { existing.mentioned_users = incoming.mentioned_users; }
    if !incoming.security_mentioned_users.is_empty() { existing.security_mentioned_users = incoming.security_mentioned_users; }
    if incoming.ref_msg_info.is_some() { existing.ref_msg_info = incoming.ref_msg_info; }
    // 逐 key 合并 HashMap 字段
    for (k, v) in incoming.ext {
        existing.ext.insert(k, v);
    }
    for (k, v) in incoming.client_ext {
        existing.client_ext.insert(k, v);
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
                                    info!("[IM] [SEND] GET_CONVERSATION_INFO_LIST_V2_BODY seqid={} logid={} service={} method={} headers={:?} payload_encoding={:?} payload_type={:?}", frame.seqid, frame.logid, frame.service, frame.method, frame.headers, frame.payload_encoding, frame.payload_type);
                                    info!("[IM] [SEND] GET_CONVERSATION_INFO_LIST_V2_BODY request={:?}", request);
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