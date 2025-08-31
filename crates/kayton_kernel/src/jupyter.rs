use anyhow::{Result, anyhow};
use chrono::{SecondsFormat, Utc};
use hmac::{Hmac, Mac};
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use std::time::Duration;
use uuid::Uuid;
use zmq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct ConnectionFile {
    pub ip: String,
    pub transport: String,
    pub key: String,
    pub shell_port: u16,
    pub iopub_port: u16,
    pub stdin_port: u16,
    pub control_port: u16,
    pub hb_port: u16,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Header {
    msg_id: String,
    username: String,
    session: String,
    date: String,
    msg_type: String,
    version: String,
}

#[derive(Serialize, Deserialize, Default)]
struct ExecuteRequestContent {
    code: String,
    silent: Option<bool>,
}

fn iso_now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn make_header(msg_type: &str, session: &str) -> Header {
    Header {
        msg_id: Uuid::new_v4().to_string(),
        username: String::from("kayton"),
        session: session.to_string(),
        date: iso_now(),
        msg_type: msg_type.to_string(),
        version: String::from("5.3"),
    }
}

fn parent_session(parent: &serde_json::Value) -> String {
    parent
        .get("session")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

const DELIM: &str = "<IDS|MSG>";

fn sign(key: &[u8], parts: &[&[u8]]) -> String {
    if key.is_empty() {
        return String::new();
    }
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    for p in parts {
        mac.update(p);
    }
    let res = mac.finalize().into_bytes();
    hex::encode(res)
}

fn frames_as_json<'a>(
    frames: &'a [zmq::Message],
) -> Result<(&'a [u8], &'a [u8], &'a [u8], &'a [u8])> {
    let hdr = frames.get(0).ok_or_else(|| anyhow!("missing header"))?;
    let parent = frames.get(1).ok_or_else(|| anyhow!("missing parent"))?;
    let meta = frames.get(2).ok_or_else(|| anyhow!("missing metadata"))?;
    let content = frames.get(3).ok_or_else(|| anyhow!("missing content"))?;
    Ok((
        hdr.as_ref(),
        parent.as_ref(),
        meta.as_ref(),
        content.as_ref(),
    ))
}

fn send_msg(
    sock: &zmq::Socket,
    idents: &[Vec<u8>],
    key: &[u8],
    header: &Header,
    parent_header: &serde_json::Value,
    metadata: &serde_json::Value,
    content: &serde_json::Value,
) -> Result<()> {
    // ROUTER sockets require identities first
    for id in idents.iter() {
        sock.send(id.as_slice(), zmq::SNDMORE)?;
    }
    if !idents.is_empty() {
        sock.send(DELIM, zmq::SNDMORE)?;
    } else {
        // PUB: no idents, but still write DELIM first
        sock.send(DELIM, zmq::SNDMORE)?;
    }

    let h_bytes = serde_json::to_vec(header)?;
    let p_bytes = serde_json::to_vec(parent_header)?;
    let m_bytes = serde_json::to_vec(metadata)?;
    let c_bytes = serde_json::to_vec(content)?;
    let sig = sign(key, &[&h_bytes, &p_bytes, &m_bytes, &c_bytes]);

    sock.send(sig.as_bytes(), zmq::SNDMORE)?;
    sock.send(h_bytes, zmq::SNDMORE)?;
    sock.send(p_bytes, zmq::SNDMORE)?;
    sock.send(m_bytes, zmq::SNDMORE)?;
    sock.send(c_bytes, 0)?;
    Ok(())
}

fn send_iopub(
    iopub: &zmq::Socket,
    key: &[u8],
    msg_type: &str,
    parent_header: &serde_json::Value,
    content: serde_json::Value,
) -> Result<()> {
    let h = make_header(msg_type, &parent_session(parent_header));
    let meta = serde_json::json!({});
    // No identities for iopub (PUB)
    let empty: [Vec<u8>; 0] = [];
    send_msg(iopub, &empty, key, &h, parent_header, &meta, &content)
}

/// Very simple completeness heuristic for the console:
/// - If code ends with ':' (block opener), report incomplete and suggest indent
/// - If parentheses are unbalanced, report incomplete
/// - Otherwise, assume complete
fn compute_is_complete(code: &str) -> (&'static str, &'static str) {
    let trimmed = code.trim_end();
    if trimmed.is_empty() {
        return ("complete", "");
    }

    // Track simple paren balance, ignore extremely complex cases
    let mut depth: i32 = 0;
    let mut in_str: bool = false;
    let mut prev_ch: char = '\0';
    for ch in trimmed.chars() {
        if ch == '"' && prev_ch != '\\' {
            in_str = !in_str;
        }
        if !in_str {
            if ch == '(' {
                depth += 1;
            }
            if ch == ')' {
                depth -= 1;
            }
        }
        prev_ch = ch;
    }
    if depth > 0 {
        return ("incomplete", "");
    }
    if trimmed.ends_with(':') {
        return ("incomplete", "    ");
    }
    ("complete", "")
}

pub fn run_kernel(connection_file: &std::path::Path) -> Result<()> {
    info!("loading connection file: {}", connection_file.display());
    let cfg: ConnectionFile = serde_json::from_str(&fs::read_to_string(connection_file)?)?;
    info!(
        "parsed connection: transport={} ip={} shell={} iopub={} control={} stdin={} hb={} key_len={}",
        cfg.transport,
        cfg.ip,
        cfg.shell_port,
        cfg.iopub_port,
        cfg.control_port,
        cfg.stdin_port,
        cfg.hb_port,
        cfg.key.len()
    );
    let key_bytes = if cfg.key.is_empty() {
        vec![]
    } else {
        cfg.key.as_bytes().to_vec()
    };
    let transport = format!("{}://{}:", cfg.transport, cfg.ip);
    let hb_addr = format!("{}{}", transport, cfg.hb_port);
    let shell_addr = format!("{}{}", transport, cfg.shell_port);
    let control_addr = format!("{}{}", transport, cfg.control_port);
    let iopub_addr = format!("{}{}", transport, cfg.iopub_port);
    let stdin_addr = format!("{}{}", transport, cfg.stdin_port);
    info!(
        "binding sockets: hb={} shell={} control={} iopub={} stdin={}",
        hb_addr, shell_addr, control_addr, iopub_addr, stdin_addr
    );

    let ctx = zmq::Context::new();

    let hb = ctx.socket(zmq::REP)?;
    hb.bind(&hb_addr)?;
    info!("hb bound");
    let shell = ctx.socket(zmq::ROUTER)?;
    shell.bind(&shell_addr)?;
    info!("shell bound");
    let control = ctx.socket(zmq::ROUTER)?;
    control.bind(&control_addr)?;
    info!("control bound");
    let iopub = ctx.socket(zmq::PUB)?;
    iopub.bind(&iopub_addr)?;
    info!("iopub bound");
    let _stdin_sock = ctx.socket(zmq::ROUTER)?; // reserved for input requests
    _stdin_sock.bind(&stdin_addr)?;
    info!("stdin bound");

    // Heartbeat echo thread
    let _hb_thread = std::thread::spawn(move || {
        let mut msg = zmq::Message::new();
        let mut count: u64 = 0;
        loop {
            if hb.recv(&mut msg, 0).is_ok() {
                count += 1;
                if count <= 5 || count % 100 == 0 {
                    debug!("heartbeat recv count={}", count);
                }
                let _ = hb.send(msg.as_ref(), 0);
            }
        }
    });

    // Execution state
    let mut state = InteractiveState::new();
    let mut exec_count: i32 = 0;

    let mut poll_items = [
        shell.as_poll_item(zmq::POLLIN),
        control.as_poll_item(zmq::POLLIN),
    ];

    loop {
        let poll_rc = zmq::poll(&mut poll_items, 1000)?;
        if poll_rc == 0 {
            debug!("poll timeout; alive");
        }

        // Control channel (shutdown)
        if poll_items[1].is_readable() {
            let mut frames: Vec<zmq::Message> = Vec::new();
            loop {
                let mut part = zmq::Message::new();
                control.recv(&mut part, 0)?;
                let more = control.get_rcvmore()?;
                frames.push(part);
                if !more {
                    break;
                }
            }
            // Parse idents and message
            // ROUTER receives: [idents..., DELIM, sig, header, parent, metadata, content]
            let mut idx = 0;
            while idx < frames.len() && frames[idx].as_ref() != DELIM.as_bytes() {
                idx += 1;
            }
            if idx >= frames.len() {
                warn!("control missing delimiter frame");
                continue;
            }
            let idents: Vec<Vec<u8>> = frames[..idx].iter().map(|m| m.as_ref().to_vec()).collect();
            let body = &frames[idx + 1..];
            if body.len() < 5 {
                warn!("control body too short: {}", body.len());
                continue;
            }
            let _sig = &body[0];
            let (hdr_b, parent_b, meta_b, content_b) = match frames_as_json(&body[1..]) {
                Ok(v) => v,
                Err(e) => {
                    warn!("control frames_as_json error: {}", e);
                    continue;
                }
            };
            let header_v: serde_json::Value = match serde_json::from_slice(hdr_b) {
                Ok(v) => v,
                Err(e) => {
                    error!("control header parse error: {}", e);
                    continue;
                }
            };
            let _parent_v: serde_json::Value = match serde_json::from_slice(parent_b) {
                Ok(v) => v,
                Err(e) => {
                    error!("control parent parse error: {}", e);
                    continue;
                }
            };
            let _meta: serde_json::Value = match serde_json::from_slice(meta_b) {
                Ok(v) => v,
                Err(e) => {
                    warn!("control metadata parse error: {}", e);
                    serde_json::json!({})
                }
            };
            let content: serde_json::Value = serde_json::from_slice(content_b)?;
            let mt = header_v
                .get("msg_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            debug!(
                "control msg_type={} idents={} body_frames={} session={}",
                mt,
                idents.len(),
                body.len(),
                header_v
                    .get("session")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            );
            if mt == "shutdown_request" {
                let reply_h = make_header("shutdown_reply", &parent_session(&header_v));
                let meta = serde_json::json!({});
                send_msg(
                    &control, &idents, &key_bytes, &reply_h, &header_v, &meta, &content,
                )?;
                info!("shutdown_request processed; exiting run_kernel loop");
                break;
            }
        }

        // Shell channel
        if poll_items[0].is_readable() {
            let mut frames: Vec<zmq::Message> = Vec::new();
            loop {
                let mut part = zmq::Message::new();
                shell.recv(&mut part, 0)?;
                let more = shell.get_rcvmore()?;
                frames.push(part);
                if !more {
                    break;
                }
            }
            let mut idx = 0;
            while idx < frames.len() && frames[idx].as_ref() != DELIM.as_bytes() {
                idx += 1;
            }
            if idx >= frames.len() {
                warn!("shell missing delimiter frame");
                continue;
            }
            let idents: Vec<Vec<u8>> = frames[..idx].iter().map(|m| m.as_ref().to_vec()).collect();
            let body = &frames[idx + 1..];
            if body.len() < 5 {
                warn!("shell body too short: {}", body.len());
                continue;
            }
            let _sig = &body[0];
            let (hdr_b, parent_b, meta_b, content_b) = match frames_as_json(&body[1..]) {
                Ok(v) => v,
                Err(e) => {
                    warn!("shell frames_as_json error: {}", e);
                    continue;
                }
            };
            let header_v: serde_json::Value = match serde_json::from_slice(hdr_b) {
                Ok(v) => v,
                Err(e) => {
                    error!("shell header parse error: {}", e);
                    continue;
                }
            };
            let _parent_v: serde_json::Value = match serde_json::from_slice(parent_b) {
                Ok(v) => v,
                Err(e) => {
                    error!("shell parent parse error: {}", e);
                    continue;
                }
            };
            let _meta: serde_json::Value = match serde_json::from_slice(meta_b) {
                Ok(v) => v,
                Err(e) => {
                    warn!("shell metadata parse error: {}", e);
                    serde_json::json!({})
                }
            };
            let content: serde_json::Value = serde_json::from_slice(content_b)?;

            match header_v
                .get("msg_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
            {
                "is_complete_request" => {
                    // Reply whether input is complete per Jupyter protocol
                    let code = content.get("code").and_then(|v| v.as_str()).unwrap_or("");
                    let (status, indent) = compute_is_complete(code);
                    let reply_h = make_header("is_complete_reply", &parent_session(&header_v));
                    let meta = serde_json::json!({});
                    let mut reply = serde_json::json!({
                        "status": status
                    });
                    if status == "incomplete" && !indent.is_empty() {
                        reply["indent"] = serde_json::Value::String(indent.to_string());
                    }
                    send_msg(
                        &shell, &idents, &key_bytes, &reply_h, &header_v, &meta, &reply,
                    )?;
                }
                "kernel_info_request" => {
                    info!("kernel_info_request received");
                    let reply_h = make_header("kernel_info_reply", &parent_session(&header_v));
                    let lang = serde_json::json!({
                        "name": "kayton",
                        "version": "0.1.0",
                        "mimetype": "text/plain",
                        "file_extension": ".kay"
                    });
                    let content = serde_json::json!({
                        "protocol_version": "5.3",
                        "implementation": "kayton-kernel",
                        "implementation_version": "0.1.0",
                        "language_info": lang,
                        "status": "ok"
                    });
                    let meta = serde_json::json!({});
                    send_msg(
                        &shell, &idents, &key_bytes, &reply_h, &header_v, &meta, &content,
                    )?;
                }
                "execute_request" => {
                    let code = content.get("code").and_then(|v| v.as_str()).unwrap_or("");
                    info!(
                        "execute_request received: bytes={} lines={}",
                        code.len(),
                        code.lines().count()
                    );
                    exec_count += 1;
                    // Publish busy
                    let _ = send_iopub(
                        &iopub,
                        &key_bytes,
                        "status",
                        &header_v,
                        serde_json::json!({"execution_state": "busy"}),
                    );
                    // Publish execute_input
                    let _ = send_iopub(
                        &iopub,
                        &key_bytes,
                        "execute_input",
                        &header_v,
                        serde_json::json!({
                            "code": code,
                            "execution_count": exec_count
                        }),
                    );

                    // Run code via interactive engine
                    let mut reply_status = "ok".to_string();
                    if !code.trim().is_empty() {
                        match prepare_input(&mut state, code) {
                            Ok(prep) => {
                                if let Err(e) = execute_prepared(&mut state, &prep) {
                                    reply_status = "error".to_string();
                                    error!("execute_prepared error: {}", e);
                                    let _ = send_iopub(
                                        &iopub,
                                        &key_bytes,
                                        "stream",
                                        &header_v,
                                        serde_json::json!({
                                            "name": "stderr",
                                            "text": e.to_string()
                                        }),
                                    );
                                }
                            }
                            Err(e) => {
                                reply_status = "error".to_string();
                                error!("prepare_input error: {}", e);
                                let _ = send_iopub(
                                    &iopub,
                                    &key_bytes,
                                    "stream",
                                    &header_v,
                                    serde_json::json!({
                                        "name": "stderr",
                                        "text": e.to_string()
                                    }),
                                );
                            }
                        }
                    }

                    // Prepare a simple execute_result showing globals
                    let globals = state.vm_mut().read_all_globals_as_strings();
                    let mut lines = String::new();
                    for (n, v) in globals {
                        lines.push_str(&format!("{} = {}\n", n, v));
                    }
                    if !lines.is_empty() {
                        let _ = send_iopub(
                            &iopub,
                            &key_bytes,
                            "execute_result",
                            &header_v,
                            serde_json::json!({
                                "execution_count": exec_count,
                                "data": {"text/plain": lines},
                                "metadata": {}
                            }),
                        );
                    }

                    // Reply on shell
                    let reply_h = make_header("execute_reply", &parent_session(&header_v));
                    let meta = serde_json::json!({});
                    let reply = serde_json::json!({
                        "status": reply_status,
                        "execution_count": exec_count,
                        "user_expressions": {},
                        "payload": []
                    });
                    debug!(
                        "sending execute_reply status={} exec_count={}",
                        reply_status, exec_count
                    );
                    send_msg(
                        &shell, &idents, &key_bytes, &reply_h, &header_v, &meta, &reply,
                    )?;

                    // Publish idle
                    let _ = send_iopub(
                        &iopub,
                        &key_bytes,
                        "status",
                        &header_v,
                        serde_json::json!({"execution_state": "idle"}),
                    );
                }
                _ => {
                    // Unknown: reply with ok blank
                    let mt = header_v
                        .get("msg_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    warn!("unhandled msg_type on shell: {}", mt);
                    let reply_h = make_header(&format!("{}_reply", mt), &parent_session(&header_v));
                    let meta = serde_json::json!({});
                    let reply = serde_json::json!({"status": "ok"});
                    let _ = send_msg(
                        &shell, &idents, &key_bytes, &reply_h, &header_v, &meta, &reply,
                    );
                }
            }
        }

        // Prevent busy loop
        std::thread::sleep(Duration::from_millis(5));
    }

    Ok(())
}
