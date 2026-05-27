use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicU64};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::CursorIcon::Default;
use tauri::Webview;
use tauri::webview::Cookie;
use uuid::Uuid;
use prost::Message;
use tokio::sync::Notify;
use tokio::time::{timeout, Duration};
use crate::commands::shop_callback::FEIGE_MANAGEMENT_COOKIE;
use crate::utils::douyin::feige_resp::{feige_shop_info, IM_DEVICE_ID_MAP, IM_TOKEN_MAP, PIGEON_SIGN_MAP};
use crate::utils::douyin::protobuf::im_proto::{Frame, GetConversationInfoV2RequestBody, GetConversationInfoListV2RequestBody, MessageBody, Request, RequestBody, SendMessageRequestBody};
use crate::utils::douyin::protobuf::im_proto::frame::ExtendedEntry;
use crate::utils::douyin::protobuf::TICKET_MAP;
use crate::utils::timestamp_millis;
use crate::utils::uuid::uuid_with_hyphen;

pub static SEQUENCE_ID: AtomicU64 = AtomicU64::new(0);
pub static LOG_ID: AtomicU64 = AtomicU64::new(0);

pub static TICKET_NOTIFY_MAP: LazyLock<Mutex<HashMap<i64, Arc<Notify>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));


pub fn add_1_all(){
    SEQUENCE_ID.fetch_add(1, Relaxed);
    LOG_ID.fetch_add(1, Relaxed);
}
pub fn build_send_frame(webview: &Webview)-> Frame {
    let pigeon_sign = get_pigeon_sign(&webview).unwrap();
    // 用 cloned() 拿到所有权
    let mut frame = Frame::default();
    add_1_all();
    frame.seqid = SEQUENCE_ID.load(Relaxed);
    frame.logid = LOG_ID.load(Relaxed);
    frame.service = 10001;
    frame.method = 1;


    frame.headers = vec![ExtendedEntry{key: "pigeon_source".to_string(), value: "web".to_string()}
                         ,ExtendedEntry{key: "PIGEON_BIZ_TYPE".to_string(), value: "2".to_string()}
                         ,ExtendedEntry{key: "pigeon_sign".to_string(), value: pigeon_sign}];
    frame.payload_type = Some("pb".to_string());

    frame
}

fn get_pigeon_sign(webview: &Webview) -> Option<String> {
    let guard = PIGEON_SIGN_MAP.lock().unwrap();
    let pigeon_sign = guard.get(webview.label()).cloned(); // 用 cloned() 拿到所有权
    pigeon_sign
}


pub  fn get_current_shop_id(webview: &Webview)-> Result<String,()>{
    let mutex_guard = FEIGE_MANAGEMENT_COOKIE.lock().unwrap();
    match mutex_guard.get(webview.label()) {
        None => {}
        Some(cookies) => {
            for cookie in cookies {
                if cookie.name().eq("SHOP_ID") {
                     return Ok(cookie.value().to_string());
                }
            }
        }
    }
    Err(())
}

pub async fn build_send_payload(webview: &Webview,conversation_short_id:Option<i64>,content:String,sub_conversation_short_id:Option<i64>,security_conversation_id:Option<String>) ->Result<Request, String>{
    let ticket = match conversation_short_id.and_then(|id| TICKET_MAP.lock().unwrap().get(&id).cloned()) {
        Some(t) => t,
        None => {
            let cid = conversation_short_id.ok_or("conversation_short_id is None")?;
            let sec_cid = security_conversation_id.clone().ok_or("security_conversation_id is None")?;
            log::info!("ticket not in cache, cid={}, sec_cid={}", cid, sec_cid);

            let notify = Arc::new(Notify::new());
            TICKET_NOTIFY_MAP.lock().unwrap().insert(cid, notify.clone());
            log::info!("notify registered for cid={}", cid);

            let payload = build_get_conversation_info_list_v2_payload(webview, cid, sec_cid)?;
            let payload_bytes = payload.encode_to_vec();
            log::info!("payload built and encoded, len={}", payload_bytes.len());

            let mut frame = build_get_conversation_info_list_v2_frame(webview);
            frame.payload = Some(payload_bytes);
            let frame_bytes = frame.encode_to_vec();
            log::info!("frame encoded, len={}", frame_bytes.len());

            let js_bytes = format!("{:?}", frame_bytes);
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
            log::info!("ws send JS evaluated for cid={}", cid);

            match timeout(Duration::from_secs(10), notify.notified()).await {
                Ok(_) => {
                    log::info!("ticket notify received for cid={}", cid);
                    TICKET_NOTIFY_MAP.lock().unwrap().remove(&cid);
                    TICKET_MAP.lock().unwrap().get(&cid).cloned()
                        .ok_or("ticket not found after notification".to_string())?
                }
                Err(_) => {
                    log::warn!("ticket wait timeout for cid={}", cid);
                    TICKET_NOTIFY_MAP.lock().unwrap().remove(&cid);
                    return Err("timeout waiting for ticket".to_string());
                }
            }
        }
    };
    let shop_id = get_current_shop_id(webview).unwrap();
    let im_token = get_feige_im_token(webview);
    let mut request = Request::default();
    request.cmd = Some(100);
    request.sequence_id = Some(SEQUENCE_ID.load(Relaxed) as i64);
    request.sdk_version = Some("0.0.0-fix-fix-inbox-array-202632312465".to_string());
    request.token = Some(im_token);
    request.refer = Some(3);
    request.inbox_type = Some(3);
    request.build_number = Some("0e70fed:fix/fix_inbox_array".to_string());
    request.device_id =  get_device_id(webview);

    let  headers = HashMap::from([
        ("browser_online".to_string(), "true".to_string()),
        ("timezone_name".to_string(), "Asia/Shanghai".to_string()),
        ("referer".to_string(), "".to_string()),
        ("browser_version".to_string(), "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".to_string()),
        ("cookie_enabled".to_string(), "true".to_string()),
        ("session_did".to_string(), request.device_id.clone().unwrap()),
        ("pigeon_source".to_string(), "web".to_string()),
        ("pigeon_sign".to_string(), get_pigeon_sign(webview).unwrap()),
        ("app_name".to_string(), "im".to_string()),
        ("session_aid".to_string(), "1383".to_string()),
        ("PIGEON_BIZ_TYPE".to_string(), "2".to_string()),
        ("screen_width".to_string(), "1707".to_string()),
        ("user_agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".to_string()),
        ("browser_language".to_string(), "zh-CN".to_string()),
        ("browser_platform".to_string(), "Win32".to_string()),
        ("priority_region".to_string(), "cn".to_string()),
        ("screen_height".to_string(), "1067".to_string()),
        ("browser_name".to_string(), "Mozilla".to_string()),
    ]);
    request.headers = headers;
    request.auth_type = Some(2);

    let mut request_body = RequestBody::default();
    let mut send_message_request_body = SendMessageRequestBody::default();
    send_message_request_body.conversation_id = Some("".to_string());
    send_message_request_body.conversation_type = Some(11);
    send_message_request_body.conversation_short_id = conversation_short_id;
    send_message_request_body.content = Some(content);
    let client_message_id = uuid_with_hyphen();
    let security_receiver_id = security_conversation_id.clone().unwrap().replace(format!(":{}::2:1:pigeon",shop_id).as_str(),"");
    send_message_request_body.ext = HashMap::from([
        ("s:mentioned_users".to_string(), "".to_string()),
        ("PIGEON_BIZ_TYPE".to_string(), "2".to_string()),
        ("source".to_string(), "pc-web".to_string()),
        ("hierarchical_dimension".to_string(), "{\"dynamic_dimension\":\"6972_9230_4541_1131_8272_6599_9420_8656_0925_4050_3823_3994_7125_8564_1528_0388_8667_2179_8682_7948_1870_1949_0989_8012_6240_4102_7898_7548_6979_6245_9393_3650_0478_4026_4034_4057_9050_6537_2965_8632_2068_8958_0363_2216_2387_0676_9033_0210_3425_9421_0982_1935_8188_3817_1943_8557_8539_3278_4065_1893_6049_6961_3814_9606_4883_4401_6638_8020_7282_3652_0568_3752_9354_0437_4769_4815_9572_7230_5054_7510_4852_3637_2188_3505_6813_2570_5394_0729\",\"goofy_id\":\"1.0.1.5926\",\"desk_version\":\"0.0.0\",\"open_stores\":\"0\",\"memL\":\"\",\"cpuL\":\"\",\"session_throughput\":0,\"message_throughput_send\":0,\"message_throughput_revice\":0}".to_string()),
        ("type".to_string(), "text".to_string()),
        ("shop_id".to_string(), shop_id),
        ("sender_role".to_string(), "2".to_string()),
        ("sender_id".to_string(), "".to_string()),
        ("s:client_message_id".to_string(),client_message_id.clone()),
        ("p:from_source".to_string(), "web".to_string()),
        ("p:check_Send".to_string(), uuid_with_hyphen()),
        ("track_info".to_string(), "{\"send_time\":\"{timestamp_millis}\",\"_send_delta\":\"410\",\"_send_delta_2\":\"392\"}".to_string().replace("{timestamp_millis}", timestamp_millis().to_string().as_str())),
        ("biz_ext".to_string(), "{}".to_string()),
        ("src".to_string(), "pc".to_string()),
        ("srcType".to_string(), "1".to_string()),
        ("user_agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".to_string()),
        ("security_receiver_id".to_string(),security_receiver_id),
    ]);
    send_message_request_body.message_type = Some(1000);
    send_message_request_body.ticket = Some(ticket.clone());
    send_message_request_body.client_message_id = Some(client_message_id);
    send_message_request_body.mentioned_users = vec![];
    send_message_request_body.client_ext = HashMap::default();
    send_message_request_body.sub_conversation_short_id = sub_conversation_short_id;
    send_message_request_body.security_conversation_id = security_conversation_id;
    send_message_request_body.security_mentioned_users = vec![];
    request_body.send_message_body = Some(send_message_request_body);
    request.body = Some(request_body);
    Ok(request)
}

fn get_device_id(webview: &Webview) -> Option<String> {
    let mutex_guard = IM_DEVICE_ID_MAP.lock().unwrap();
    let option = mutex_guard.get(webview.label()).cloned();
    option
}

fn get_feige_im_token(webview: &Webview) -> String {
    let mutex_guard = IM_TOKEN_MAP.lock().unwrap();
    mutex_guard.get(webview.label()).cloned().unwrap()
}

pub fn build_get_conversation_info_list_v2_frame(webview: &Webview) -> Frame {
    build_send_frame(webview)
}

pub fn build_get_conversation_info_list_v2_payload(
    webview: &Webview,
    conversation_short_id: i64,
    security_conversation_id: String,
) -> Result<Request, String> {
    let im_token = get_feige_im_token(webview);
    let mut request = Request::default();
    request.cmd = Some(610);
    request.sequence_id = Some(SEQUENCE_ID.load(Relaxed) as i64);
    request.sdk_version = Some("0.0.0-fix-fix-inbox-array-202632312465".to_string());
    request.token = Some(im_token);
    request.refer = Some(3);
    request.inbox_type = Some(3);
    request.build_number = Some("0e70fed:fix/fix_inbox_array".to_string());
    request.device_id = get_device_id(webview);

    let headers = HashMap::from([
        ("browser_online".to_string(), "true".to_string()),
        ("timezone_name".to_string(), "Asia/Shanghai".to_string()),
        ("referer".to_string(), "https://fxg.jinritemai.com/".to_string()),
        ("browser_version".to_string(), "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".to_string()),
        ("cookie_enabled".to_string(), "true".to_string()),
        ("session_did".to_string(), request.device_id.clone().unwrap()),
        ("pigeon_source".to_string(), "web".to_string()),
        ("pigeon_sign".to_string(), get_pigeon_sign(webview).unwrap()),
        ("app_name".to_string(), "im".to_string()),
        ("session_aid".to_string(), "1383".to_string()),
        ("PIGEON_BIZ_TYPE".to_string(), "2".to_string()),
        ("screen_width".to_string(), "1707".to_string()),
        ("user_agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".to_string()),
        ("browser_language".to_string(), "zh-CN".to_string()),
        ("browser_platform".to_string(), "Win32".to_string()),
        ("priority_region".to_string(), "cn".to_string()),
        ("screen_height".to_string(), "1067".to_string()),
        ("browser_name".to_string(), "Mozilla".to_string()),
    ]);
    request.headers = headers;
    request.auth_type = Some(2);

    let mut request_body = RequestBody::default();
    let conversation_info = GetConversationInfoV2RequestBody {
        conversation_id: Some("".to_string()),
        conversation_short_id: Some(conversation_short_id),
        conversation_type: Some(10),
        need_coreinfo_related: None,
        need_userinfo_related: None,
        user_ids: vec![],
        need_settinginfo_related: None,
        need_last_convindex_v2: None,
        security_conversation_id: Some(security_conversation_id),
        security_user_ids: vec![],
    };
    request_body.get_conversation_info_list_v2_body = Some(GetConversationInfoListV2RequestBody {
        conversation_info_list: vec![conversation_info],
    });
    request.body = Some(request_body);
    Ok(request)
}