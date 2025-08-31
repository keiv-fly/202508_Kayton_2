# Kayton Jupyter Kernel

This crate provides a Jupyter kernel for the Kayton language. It uses the shared interactive engine to execute code and persists globals across cells.

## Build

- From the workspace root:

```
# Debug build
cargo build -p kayton_kernel

# Release build
cargo build -p kayton_kernel --release
```

During build, a minimal Jupyter kernel spec is generated at:

- target/<profile>/kayton_kernelspec/kayton/kernel.json

The kernelspec points to an executable named `kayton_kernel` on your PATH, and uses the standard `-f {connection_file}` CLI for Jupyter connection files.

## Install the kernel into Jupyter

Option A: Using Jupyter’s install command with a directory

```
jupyter kernelspec install --user target/debug/kayton_kernelspec/kayton --name kayton
# Or release
jupyter kernelspec install --user target/release/kayton_kernelspec/kayton --name kayton
```

Option B: Copy the spec manually (advanced)

- Find your Jupyter data dir: `jupyter --data-dir`
- Create a directory: `<data-dir>/kernels/kayton`
- Copy `kernel.json` there
- Ensure `kayton_kernel` executable is on your PATH

## Use

- Launch JupyterLab or Notebook and select the “Kayton” kernel when creating a new notebook.

### Optional: Full Jupyter protocol

- Build with the `jupyter` feature to enable ZeroMQ/HMAC protocol wiring (placeholder module included; extend as needed):

```
cargo build -p kayton_kernel --features jupyter
```

- The generated `kernel.json` already uses the `-f {connection_file}` argument that Jupyter provides.

## Notes

- The current kernel is a minimal implementation that executes cell source via the Kayton interactive engine and returns a JSON summary of globals. Full ZeroMQ-based Jupyter message handling can be added next.
