use std::io::Read;
use log::{info, warn};
use prost::Message;

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

pub fn decode_titan_frame(bytes: &[u8]) {
    if bytes.len() < 16 {
        warn!("[PDD][TITAN] 帧太短: len={}", bytes.len());
        return;
    }

    let magic    = u16::from_be_bytes([bytes[0], bytes[1]]);
    let cmd      = u16::from_be_bytes([bytes[2], bytes[3]]);
    let body_len = u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

    let payload = &bytes[16..];
    let downstream = match titan_proto::TitanDownstream::decode(payload) {
        Ok(d) => d,
        Err(e) => {
            warn!("[PDD][TITAN] TitanDownstream 解码失败: {}", e);
            return;
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
            return;
        }
        out
    } else {
        downstream.body.clone()
    };

    match downstream.command.as_str() {
        "titan.notifyDataLite" => {
            match titan_proto::NotifyDataLite::decode(body_bytes.as_slice()) {
                Ok(mut msg) => {
                    let payload_json = String::from_utf8(msg.payload.clone()).unwrap_or_default();
                    msg.payload = vec![];
                    info!("[PDD][TITAN] NotifyDataLite: {:?}", msg);
                    info!("[PDD][TITAN] payload(json): {}", payload_json);
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
        "titan.ping" => {
            info!("[PDD][TITAN] ping");
        }
        "titan.session" => {
            info!("[PDD][TITAN] session");
        }
        other => {
            info!("[PDD][TITAN] 未知 command: {}", other);
        }
    }
}
