# Kayton Echo Jupyter Kernel

This crate provides a standalone Jupyter kernel that echoes the input back as output. It is not related to, nor does it depend on, any other crates in this workspace.

## Build

- From the workspace root or this crate directory:

```
# Debug build
cargo build -p kayton_kernel

# Release build
cargo build -p kayton_kernel --release
```

## Install the kernel into Jupyter

Use the built-in installer (writes a kernelspec with display name "Kayton Echo"):

```
# From this crate directory
./target/debug/kayton_kernel.exe --install
# Or release
./target/release/kayton_kernel.exe --install

# Verify registration
jupyter kernelspec list
```

This installs the kernelspec under your user Jupyter data directory (on Windows typically `%APPDATA%/jupyter/kernels/kayton_kernel`). The kernel is registered with the name `kayton_kernel` and a display name of "Kayton Echo".

To uninstall later:

```
jupyter kernelspec uninstall -y kayton_kernel
```

## Use

- Jupyter console:

```
jupyter console --kernel kayton_kernel
```

- JupyterLab/Notebook: create a new notebook and select the "Kayton Echo" kernel.

### What to expect

- The kernel does not execute code; it simply echoes the cell's input.
- For an `execute_request`, it publishes the input on stdout and as an `execute_result`, and returns an `execute_reply` with status `ok`.

## How it works (protocol)

- Uses the standard Jupyter kernel messaging protocol over ZeroMQ.
- Channels and socket types:
  - Shell, Control, Stdin: ROUTER
  - IOPub: PUB
  - Heartbeat: REP (echoes frames)
- Message signing: HMAC-SHA256 when a non-empty key is provided by the connection file.
- WebSockets: Browsers talk WebSockets to Jupyter Server, which bridges to the kernel's ZeroMQ sockets. No WebSocket code is required in the kernel; it works seamlessly in both cases.

## Kernelspec details

- The installer writes `kernel.json` with `argv` like:
  - `["<path-to-exe>", "-f", "{connection_file}"]`
- Environment: sets `RUST_LOG=info` for basic logging.

## Troubleshooting

- Kernel not listed: run `jupyter kernelspec list` and ensure `kayton_kernel` appears. Re-run `--install` if needed.
- Kernel fails to start: check the installed `kernel.json` path and that the executable exists. Increase logging by setting `RUST_LOG=debug` and restart the kernel.
