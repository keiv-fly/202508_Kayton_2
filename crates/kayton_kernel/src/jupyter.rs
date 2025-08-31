use anyhow::{Result, anyhow};
use chrono::{SecondsFormat, Utc};
use hmac::{Hmac, Mac};
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};
use log::{info, warn};
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
        .get("header")
        .and_then(|h| h.get("session"))
        .and_then(|v| v.as_str())
        .or_else(|| parent.get("session").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string()
}

const DELIM: &str = "<IDS|MSG>";

fn sign(key: &[u8], parts: &[&[u8]]) -> String {
    if key.is_empty() {
        return String::new();
    }
    match HmacSha256::new_from_slice(key) {
        Ok(mut mac) => {
            for p in parts {
                mac.update(p);
            }
            let res = mac.finalize().into_bytes();
            hex::encode(res)
        }
        Err(_) => String::new(),
    }
}

fn verify_signature(key: &[u8], signature: &str, parts: &[&[u8]]) -> bool {
    if key.is_empty() && signature.is_empty() {
        return true; // No authentication required
    }
    let expected = sign(key, parts);
    expected == signature
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
    // Send identities first for ROUTER sockets
    for id in idents.iter() {
        sock.send(id.as_slice(), zmq::SNDMORE)?;
    }

    // Send delimiter
    sock.send(DELIM, zmq::SNDMORE)?;

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
    // No identities for iopub (PUB socket)
    let empty: [Vec<u8>; 0] = [];
    send_msg(iopub, &empty, key, &h, parent_header, &meta, &content)
}

fn compute_is_complete(code: &str) -> (&'static str, &'static str) {
    let trimmed = code.trim_end();
    if trimmed.is_empty() {
        return ("complete", "");
    }

    let mut depth: i32 = 0;
    let mut in_str: bool = false;
    let mut prev_ch: char = '\0';

    for ch in trimmed.chars() {
        if ch == '"' && prev_ch != '\\' {
            in_str = !in_str;
        }
        if !in_str {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
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

fn parse_message(
    frames: &[zmq::Message],
    key: &[u8],
) -> Result<(
    Vec<Vec<u8>>,
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
)> {
    // Find delimiter
    let mut delim_idx = None;
    for (i, frame) in frames.iter().enumerate() {
        if frame.as_ref() == DELIM.as_bytes() {
            delim_idx = Some(i);
            break;
        }
    }

    let delim_idx = delim_idx.ok_or_else(|| anyhow!("No delimiter found"))?;

    // Extract identities (everything before delimiter)
    let idents: Vec<Vec<u8>> = frames[..delim_idx]
        .iter()
        .map(|f| f.as_ref().to_vec())
        .collect();

    // Message parts after delimiter: signature, header, parent_header, metadata, content
    let msg_frames = &frames[delim_idx + 1..];
    if msg_frames.len() < 5 {
        return Err(anyhow!("Insufficient message frames: {}", msg_frames.len()));
    }

    let signature = String::from_utf8_lossy(msg_frames[0].as_ref()).to_string();

    // Verify signature before parsing JSON
    let message_parts = [
        msg_frames[1].as_ref(), // header
        msg_frames[2].as_ref(), // parent_header
        msg_frames[3].as_ref(), // metadata
        msg_frames[4].as_ref(), // content
    ];

    if !verify_signature(key, &signature, &message_parts) {
        return Err(anyhow!("Message signature verification failed"));
    }

    let header: serde_json::Value = serde_json::from_slice(msg_frames[1].as_ref())?;
    let parent_header: serde_json::Value = serde_json::from_slice(msg_frames[2].as_ref())?;
    let metadata: serde_json::Value = serde_json::from_slice(msg_frames[3].as_ref())?;
    let content: serde_json::Value = serde_json::from_slice(msg_frames[4].as_ref())?;

    Ok((idents, header, parent_header, metadata, content))
}

pub fn run_kernel(connection_file: &std::path::Path) -> Result<()> {
    info!("Loading connection file: {}", connection_file.display());
    let cfg: ConnectionFile = serde_json::from_str(&fs::read_to_string(connection_file)?)?;

    let key_bytes = if cfg.key.is_empty() {
        vec![]
    } else {
        cfg.key.as_bytes().to_vec()
    };

    let transport = format!("{}://{}:", cfg.transport, cfg.ip);
    info!("Binding to transport: {}", transport);

    let ctx = zmq::Context::new();

    // Create sockets
    let hb = ctx.socket(zmq::REP)?;
    let shell = ctx.socket(zmq::ROUTER)?;
    let control = ctx.socket(zmq::ROUTER)?;
    let iopub = ctx.socket(zmq::PUB)?;
    let stdin_sock = ctx.socket(zmq::ROUTER)?;

    // Set socket options for better performance
    hb.set_linger(1000)?;
    shell.set_linger(1000)?;
    control.set_linger(1000)?;
    iopub.set_linger(1000)?;
    stdin_sock.set_linger(1000)?;

    // Bind sockets
    hb.bind(&format!("{}{}", transport, cfg.hb_port))?;
    shell.bind(&format!("{}{}", transport, cfg.shell_port))?;
    control.bind(&format!("{}{}", transport, cfg.control_port))?;
    iopub.bind(&format!("{}{}", transport, cfg.iopub_port))?;
    stdin_sock.bind(&format!("{}{}", transport, cfg.stdin_port))?;

    info!("All sockets bound successfully");

    // Give ZeroMQ time to establish connections
    std::thread::sleep(Duration::from_millis(100));

    // Heartbeat thread
    let _hb_thread = std::thread::spawn(move || {
        let mut msg = zmq::Message::new();
        loop {
            if hb.recv(&mut msg, 0).is_ok() {
                let _ = hb.send(msg.as_ref(), 0);
            }
        }
    });

    let mut state = InteractiveState::new();
    let mut exec_count: i32 = 0;

    // Main event loop
    let mut poll_items = [
        shell.as_poll_item(zmq::POLLIN),
        control.as_poll_item(zmq::POLLIN),
    ];

    info!("Kernel ready, starting event loop");

    loop {
        // Poll with shorter timeout for better responsiveness
        let poll_result = zmq::poll(&mut poll_items, 100)?;

        // Handle control messages (shutdown, etc.)
        if poll_items[1].is_readable() {
            match handle_control_message(&control, &key_bytes) {
                Ok(should_shutdown) => {
                    if should_shutdown {
                        info!("Shutdown requested, exiting");
                        break;
                    }
                }
                Err(e) => warn!("Control message error: {}", e),
            }
        }

        // Handle shell messages (execute, kernel_info, etc.)
        if poll_items[0].is_readable() {
            match handle_shell_message(&shell, &iopub, &key_bytes, &mut state, &mut exec_count) {
                Ok(_) => {}
                Err(e) => warn!("Shell message error: {}", e),
            }
        }

        if poll_result == 0 {
            // No messages, small sleep to prevent busy waiting
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    Ok(())
}

fn handle_control_message(control: &zmq::Socket, key: &[u8]) -> Result<bool> {
    let mut frames = Vec::new();
    loop {
        let mut frame = zmq::Message::new();
        control.recv(&mut frame, 0)?;
        let more = control.get_rcvmore()?;
        frames.push(frame);
        if !more {
            break;
        }
    }

    let (idents, header, _parent_header, _metadata, content) = parse_message(&frames, key)?;

    let msg_type = header
        .get("msg_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if msg_type == "shutdown_request" {
        let reply_header = make_header(
            "shutdown_reply",
            &header.get("session").and_then(|v| v.as_str()).unwrap_or(""),
        );
        let metadata = serde_json::json!({});

        send_msg(
            control,
            &idents,
            key,
            &reply_header,
            &header,
            &metadata,
            &content,
        )?;
        return Ok(true); // Signal shutdown
    }

    Ok(false)
}

fn handle_shell_message(
    shell: &zmq::Socket,
    iopub: &zmq::Socket,
    key: &[u8],
    state: &mut InteractiveState,
    exec_count: &mut i32,
) -> Result<()> {
    let mut frames = Vec::new();
    loop {
        let mut frame = zmq::Message::new();
        shell.recv(&mut frame, 0)?;
        let more = shell.get_rcvmore()?;
        frames.push(frame);
        if !more {
            break;
        }
    }

    let (idents, header, _parent_header, _metadata, content) = parse_message(&frames, key)?;

    let msg_type = header
        .get("msg_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let session = header.get("session").and_then(|v| v.as_str()).unwrap_or("");

    match msg_type {
        "kernel_info_request" => {
            info!("Handling kernel_info_request");
            let reply_header = make_header("kernel_info_reply", session);
            let metadata = serde_json::json!({});
            let content = serde_json::json!({
                "protocol_version": "5.3",
                "implementation": "kayton-kernel",
                "implementation_version": "0.1.0",
                "language_info": {
                    "name": "kayton",
                    "version": "0.1.0",
                    "mimetype": "text/plain",
                    "file_extension": ".kay"
                },
                "banner": "Kayton 0.1.0",
                "status": "ok"
            });
            send_msg(
                shell,
                &idents,
                key,
                &reply_header,
                &header,
                &metadata,
                &content,
            )?;
        }

        "is_complete_request" => {
            let code = content.get("code").and_then(|v| v.as_str()).unwrap_or("");
            let (status, indent) = compute_is_complete(code);

            let reply_header = make_header("is_complete_reply", session);
            let metadata = serde_json::json!({});
            let mut reply_content = serde_json::json!({"status": status});

            if status == "incomplete" && !indent.is_empty() {
                reply_content["indent"] = serde_json::Value::String(indent.to_string());
            }

            send_msg(
                shell,
                &idents,
                key,
                &reply_header,
                &header,
                &metadata,
                &reply_content,
            )?;
        }

        "execute_request" => {
            let code = content.get("code").and_then(|v| v.as_str()).unwrap_or("");
            *exec_count += 1;

            info!(
                "Executing code (exec_count={}): {} chars",
                *exec_count,
                code.len()
            );

            // Send status: busy
            send_iopub(
                iopub,
                key,
                "status",
                &header,
                serde_json::json!({"execution_state": "busy"}),
            )?;

            // Send execute_input
            send_iopub(
                iopub,
                key,
                "execute_input",
                &header,
                serde_json::json!({
                    "code": code,
                    "execution_count": *exec_count
                }),
            )?;

            let mut reply_status = "ok";

            // Execute the code
            if !code.trim().is_empty() {
                match prepare_input(state, code) {
                    Ok(prep) => {
                        if let Err(e) = execute_prepared(state, &prep) {
                            reply_status = "error";
                            send_iopub(
                                iopub,
                                key,
                                "error",
                                &header,
                                serde_json::json!({
                                    "ename": "ExecutionError",
                                    "evalue": e.to_string(),
                                    "traceback": [e.to_string()]
                                }),
                            )?;
                        }
                    }
                    Err(e) => {
                        reply_status = "error";
                        send_iopub(
                            iopub,
                            key,
                            "error",
                            &header,
                            serde_json::json!({
                                "ename": "ParseError",
                                "evalue": e.to_string(),
                                "traceback": [e.to_string()]
                            }),
                        )?;
                    }
                }
            }

            // Send results if successful
            if reply_status == "ok" {
                let globals = state.vm_mut().read_all_globals_as_strings();
                if !globals.is_empty() {
                    let mut output = String::new();
                    for (name, value) in globals {
                        output.push_str(&format!("{} = {}\n", name, value));
                    }

                    send_iopub(
                        iopub,
                        key,
                        "execute_result",
                        &header,
                        serde_json::json!({
                            "execution_count": *exec_count,
                            "data": {"text/plain": output.trim()},
                            "metadata": {}
                        }),
                    )?;
                }
            }

            // Send execute_reply
            let reply_header = make_header("execute_reply", session);
            let metadata = serde_json::json!({});
            let reply_content = serde_json::json!({
                "status": reply_status,
                "execution_count": *exec_count,
                "user_expressions": {},
                "payload": []
            });

            send_msg(
                shell,
                &idents,
                key,
                &reply_header,
                &header,
                &metadata,
                &reply_content,
            )?;

            // Send status: idle
            send_iopub(
                iopub,
                key,
                "status",
                &header,
                serde_json::json!({"execution_state": "idle"}),
            )?;
        }

        _ => {
            warn!("Unhandled message type: {}", msg_type);
            // Send a generic reply
            let reply_header = make_header(&format!("{}_reply", msg_type), session);
            let metadata = serde_json::json!({});
            let reply_content = serde_json::json!({"status": "ok"});
            send_msg(
                shell,
                &idents,
                key,
                &reply_header,
                &header,
                &metadata,
                &reply_content,
            )?;
        }
    }

    Ok(())
}
