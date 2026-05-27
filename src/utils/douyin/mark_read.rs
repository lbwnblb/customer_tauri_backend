use std::collections::HashMap;
use std::sync::atomic::Ordering::Relaxed;
use log::info;
use prost::Message;
use tauri::Webview;
use crate::utils::douyin::doudian_utils::{add_1_all, build_send_frame, SEQUENCE_ID};
use crate::utils::douyin::feige_resp::{IM_DEVICE_ID_MAP, IM_TOKEN_MAP, PIGEON_SIGN_MAP};
use crate::utils::douyin::protobuf::im_proto::{MarkConversationReadRequestBody, Request, RequestBody};

pub struct MarkReadParams {
    pub conversation_short_id: i64,
    pub read_message_index: i64,
    pub read_message_index_v2: i64,
    pub read_badge_count: i32,
    pub conv_unread_count: i64,
    pub total_unread_count: i64,
    pub sub_conversation_short_id: Option<i64>,
    pub security_conversation_id: Option<String>,
}

fn build_mark_read_request(webview: &Webview, params: &MarkReadParams) -> Request {
    let im_token = IM_TOKEN_MAP.lock().unwrap().get(webview.label()).cloned().unwrap_or_default();
    let device_id = IM_DEVICE_ID_MAP.lock().unwrap().get(webview.label()).cloned();
    let pigeon_sign = PIGEON_SIGN_MAP.lock().unwrap().get(webview.label()).cloned().unwrap_or_default();

    add_1_all();

    let mut request = Request::default();
    request.cmd = Some(2002);
    request.sequence_id = Some(SEQUENCE_ID.load(Relaxed) as i64);
    request.sdk_version = Some("0.0.0-fix-fix-inbox-array-202632312465".to_string());
    request.token = Some(im_token);
    request.refer = Some(3);
    request.inbox_type = Some(3);
    request.build_number = Some("0e70fed:fix/fix_inbox_array".to_string());
    request.device_id = device_id.clone();
    request.device_platform = Some("web".to_string());
    request.auth_type = Some(2);

    request.headers = HashMap::from([
        ("browser_online".to_string(), "true".to_string()),
        ("timezone_name".to_string(), "Asia/Shanghai".to_string()),
        ("referer".to_string(), "https://im.jinritemai.com/pc_seller/".to_string()),
        ("browser_version".to_string(), "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36".to_string()),
        ("cookie_enabled".to_string(), "true".to_string()),
        ("session_did".to_string(), device_id.clone().unwrap_or_default()),
        ("pigeon_source".to_string(), "web".to_string()),
        ("pigeon_sign".to_string(), pigeon_sign),
        ("app_name".to_string(), "im".to_string()),
        ("session_aid".to_string(), "1383".to_string()),
        ("PIGEON_BIZ_TYPE".to_string(), "2".to_string()),
        ("screen_width".to_string(), "1707".to_string()),
        ("user_agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36".to_string()),
        ("browser_language".to_string(), "zh-CN".to_string()),
        ("browser_platform".to_string(), "Win32".to_string()),
        ("priority_region".to_string(), "cn".to_string()),
        ("screen_height".to_string(), "1067".to_string()),
        ("browser_name".to_string(), "Mozilla".to_string()),
    ]);

    let mut mark_read_body = MarkConversationReadRequestBody::default();
    mark_read_body.conversation_id = Some("".to_string());
    mark_read_body.conversation_short_id = Some(params.conversation_short_id);
    mark_read_body.conversation_type = Some(11);
    mark_read_body.read_message_index = Some(params.read_message_index);
    mark_read_body.conv_unread_count = Some(params.conv_unread_count);
    mark_read_body.total_unread_count = Some(params.total_unread_count);
    mark_read_body.read_message_index_v2 = Some(params.read_message_index_v2);
    mark_read_body.read_badge_count = Some(params.read_badge_count);
    mark_read_body.sub_conversation_short_id = params.sub_conversation_short_id;
    mark_read_body.security_conversation_id = params.security_conversation_id.clone();

    let mut request_body = RequestBody::default();
    request_body.mark_conversation_read_body = Some(mark_read_body);
    request.body = Some(request_body);

    request
}

pub fn send_mark_read(webview: &Webview, params: MarkReadParams) {
    let payload = build_mark_read_request(webview, &params);
    //先测试打印
    // info!("build_mark_read_request:{:?}",payload);
    let payload_bytes = payload.encode_to_vec();

    let mut frame = build_send_frame(webview);
    frame.payload = Some(payload_bytes);
    let frame_bytes = frame.encode_to_vec();

    let js_bytes = format!("{:?}", frame_bytes);
    let js = format!(
        r#"(() => {{ const ws = window.__WS_INSTANCE__; if (ws && ws.readyState === 1) {{ ws.send(new Uint8Array({}).buffer); }} }})()"#,
        js_bytes
    );

    webview.eval(&js).ok();
    info!(
        "[IM] [MARK_READ] cid={}  idx={} idx_v2={}",
        params.conversation_short_id,
        params.read_message_index,
        params.read_message_index_v2,
    );
}
