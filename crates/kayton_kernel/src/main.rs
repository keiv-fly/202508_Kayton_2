use serde_json::json;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::Parser;
use hex;
use hmac::{Hmac, Mac};
use kayton_interactive_shared::{
    InteractiveState, execute_prepared, prepare_input, set_stdout_callback_thunk,
};
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use sha2::Sha256;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "kayton_kernel",
    author,
    version,
    about = "Minimal Jupyter echo kernel"
)]
struct Args {
    #[arg(short = 'f', long = "connection-file")]
    connection_file: Option<PathBuf>,

    #[arg(long)]
    install: bool,
}

#[derive(Debug, Deserialize)]
struct ConnectionConfig {
    ip: String,
    transport: String,
    signature_scheme: String,
    key: String,
    shell_port: u16,
    iopub_port: u16,
    stdin_port: u16,
    control_port: u16,
    hb_port: u16,
}

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MessageHeader {
    msg_id: String,
    username: String,
    session: String,
    date: String,
    msg_type: String,
    version: String,
}

#[derive(Debug)]
struct ParsedMessage {
    idents: Vec<Vec<u8>>,
    signature: String,
    header_bytes: Vec<u8>,
    parent_bytes: Vec<u8>,
    metadata_bytes: Vec<u8>,
    content_bytes: Vec<u8>,
    header: Value,
    parent: Value,
    metadata: Value,
    content: Value,
    buffers: Vec<Vec<u8>>,
}

static KERNEL_SESSION: OnceLock<String> = OnceLock::new();

fn kernel_session_id() -> &'static str {
    KERNEL_SESSION.get_or_init(|| Uuid::new_v4().to_string())
}

fn base_header(msg_type: &str, _parent: &Value) -> MessageHeader {
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

fn serialize_header(h: &MessageHeader) -> Vec<u8> {
    serde_json::to_vec(h).unwrap()
}
fn empty_obj_bytes() -> Vec<u8> {
    b"{}".to_vec()
}

fn sign_bytes(key: &[u8], parts: &[&[u8]]) -> String {
    if key.is_empty() {
        return String::new();
    }
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
    for p in parts {
        mac.update(p);
    }
    let out = mac.finalize().into_bytes();
    hex::encode(out)
}

fn validate_signature(
    key: &[u8],
    header_bytes: &[u8],
    parent_bytes: &[u8],
    metadata_bytes: &[u8],
    content_bytes: &[u8],
    signature: &str,
) -> bool {
    if key.is_empty() {
        return true;
    }
    let computed = sign_bytes(
        key,
        &[header_bytes, parent_bytes, metadata_bytes, content_bytes],
    );
    computed.eq_ignore_ascii_case(signature)
}

fn send_reply(
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
        &[
            &header_bytes,
            &parent_bytes,
            &metadata_bytes,
            &content_bytes,
        ],
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
        &[
            &header_bytes,
            &parent_bytes,
            &metadata_bytes,
            &content_bytes,
        ],
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

fn publish_status(iopub: &zmq::Socket, key: &[u8], parent: &Value, state: &str) -> Result<()> {
    let content = serde_json::json!({"execution_state": state});
    publish_on_iopub(iopub, key, parent, "status", content)
}

fn publish_execute_input(
    iopub: &zmq::Socket,
    key: &[u8],
    parent: &Value,
    code: &str,
    execution_count: i32,
) -> Result<()> {
    let content = serde_json::json!({"code": code, "execution_count": execution_count});
    publish_on_iopub(iopub, key, parent, "execute_input", content)
}

fn publish_stream(
    iopub: &zmq::Socket,
    key: &[u8],
    parent: &Value,
    name: &str,
    text: &str,
) -> Result<()> {
    let content = serde_json::json!({"name": name, "text": text});
    publish_on_iopub(iopub, key, parent, "stream", content)
}

fn publish_execute_result(
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

fn parse_message(
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

fn parse_message2(frames: &[Vec<u8>]) -> Option<ParsedMessage> {
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

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

// Thread-local context to stream stdout directly from compiled code callbacks
thread_local! {
    static TLS_IOPUB: std::cell::RefCell<Option<usize>> = std::cell::RefCell::new(None);
    static TLS_KEY: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::new());
    static TLS_PARENT: std::cell::RefCell<Option<Value>> = std::cell::RefCell::new(None);
}

extern "C" fn stdout_stream_cb(text_ptr: *const u8, text_len: usize) {
    unsafe {
        let slice = core::slice::from_raw_parts(text_ptr, text_len);
        if let Ok(text) = core::str::from_utf8(slice) {
            TLS_IOPUB.with(|sock| {
                if let Some(sock_usize) = *sock.borrow() {
                    let iopub: &zmq::Socket = &*(sock_usize as *const zmq::Socket);
                    TLS_KEY.with(|k| {
                        TLS_PARENT.with(|p| {
                            if let Some(parent) = p.borrow().as_ref() {
                                let _ = publish_stream(iopub, &k.borrow(), parent, "stdout", text);
                            }
                        });
                    });
                }
            });
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if args.install {
        install_kernelspec().context("failed to install kernelspec")?;
        println!("Installed kayton_kernel kernelspec.");
        return Ok(());
    }

    let Some(connection_file) = args.connection_file else {
        eprintln!(
            "No connection file specified. Run with --install to install kernelspec, or pass -f <connection_file> to run."
        );
        std::process::exit(2);
    };

    let cfg = read_connection_file(&connection_file).context("failed to read connection file")?;

    run_kernel(&cfg)
}

fn install_kernelspec() -> Result<()> {
    let mut base = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("cannot determine data_dir"))?;
    base.push("jupyter");
    base.push("kernels");
    base.push("kayton");
    if base.exists() {
        // Uninstall existing kernelspec directory before reinstalling
        fs::remove_dir_all(&base)?;
    }
    fs::create_dir_all(&base)?;

    let exe = std::env::current_exe()?;
    let argv = vec![
        exe.to_string_lossy().to_string(),
        "-f".to_string(),
        "{connection_file}".to_string(),
    ];

    let kernel_json = json!({
        "argv": argv,
        "display_name": "Kayton",
        "language": "kayton",
        "interrupt_mode": "message",
        "env": {"RUST_LOG": "info"}
    });

    let kernel_json_path = base.join("kernel.json");
    fs::write(&kernel_json_path, serde_json::to_vec_pretty(&kernel_json)?)?;

    Ok(())
}

fn read_connection_file(path: &Path) -> Result<ConnectionConfig> {
    let mut file = fs::File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let cfg: ConnectionConfig = serde_json::from_str(&s)?;
    Ok(cfg)
}

fn run_kernel(cfg: &ConnectionConfig) -> Result<()> {
    let context = zmq::Context::new();

    {
        let ctx_hb = context.clone();
        let addr = format!("{}://{}:{}", cfg.transport, cfg.ip, cfg.hb_port);
        std::thread::spawn(move || {
            let socket = ctx_hb.socket(zmq::REP).expect("hb socket");
            socket.bind(&addr).expect("bind hb");
            loop {
                if let Ok(msg) = socket.recv_msg(0) {
                    let _ = socket.send(msg, 0);
                }
            }
        });
    }

    let iopub = context.socket(zmq::PUB)?;
    iopub.bind(&format!(
        "{}://{}:{}",
        cfg.transport, cfg.ip, cfg.iopub_port
    ))?;

    // Send initial status: starting
    let init_parent = serde_json::json!({
        "msg_id": Uuid::new_v4().to_string(),
        "username": "server",
        "session": kernel_session_id(),
        "date": now_rfc3339(),
        "msg_type": "startup",
        "version": "5.3"
    });
    let _ = publish_on_iopub(
        &iopub,
        &[],
        &init_parent,
        "status",
        serde_json::json!({"execution_state": "starting"}),
    );
    let _ = publish_on_iopub(
        &iopub,
        &[],
        &init_parent,
        "status",
        serde_json::json!({"execution_state": "idle"}),
    );

    let shell = context.socket(zmq::ROUTER)?;
    shell.bind(&format!(
        "{}://{}:{}",
        cfg.transport, cfg.ip, cfg.shell_port
    ))?;

    let control = context.socket(zmq::ROUTER)?;
    control.bind(&format!(
        "{}://{}:{}",
        cfg.transport, cfg.ip, cfg.control_port
    ))?;

    let stdin_sock = context.socket(zmq::ROUTER)?;
    stdin_sock.bind(&format!(
        "{}://{}:{}",
        cfg.transport, cfg.ip, cfg.stdin_port
    ))?;

    let key_bytes: Vec<u8> = if cfg.key.is_empty() {
        vec![]
    } else {
        cfg.key.as_bytes().to_vec()
    };
    let use_hmac = !key_bytes.is_empty() && cfg.signature_scheme.as_str() == "hmac-sha256";

    let mut execution_count: i32 = 1;
    let mut state = InteractiveState::new();
    let mut running = true;

    let mut poll_items = [
        shell.as_poll_item(zmq::POLLIN),
        control.as_poll_item(zmq::POLLIN),
    ];

    while running {
        let _ = zmq::poll(&mut poll_items, 100)?;

        if poll_items[0].is_readable() {
            let frames = shell.recv_multipart(0)?;
            if let Some(pm) = parse_message2(&frames) {
                if use_hmac
                    && !validate_signature(
                        &key_bytes,
                        &pm.header_bytes,
                        &pm.parent_bytes,
                        &pm.metadata_bytes,
                        &pm.content_bytes,
                        &pm.signature,
                    )
                {
                    warn!("Invalid HMAC on shell message, dropping");
                } else {
                    let msg_type = pm
                        .header
                        .get("msg_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    publish_status(&iopub, &key_bytes, &pm.header, "busy")?;

                    match msg_type {
                        "kernel_info_request" => {
                            let reply_content = kernel_info_content();
                            send_reply(
                                &shell,
                                &pm.idents,
                                &key_bytes,
                                &pm.header,
                                "kernel_info_reply",
                                reply_content,
                            )?;
                        }
                        "execute_request" => {
                            let code = pm
                                .content
                                .get("code")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            // Jupyter requires we echo the input first
                            publish_execute_input(
                                &iopub,
                                &key_bytes,
                                &pm.header,
                                code,
                                execution_count,
                            )?;

                            // Prepare + execute using shared interactive engine
                            let first_line_no_crlf =
                                code.trim_end_matches(&['\n', '\r'][..]).to_string();
                            let first_line_trimmed = first_line_no_crlf.trim();

                            // Handle multiline function entry: if cell starts with `fn ` and ends with ':' we store definitions
                            if first_line_trimmed.starts_with("fn ")
                                && first_line_trimmed.ends_with(':')
                            {
                                state.stored_functions.push(first_line_no_crlf);
                                // Do not execute immediately; acknowledge success
                                let reply = serde_json::json!({
                                    "status": "ok",
                                    "execution_count": execution_count,
                                    "payload": [],
                                    "user_expressions": {}
                                });
                                send_reply(
                                    &shell,
                                    &pm.idents,
                                    &key_bytes,
                                    &pm.header,
                                    "execute_reply",
                                    reply,
                                )?;
                                publish_status(&iopub, &key_bytes, &pm.header, "idle")?;
                                continue;
                            }

                            if first_line_trimmed.is_empty() {
                                let reply = serde_json::json!({
                                    "status": "ok",
                                    "execution_count": execution_count,
                                    "payload": [],
                                    "user_expressions": {}
                                });
                                send_reply(
                                    &shell,
                                    &pm.idents,
                                    &key_bytes,
                                    &pm.header,
                                    "execute_reply",
                                    reply,
                                )?;
                                execution_count += 1;
                            } else {
                                match prepare_input(&mut state, &first_line_no_crlf) {
                                    Ok(prep) => {
                                        // Install TLS context so callback can publish on this thread
                                        let iopub_ptr = (&iopub as *const zmq::Socket) as usize;
                                        TLS_IOPUB.with(|slot| {
                                            *slot.borrow_mut() = Some(iopub_ptr);
                                        });
                                        TLS_KEY.with(|slot| {
                                            *slot.borrow_mut() = key_bytes.clone();
                                        });
                                        TLS_PARENT.with(|slot| {
                                            *slot.borrow_mut() = Some(pm.header.clone());
                                        });
                                        set_stdout_callback_thunk(Some(stdout_stream_cb));

                                        // Execute synchronously (prints stream immediately via callback)
                                        let exec_result = execute_prepared(&mut state, &prep);

                                        // Clear callback and TLS after execution
                                        set_stdout_callback_thunk(None);
                                        TLS_IOPUB.with(|slot| {
                                            *slot.borrow_mut() = None;
                                        });
                                        TLS_KEY.with(|slot| {
                                            slot.borrow_mut().clear();
                                        });
                                        TLS_PARENT.with(|slot| {
                                            *slot.borrow_mut() = None;
                                        });

                                        if let Err(e) = exec_result {
                                            // Send stderr stream and error reply
                                            let err_s = e.to_string();
                                            let _ = publish_stream(
                                                &iopub, &key_bytes, &pm.header, "stderr", &err_s,
                                            );
                                            let reply = serde_json::json!({
                                                "status": "error",
                                                "ename": "ExecutionError",
                                                "evalue": err_s,
                                                "traceback": [],
                                            });
                                            send_reply(
                                                &shell,
                                                &pm.idents,
                                                &key_bytes,
                                                &pm.header,
                                                "execute_reply",
                                                reply,
                                            )?;
                                        } else {
                                            // Stdout already streamed live by callback

                                            // Format current globals (excluding __stdout) and show as display data
                                            let mut output_lines: Vec<String> = Vec::new();
                                            for (name, handle) in state.vm().snapshot_globals() {
                                                if name == "__stdout" {
                                                    continue;
                                                }
                                                let mut vm_ref = state.vm_mut();
                                                match vm_ref.format_value_by_handle(handle) {
                                                    Ok(s) => output_lines
                                                        .push(format!("{} = {}", name, s)),
                                                    Err(_) => output_lines
                                                        .push(format!("{} = <error>", name)),
                                                }
                                            }
                                            let text = if output_lines.is_empty() {
                                                String::new()
                                            } else {
                                                output_lines.join("\n")
                                            };
                                            if !text.is_empty() {
                                                let content = serde_json::json!({
                                                    "execution_count": execution_count,
                                                    "data": {"text/plain": text},
                                                    "metadata": {}
                                                });
                                                let _ = publish_on_iopub(
                                                    &iopub,
                                                    &key_bytes,
                                                    &pm.header,
                                                    "execute_result",
                                                    content,
                                                );
                                            }
                                            let reply = serde_json::json!({
                                                "status": "ok",
                                                "execution_count": execution_count,
                                                "payload": [],
                                                "user_expressions": {}
                                            });
                                            send_reply(
                                                &shell,
                                                &pm.idents,
                                                &key_bytes,
                                                &pm.header,
                                                "execute_reply",
                                                reply,
                                            )?;
                                        }
                                        state.input_counter += 1;
                                        execution_count += 1;
                                    }
                                    Err(e) => {
                                        let err_s = e.to_string();
                                        let _ = publish_stream(
                                            &iopub, &key_bytes, &pm.header, "stderr", &err_s,
                                        );
                                        let reply = serde_json::json!({
                                            "status": "error",
                                            "ename": "TypeError",
                                            "evalue": err_s,
                                            "traceback": [],
                                        });
                                        send_reply(
                                            &shell,
                                            &pm.idents,
                                            &key_bytes,
                                            &pm.header,
                                            "execute_reply",
                                            reply,
                                        )?;
                                        state.input_counter += 1;
                                        execution_count += 1;
                                    }
                                }
                            }
                        }
                        _ => {
                            let reply = serde_json::json!({});
                            let reply_type =
                                format!("{}_reply", msg_type.trim_end_matches("_request"));
                            let reply_type_str = if reply_type.ends_with("_reply") {
                                reply_type.as_str()
                            } else {
                                "unknown_reply"
                            };
                            send_reply(
                                &shell,
                                &pm.idents,
                                &key_bytes,
                                &pm.header,
                                reply_type_str,
                                reply,
                            )?;
                        }
                    }

                    publish_status(&iopub, &key_bytes, &pm.header, "idle")?;
                }
            }
        }

        if poll_items[1].is_readable() {
            let frames = control.recv_multipart(0)?;
            if let Some(pm) = parse_message2(&frames) {
                if use_hmac
                    && !validate_signature(
                        &key_bytes,
                        &pm.header_bytes,
                        &pm.parent_bytes,
                        &pm.metadata_bytes,
                        &pm.content_bytes,
                        &pm.signature,
                    )
                {
                    warn!("Invalid HMAC on control message, dropping");
                } else {
                    let msg_type = pm
                        .header
                        .get("msg_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match msg_type {
                        "shutdown_request" => {
                            let restart = pm
                                .content
                                .get("restart")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let reply = serde_json::json!({"restart": restart});
                            send_reply(
                                &control,
                                &pm.idents,
                                &key_bytes,
                                &pm.header,
                                "shutdown_reply",
                                reply,
                            )?;
                            running = false;
                        }
                        _ => {
                            let reply = serde_json::json!({});
                            let reply_type =
                                format!("{}_reply", msg_type.trim_end_matches("_request"));
                            let reply_type_str = if reply_type.ends_with("_reply") {
                                reply_type.as_str()
                            } else {
                                "unknown_reply"
                            };
                            send_reply(
                                &control,
                                &pm.idents,
                                &key_bytes,
                                &pm.header,
                                reply_type_str,
                                reply,
                            )?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn kernel_info_content() -> Value {
    serde_json::json!({
        "protocol_version": "5.3",
        "implementation": "kayton_kernel",
        "implementation_version": env!("CARGO_PKG_VERSION"),
        "language_info": {
            "name": "kayton",
            "version": "0.1.0",
            "mimetype": "text/plain",
            "file_extension": ".kay"
        },
        "banner": "Kayton Kernel - interactive Kayton execution",
        "help_links": []
    })
}
