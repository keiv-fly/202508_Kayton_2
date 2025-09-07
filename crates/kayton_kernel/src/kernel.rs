use anyhow::Result;
use kayton_interactive_shared::{
    execute_prepared, prepare_input, set_stdout_callback_thunk, InteractiveState,
};
use log::warn;
use serde_json::{self, json, Value};
use uuid::Uuid;

use crate::config::ConnectionConfig;
use crate::iopub::{
    publish_execute_input, publish_execute_result, publish_status, publish_stream, send_reply,
};
use crate::protocol::{kernel_session_id, now_rfc3339, parse_message2};
use crate::signing::validate_signature;

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

pub fn run_kernel(cfg: &ConnectionConfig) -> Result<()> {
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
    iopub.bind(&format!("{}://{}:{}", cfg.transport, cfg.ip, cfg.iopub_port))?;

    // Send initial status: starting then idle
    let init_parent = json!({
        "msg_id": Uuid::new_v4().to_string(),
        "username": "server",
        "session": kernel_session_id(),
        "date": now_rfc3339(),
        "msg_type": "startup",
        "version": "5.3",
    });
    let _ = publish_status(&iopub, &[], &init_parent, "starting");
    let _ = publish_status(&iopub, &[], &init_parent, "idle");

    let shell = context.socket(zmq::ROUTER)?;
    shell.bind(&format!("{}://{}:{}", cfg.transport, cfg.ip, cfg.shell_port))?;

    let control = context.socket(zmq::ROUTER)?;
    control.bind(&format!("{}://{}:{}", cfg.transport, cfg.ip, cfg.control_port))?;

    let stdin_sock = context.socket(zmq::ROUTER)?;
    stdin_sock.bind(&format!("{}://{}:{}", cfg.transport, cfg.ip, cfg.stdin_port))?;

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
                                    "user_expressions": {},
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
                                    "user_expressions": {},
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

                                            // Publish only the value of the last expression (if any)
                                            if let Some(h) = state.vm().resolve_name("__last") {
                                                let mut vm_ref = state.vm_mut();
                                                if let Ok(text) = vm_ref.format_value_by_handle(h) {
                                                    if !text.is_empty() {
                                                        publish_execute_result(
                                                            &iopub,
                                                            &key_bytes,
                                                            &pm.header,
                                                            execution_count,
                                                            &text,
                                                        )?;
                                                    }
                                                }
                                            }
                                            let reply = serde_json::json!({
                                                "status": "ok",
                                                "execution_count": execution_count,
                                                "payload": [],
                                                "user_expressions": {},
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
