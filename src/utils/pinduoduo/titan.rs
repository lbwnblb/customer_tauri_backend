use std::collections::{HashSet, VecDeque};
use std::io::Read;
use std::sync::{LazyLock, Mutex};
use log::{info, warn};
use prost::Message;
use serde_json::Value;

// 去重：记录最近已触发过自动回复的 msg_id，防止同一条消息多次触发
static SEEN_MSG_IDS: LazyLock<Mutex<(HashSet<String>, VecDeque<String>)>> =
    LazyLock::new(|| Mutex::new((HashSet::new(), VecDeque::new())));

fn mark_seen(msg_id: &str) -> bool {
    let mut guard = SEEN_MSG_IDS.lock().unwrap();
    let (set, queue) = &mut *guard;
    if !set.insert(msg_id.to_string()) {
        return false; // 已处理过
    }
    queue.push_back(msg_id.to_string());
    if queue.len() > 500 {
        if let Some(old) = queue.pop_front() {
            set.remove(&old);
        }
    }
    true
}

pub mod titan_proto {
    include!(concat!(env!("OUT_DIR"), "/titan.rs"));
}

pub fn decode_titan_upstream_frame(bytes: &[u8]) {
    if bytes.len() < 16 {
        warn!("[PDD][TITAN] 上行帧太短: len={}", bytes.len());
        return;
    }

    let magic    = u16::from_be_bytes([bytes[0], bytes[1]]);
    let cmd      = u16::from_be_bytes([bytes[2], bytes[3]]);
    let body_len = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

    let payload = &bytes[16..];
    let upstream = match titan_proto::TitanUpstream::decode(payload) {
        Ok(u) => u,
        Err(e) => {
            warn!("[PDD][TITAN] TitanUpstream 解码失败: {}", e);
            return;
        }
    };

    info!(
        "[PDD][TITAN][UP] magic={} cmd={} bodyLen={} command={} upstreamSeq={}",
        magic, cmd, body_len, upstream.command, upstream.upstream_seq
    );

    match upstream.command.as_str() {
        "titan.notifyDataLite.ack" => {
            match titan_proto::NotifyDataLiteAck::decode(upstream.body.as_slice()) {
                Ok(ack) => info!("[PDD][TITAN][UP] NotifyDataLiteAck: {:?}", ack),
                Err(e) => warn!("[PDD][TITAN][UP] NotifyDataLiteAck 解码失败: {}", e),
            }
        }
        other => {
            info!("[PDD][TITAN][UP] command={} body_len={}", other, upstream.body.len());
        }
    }
}

/// 从单个 message 对象中提取买家 UID（role 必须明确为 "user"，type 必须为 0）。
fn try_extract_uid_from_msg(msg: &Value) -> Option<String> {
    let from = msg.get("from")?;
    let role = from.get("role").and_then(|v| v.as_str())?;
    if role != "user" {
        return None;
    }
    let uid = from.get("uid").and_then(|v| v.as_str())?;
    if uid.is_empty() {
        return None;
    }
    let msg_type = msg.get("type").and_then(|v| v.as_u64()).unwrap_or(0);
    if msg_type != 0 {
        return None;
    }
    Some(uid.to_string())
}

/// 从 titan.notifyDataLite payload JSON 中提取买家 UID。
/// 支持两种结构：
///   1. push_data.data[].message.from  （实际 PDD 推送格式）
///   2. 顶层 message.from / from        （兜底）
fn extract_buyer_uid(payload_bytes: &[u8]) -> Option<String> {
    let json: Value = serde_json::from_slice(payload_bytes).ok()?;

    // 结构 1：push_data.data[].message
    if let Some(arr) = json.pointer("/push_data/data").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(uid) = item.get("message").and_then(|m| try_extract_uid_from_msg(m)) {
                return Some(uid);
            }
        }
    }

    // 结构 2：顶层 message 或顶层 from（兜底）
    if let Some(msg) = json.get("message") {
        return try_extract_uid_from_msg(msg);
    }

    None
}

/// 解码 titan 下行帧，返回买家 UID（若该帧包含需回复的买家文本消息）。
pub fn decode_titan_frame(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 16 {
        warn!("[PDD][TITAN] 帧太短: len={}", bytes.len());
        return None;
    }

    let magic    = u16::from_be_bytes([bytes[0], bytes[1]]);
    let cmd      = u16::from_be_bytes([bytes[2], bytes[3]]);
    let body_len = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

    let payload = &bytes[16..];
    let downstream = match titan_proto::TitanDownstream::decode(payload) {
        Ok(d) => d,
        Err(e) => {
            warn!("[PDD][TITAN] TitanDownstream 解码失败: {}", e);
            return None;
        }
    };

    info!(
        "[PDD][TITAN] magic={} cmd={} bodyLen={} command={} compress={}",
        magic, cmd, body_len, downstream.command, downstream.compress
    );

    let body_bytes = if downstream.compress == 1 {
        let mut decoder = flate2::read::GzDecoder::new(downstream.body.as_slice());
        let mut out = Vec::new();
        if let Err(e) = decoder.read_to_end(&mut out) {
            warn!("[PDD][TITAN] gzip 解压失败: {}", e);
            return None;
        }
        out
    } else {
        downstream.body.clone()
    };

    match downstream.command.as_str() {
        "titan.notifyDataLite" => {
            match titan_proto::NotifyDataLite::decode(body_bytes.as_slice()) {
                Ok(mut msg) => {
                    let payload_bytes = msg.payload.clone();
                    let payload_json  = String::from_utf8(payload_bytes.clone()).unwrap_or_default();
                    msg.payload = vec![];
                    info!(
                        "[PDD][TITAN] NotifyDataLite uid={} bizType={} msgId={}",
                        msg.uid, msg.biz_type, msg.msg_id
                    );
                    info!("[PDD][TITAN] payload(json): {}", payload_json);

                    // 只从 payload JSON 提取买家 UID（role 必须明确为 "user"）
                    // 不使用 NotifyDataLite.uid 兜底：该字段在送达回执里也是买家 UID，会误触发
                    let buyer_uid = extract_buyer_uid(&payload_bytes);
                    if buyer_uid.is_some() && !mark_seen(&msg.msg_id) {
                        info!("[PDD][TITAN] 重复 msg_id={} 跳过", msg.msg_id);
                        return None;
                    }
                    return buyer_uid;
                }
                Err(e) => warn!("[PDD][TITAN] NotifyDataLite 解码失败: {}", e),
            }
        }
        "titan.notify" => {
            match titan_proto::Notify::decode(body_bytes.as_slice()) {
                Ok(msg) => info!("[PDD][TITAN] Notify: {:?}", msg),
                Err(e) => warn!("[PDD][TITAN] Notify 解码失败: {}", e),
            }
        }
        "titan.notifyData" => {
            match titan_proto::NotifyData::decode(body_bytes.as_slice()) {
                Ok(msg) => info!("[PDD][TITAN] NotifyData: {:?}", msg),
                Err(e) => warn!("[PDD][TITAN] NotifyData 解码失败: {}", e),
            }
        }
        "titan.ping" => info!("[PDD][TITAN] ping"),
        "titan.session" => info!("[PDD][TITAN] session"),
        other => info!("[PDD][TITAN] 未知 command: {}", other),
    }
    None
}
