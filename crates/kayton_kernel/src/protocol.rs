use std::sync::OnceLock;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageHeader {
    pub msg_id: String,
    pub username: String,
    pub session: String,
    pub date: String,
    pub msg_type: String,
    pub version: String,
}

#[derive(Debug)]
pub struct ParsedMessage {
    pub idents: Vec<Vec<u8>>,
    pub signature: String,
    pub header_bytes: Vec<u8>,
    pub parent_bytes: Vec<u8>,
    pub metadata_bytes: Vec<u8>,
    pub content_bytes: Vec<u8>,
    pub header: Value,
    pub parent: Value,
    pub metadata: Value,
    pub content: Value,
    pub buffers: Vec<Vec<u8>>,
}

static KERNEL_SESSION: OnceLock<String> = OnceLock::new();

pub fn kernel_session_id() -> &'static str {
    KERNEL_SESSION.get_or_init(|| Uuid::new_v4().to_string())
}

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn base_header(msg_type: &str, _parent: &Value) -> MessageHeader {
    let username = "kayton".to_string();
    let session = kernel_session_id().to_string();
    MessageHeader {
        msg_id: Uuid::new_v4().to_string(),
        username,
        session,
        date: now_rfc3339(),
        msg_type: msg_type.to_string(),
        version: "5.3".to_string(),
    }
}

pub fn serialize_header(h: &MessageHeader) -> Vec<u8> {
    serde_json::to_vec(h).unwrap()
}

pub fn empty_obj_bytes() -> Vec<u8> {
    b"{}".to_vec()
}

pub fn parse_message(
    frames: &[Vec<u8>],
) -> Option<(
    Vec<Vec<u8>>, // idents
    String,       // signature
    Value,        // header
    Value,        // parent
    Value,        // metadata
    Value,        // content
    Vec<Vec<u8>>, // buffers
)> {
    if frames.is_empty() {
        return None;
    }
    let mut idx = 0usize;
    let mut idents: Vec<Vec<u8>> = Vec::new();
    while idx < frames.len() && frames[idx].as_slice() != b"<IDS|MSG>" {
        idents.push(frames[idx].clone());
        idx += 1;
    }
    if idx >= frames.len() {
        return None;
    }
    idx += 1; // <IDS|MSG>
    if idx + 4 >= frames.len() {
        return None;
    }

    let signature = String::from_utf8(frames[idx].clone()).unwrap_or_default();
    idx += 1;

    let header: Value = serde_json::from_slice(&frames[idx]).ok()?;
    idx += 1;
    let parent: Value = serde_json::from_slice(&frames[idx]).ok()?;
    idx += 1;
    let metadata: Value = serde_json::from_slice(&frames[idx]).ok()?;
    idx += 1;
    let content: Value = serde_json::from_slice(&frames[idx]).ok()?;
    idx += 1;

    let buffers = frames[idx..].to_vec();
    Some((
        idents, signature, header, parent, metadata, content, buffers,
    ))
}

pub fn parse_message2(frames: &[Vec<u8>]) -> Option<ParsedMessage> {
    if frames.is_empty() {
        return None;
    }
    let mut idx = 0usize;
    let mut idents: Vec<Vec<u8>> = Vec::new();
    while idx < frames.len() && frames[idx].as_slice() != b"<IDS|MSG>" {
        idents.push(frames[idx].clone());
        idx += 1;
    }
    if idx >= frames.len() {
        return None;
    }
    idx += 1; // <IDS|MSG>
    if idx + 4 >= frames.len() {
        return None;
    }

    let signature = String::from_utf8(frames[idx].clone()).ok()?;
    idx += 1;

    let header_bytes = frames[idx].clone();
    let header: Value = serde_json::from_slice(&header_bytes).ok()?;
    idx += 1;

    let parent_bytes = frames[idx].clone();
    let parent: Value = serde_json::from_slice(&parent_bytes).ok()?;
    idx += 1;

    let metadata_bytes = frames[idx].clone();
    let metadata: Value = serde_json::from_slice(&metadata_bytes).ok()?;
    idx += 1;

    let content_bytes = frames[idx].clone();
    let content: Value = serde_json::from_slice(&content_bytes).ok()?;
    idx += 1;

    let buffers = frames[idx..].to_vec();

    Some(ParsedMessage {
        idents,
        signature,
        header_bytes,
        parent_bytes,
        metadata_bytes,
        content_bytes,
        header,
        parent,
        metadata,
        content,
        buffers,
    })
}
