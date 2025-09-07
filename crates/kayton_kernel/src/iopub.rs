use anyhow::Result;
use serde_json::Value;

use crate::protocol::{base_header, empty_obj_bytes, serialize_header};
use crate::signing::sign_bytes;

pub fn send_reply(
    socket: &zmq::Socket,
    idents: &[Vec<u8>],
    key: &[u8],
    parent_header: &Value,
    msg_type: &str,
    content: Value,
) -> Result<()> {
    let header = base_header(msg_type, parent_header);
    let header_bytes = serialize_header(&header);
    let parent_bytes = serde_json::to_vec(parent_header)?;
    let metadata_bytes = empty_obj_bytes();
    let content_bytes = serde_json::to_vec(&content)?;
    let signature = sign_bytes(
        key,
        &[&header_bytes, &parent_bytes, &metadata_bytes, &content_bytes],
    );

    let mut frames: Vec<Vec<u8>> = Vec::with_capacity(idents.len() + 7);
    frames.extend(idents.iter().cloned());
    frames.push(b"<IDS|MSG>".to_vec());
    frames.push(signature.into_bytes());
    frames.push(header_bytes);
    frames.push(parent_bytes);
    frames.push(metadata_bytes);
    frames.push(content_bytes);
    socket.send_multipart(frames, 0)?;
    Ok(())
}

fn publish_on_iopub(
    iopub: &zmq::Socket,
    key: &[u8],
    parent_header: &Value,
    msg_type: &str,
    content: Value,
) -> Result<()> {
    let header = base_header(msg_type, parent_header);
    let header_bytes = serialize_header(&header);
    let parent_bytes = serde_json::to_vec(parent_header)?;
    let metadata_bytes = empty_obj_bytes();
    let content_bytes = serde_json::to_vec(&content)?;
    let signature = sign_bytes(
        key,
        &[&header_bytes, &parent_bytes, &metadata_bytes, &content_bytes],
    );

    let frames = vec![
        b"<IDS|MSG>".to_vec(),
        signature.into_bytes(),
        header_bytes,
        parent_bytes,
        metadata_bytes,
        content_bytes,
    ];
    iopub.send_multipart(frames, 0)?;
    Ok(())
}

pub fn publish_status(iopub: &zmq::Socket, key: &[u8], parent: &Value, state: &str) -> Result<()> {
    let content = serde_json::json!({"execution_state": state});
    publish_on_iopub(iopub, key, parent, "status", content)
}

pub fn publish_execute_input(
    iopub: &zmq::Socket,
    key: &[u8],
    parent: &Value,
    code: &str,
    execution_count: i32,
) -> Result<()> {
    let content = serde_json::json!({"code": code, "execution_count": execution_count});
    publish_on_iopub(iopub, key, parent, "execute_input", content)
}

pub fn publish_stream(
    iopub: &zmq::Socket,
    key: &[u8],
    parent: &Value,
    name: &str,
    text: &str,
) -> Result<()> {
    let content = serde_json::json!({"name": name, "text": text});
    publish_on_iopub(iopub, key, parent, "stream", content)
}

pub fn publish_execute_result(
    iopub: &zmq::Socket,
    key: &[u8],
    parent: &Value,
    execution_count: i32,
    code: &str,
) -> Result<()> {
    let content = serde_json::json!({
        "execution_count": execution_count,
        "data": {"text/plain": code},
        "metadata": {}
    });
    publish_on_iopub(iopub, key, parent, "execute_result", content)
}
