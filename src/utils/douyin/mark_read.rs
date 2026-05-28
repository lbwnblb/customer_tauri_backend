use std::collections::HashMap;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{LazyLock, Mutex};
use log::info;
use prost::Message;
use tauri::Webview;
use crate::utils::douyin::doudian_utils::{add_1_all, build_send_frame, SEQUENCE_ID};
use crate::utils::douyin::feige_resp::{IM_DEVICE_ID_MAP, IM_TOKEN_MAP, PIGEON_SIGN_MAP};
use crate::utils::douyin::protobuf::im_proto::{Frame, MarkConversationReadRequestBody, MessageBody, NewMessageNotify, Request, RequestBody, Response, ResponseBody};

/// 最近一次新消息通知中的会话 badge count，keyed by conversation_short_id
/// 用于 mark_read 时传递正确的 read_badge_count（JS SDK 以 badgeCount - readBadgeCount 计算未读数）
pub static CONV_BADGE_COUNT_MAP: LazyLock<Mutex<HashMap<i64, i32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
    mark_read_body.conversation_type = Some(10);
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
    let payload_bytes = payload.encode_to_vec();

    let mut frame = build_send_frame(webview);
    frame.payload = Some(payload_bytes);
    let frame_bytes = frame.encode_to_vec();

    let js_bytes = format!("{:?}", frame_bytes);
    let ws_js = format!(
        r#"(() => {{ const ws = window.__WS_INSTANCE__; if (ws && ws.readyState === 1) {{ ws.send(new Uint8Array({}).buffer); }} }})()"#,
        js_bytes
    );
    webview.eval(&ws_js).ok();

    // JS SDK 在发送 WS 前会先本地更新状态（乐观更新），我们绕过了这一步，
    // 所以额外扫描 window.Mera 上的 SDK 实例，尝试调用其 markConversationRead
    // 以更新 readIndex/readBadgeCount，消除左侧红点。
    let sec_cid = params.security_conversation_id.as_deref().unwrap_or("").replace('\'', "\\'");
    let local_update_js = format!(
        r#"(() => {{
  try {{
    const secCid = '{sec_cid}';
    const readIdx = {read_idx};
    const readIdxV2 = {read_idx_v2};
    const badgeCnt = {badge_cnt};

    // Call markConversationRead on a BytedIM/RustIM core instance
    const callMark = (core) => {{
      try {{ core.markConversationRead(3, secCid, readIdx, {{ readIndexV2: readIdxV2, readBadgeCount: badgeCnt }}); }} catch(_) {{}}
    }};

    // Try calling markConversationRead on a candidate object, checking common paths
    const tryMark = (inst) => {{
      if (!inst || typeof inst !== 'object') return false;
      // Direct: inst is the IM core (BytedIM/RustIM)
      if (typeof inst.markConversationRead === 'function') {{ callMark(inst); return true; }}
      // PigeonApp stores coreIM as ._coreIM or .coreIM (getter)
      for (const key of ['_coreIM', 'coreIM']) {{
        const core = inst[key];
        if (core && typeof core.markConversationRead === 'function') {{ callMark(core); return true; }}
      }}
      // Deeper: inst.ctx.app._coreIM (Mera service pattern: r.ctx.app.coreIM)
      const app = inst.ctx && inst.ctx.app;
      if (app) {{
        for (const key of ['_coreIM', 'coreIM']) {{
          const core = app[key];
          if (core && typeof core.markConversationRead === 'function') {{ callMark(core); return true; }}
        }}
        if (typeof app.markConversationRead === 'function') {{ callMark(app); return true; }}
      }}
      return false;
    }};

    // 1. Scan window.Mera (Mera framework registry)
    const mera = window.Mera;
    if (mera) {{
      const maps = [mera, mera.instanceMap, mera.matchMap].filter(Boolean);
      for (const map of maps) {{
        for (const k of Object.keys(map)) {{
          if (tryMark(map[k])) return;
        }}
      }}
    }}

    // 2. Scan top-level window properties for BytedIM/RustIM instances
    for (const k of Object.keys(window)) {{
      try {{
        const v = window[k];
        if (!v || typeof v !== 'object') continue;
        // Direct match
        if (typeof v.markConversationRead === 'function') {{ tryMark(v); return; }}
        // Via _coreIM / coreIM stored on a window-level object
        for (const ck of ['_coreIM', 'coreIM']) {{
          const core = v[ck];
          if (core && typeof core.markConversationRead === 'function') {{ callMark(core); return; }}
        }}
      }} catch(_) {{}}
    }}
  }} catch(_) {{}}
}})()"#,
        sec_cid = sec_cid,
        read_idx = params.read_message_index,
        read_idx_v2 = params.read_message_index_v2,
        badge_cnt = params.read_badge_count,
    );
    webview.eval(&local_update_js).ok();

    // 模拟服务端推送：注入一条 cmd=500 / message_type=50001 / command_type=1 的假消息，
    // 让 JS SDK 经由正常的 handleMarkConversationRead 路径更新本地状态、消除红点。
    if let Some(ref sec_cid) = params.security_conversation_id {
        inject_fake_mark_read_push(
            webview,
            params.conversation_short_id,
            sec_cid,
            params.read_message_index,
            params.read_message_index_v2,
            params.read_badge_count,
            params.sub_conversation_short_id,
        );
    }

    info!(
        "[IM] [MARK_READ] cid={}  idx={} idx_v2={} badge={}",
        params.conversation_short_id,
        params.read_message_index,
        params.read_message_index_v2,
        params.read_badge_count,
    );
}

/// 向 WebSocket 的 onmessage 注入一条伪造的服务端推送帧，
/// 内容与服务端发送的已读命令推送完全一致（cmd=500, message_type=50001, command_type=1），
/// 以触发 JS SDK 内部的 handleMarkConversationRead，正确更新本地 readIndex/readBadgeCount，
/// 消除会话列表左侧红点。
fn inject_fake_mark_read_push(
    webview: &Webview,
    conversation_short_id: i64,
    security_conversation_id: &str,
    read_message_index: i64,
    read_message_index_v2: i64,
    read_badge_count: i32,
    sub_conversation_short_id: Option<i64>,
) {
    // JSON content 字段名与服务端推送完全一致（经官方抓包确认）
    let sec = security_conversation_id.replace('"', "\\\"");
    let content = format!(
        concat!(
            r#"{{"mute_read_badge_count_info":[],"read_badge_count":{b},"#,
            r#""read_index":{ri},"security_conversation_id":"{s}","#,
            r#""command_type":1,"inbox_type":3,"#,
            r#""conv_unread_union":{{"ConvUnreadInfo":{{"0":{{"badge_count":-1,"read_badge_count":-1}},"#,
            r#""1":{{"badge_count":-1,"read_badge_count":-1}}}}}},"#,
            r#""conversation_type":10,"read_index_v2":{ri2}}}"#,
        ),
        b   = read_badge_count,
        ri  = read_message_index,
        s   = sec,
        ri2 = read_message_index_v2,
    );

    let mut ext: HashMap<String, String> = HashMap::new();
    ext.insert("s:command_type".into(), "1".into());
    ext.insert("s:need_bcp".into(), "1".into());
    ext.insert("s:base_scene".into(), "default".into());
    ext.insert("s:new_mark_read_push".into(), "true".into());
    ext.insert("s:msg_priority".into(), "-1".into());

    let mut msg = MessageBody::default();
    msg.conversation_short_id = Some(conversation_short_id);
    msg.message_type = Some(50001); // MESSAGE_TYPE_COMMAND
    msg.content = Some(content);
    msg.ext = ext;
    msg.security_conversation_id = Some(security_conversation_id.to_string());
    msg.index_in_conversation_v2 = Some(read_message_index_v2);
    if let Some(sub) = sub_conversation_short_id {
        msg.sub_conversation_short_id = Some(sub);
    }

    let mut notify = NewMessageNotify::default();
    notify.conversation_type = Some(10); // BC_PARENT_CONVERSATION
    notify.notify_type = Some(1);        // PER_USER
    notify.message = Some(msg);
    notify.badge_count = Some(read_badge_count);
    notify.security_conversation_id = Some(security_conversation_id.to_string());
    if let Some(sub) = sub_conversation_short_id {
        notify.sub_conversation_short_id = Some(sub);
    }

    let mut resp_body = ResponseBody::default();
    resp_body.has_new_message_notify = Some(notify);

    let mut resp = Response::default();
    resp.cmd = Some(500);         // NEW_MSG_NOTIFY
    resp.status_code = Some(0);
    resp.inbox_type = Some(3);
    resp.body = Some(resp_body);

    let resp_bytes = resp.encode_to_vec();

    let mut frame = Frame::default();
    frame.seqid = 1;
    frame.logid = 1;
    frame.service = 10001;
    frame.method = 1;
    frame.payload = Some(resp_bytes);

    let frame_bytes = frame.encode_to_vec();
    let js_bytes = format!("{:?}", frame_bytes);
    let js = format!(
        r#"(() => {{ const ws = window.__WS_INSTANCE__; if (ws) {{ ws.dispatchEvent(new MessageEvent('message', {{ data: new Uint8Array({}).buffer }})); }} }})()"#,
        js_bytes
    );
    webview.eval(&js).ok();
    info!("[IM] [MARK_READ] injected fake cmd=500 push for cid={}", conversation_short_id);
}
