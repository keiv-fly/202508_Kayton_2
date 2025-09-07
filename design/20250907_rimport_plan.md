## rimport and Environments: High-Level Design (2025-09-07)

This plan enables importing Rust crates at runtime in Kayton using `rimport`, backed by environment-managed, versioned DLL plugins compiled as `dylib` with the unstable Rust ABI.

### Goals

- **Runtime crate import**: Load Rust crates as DLL plugins via `rimport` and `from ... rimport ...`.
- **VM registration**: Plugins register types, functions, and selected generic instantiations into the Kayton VM via `kayton_api`.
- **Compiler integration**: Keyton codegen resolves imported items to function/type pointers fetched at startup; execution uses those pointers only.
- **Environment management**: `kik` CLI creates/activates environments, installs/uninstalls crate plugins, lists contents, and installs Jupyter kernels from an environment. Environments can be global or project-local (".venv"-like).

### Non-goals (initial increment)

- Full Rust trait resolution or blanket generics across the boundary.
- Macro import. Prefer function exports or codegen templates.
- Auto-install on `rimport`. Missing libraries should error with guidance to use `kik`.

## Core Decisions and Constraints

- **Plugin artifact**: Rust crate compiled as `dylib` (not `cdylib`).
- **ABI**: Use `extern "Rust"` (unstable Rust ABI) for plugin entrypoints and exported function pointers.
  - Enforced by pinning a nightly toolchain per environment (`rust-toolchain.toml`).
  - ABI compatibility is scoped to the environment’s pinned `rustc` and `kayton_api` versions.
- **Safety boundary**: All cross-boundary data uses `kayton_api` types and handles. Opaque Rust types (e.g., `reqwest::Client`) are stored via dynamic kinds in the VM’s dynamic store with plugin-provided drop functions.
- **Generics**: Export named, pre-instantiated variants for a curated set: `i64`, `f64`, `&str`, `String`, `Vec<i64>`, `Vec<f64>`, and a `Dynamic` opaque handle.
- **Error policy**: `rimport X` fails if `X` is not present in the active environment. The recommended flow is `kik rinstall X` before execution.

## High-Level Architecture

### Components

- **kayton_api**: `no_std` VM bridge; add a lightweight registry for plugin-declared functions and types, keyed by stable strings.
- **kayton_vm**: Host-side storage for dynamic kinds and a new function registry; manages symbol and type metadata lifecycles.
- **kayton_interactive_shared**: Runtime that loads compiled user code and calls `run()`. It will also preflight rimport presence checks against the active environment and orchestrate plugin loading prior to `run()` when requested by generated code.
- **keyton_rust_compiler**: Frontend parsing and typechecking for `rimport`; codegen to:
  1) Load required plugins at program start,
  2) Fetch function/type pointers from the VM registry,
  3) Use only those pointers in main execution.
- **kik** (new CLI): Environment manager and plugin builder/installer.
- **kayton_plugin_sdk** (new crate): Helper macros/types for defining exported functions/types in plugin wrappers and emitting a manifest.

### Plugin contract (DLL)

Every plugin exports a minimal, versioned surface using Rust ABI:

- `#[no_mangle] pub extern "Rust" fn kayton_plugin_abi_version() -> u32`
- `#[no_mangle] pub extern "Rust" fn kayton_plugin_manifest_json() -> &'static [u8]`
- `#[no_mangle] pub extern "Rust" fn kayton_plugin_register(ctx: &mut KaytonContext)`

Where:
- `kayton_plugin_abi_version` must match the host’s expected `KAYTON_PLUGIN_ABI_VERSION`.
- `kayton_plugin_manifest_json` returns a JSON blob describing exported items (functions, types, generic instantiations) with stable names and signatures.
- `kayton_plugin_register` uses `kayton_api` to:
  - Register types (size, align, drop, clone, optional to_string/debug) and assign them stable string IDs.
  - Register function pointers with stable names and a compact signature descriptor (for compiler/typechecker validation).
  - Register generic instantiations as distinct, named entries (e.g., `map_i64`, `map_f64`).

Notes:
- Returning `&'static [u8]` keeps memory owned by the plugin. Avoids cross-allocator transfer.
- All exported functions use Rust ABI and must only accept/return `kayton_api`-compatible types or dynamic handles registered by the plugin.

## rimport Language Semantics

- Syntax examples:
  - `rimport reqwest`
  - `from reqwest rimport Client, StatusCode`
  - Optional aliasing (later increment): `rimport serde_json as json`
- Resolution rules:
  1) Identify the active environment (global or local project). If none, use the base environment.
  2) Check for presence of the plugin and compatible `kayton_api`/ABI versions.
  3) Load plugin manifest for compile-time name binding and typechecking.
- Error handling:
  - If plugin is missing: raise an import error with remediation: `Run: kik rinstall <crate>`.
  - If ABI or `kayton_api` version mismatches: error with remediation: upgrade/downgrade environment or reinstall plugin.

## Compiler and Codegen Changes (keyton_rust_compiler)

1) **Parsing and HIR**: Add `rimport` AST nodes. Record module name and optional item lists.
2) **Name resolution**: Use the active environment’s plugin manifests to resolve imported names. Bind types/functions to stable, fully-qualified plugin names.
3) **Typechecking**: Validate function signatures from the manifest against usage in Keyton code. For generics, allow only pre-exposed instantiations from the manifest.
4) **Lowering**: Emit IR that carries a list of required plugins and a list of required function/type symbol names per plugin.
5) **Rust codegen**:
   - At program start:
     - For each required plugin, dynamically open the DLL from the environment path.
     - Look up and verify `kayton_plugin_abi_version`.
     - Call `kayton_plugin_register(&mut ctx)`.
     - For each required function/type, call VM `get_function_ptr(name)` / `get_type_meta(name)`.
     - Store pointers in local `static mut` or stack locals wrapped in strongly typed newtypes.
   - In main body:
     - Invoke through the retrieved function pointers.

Implementation detail:
- Generate Rust declarations with `extern "Rust" fn` signatures matching the plugin exports. Wrap raw pointers in safe structs; perform minimal `unsafe` at the boundary only.

## VM and API Extensions (kayton_vm, kayton_api)

- **Function registry**:
  - `register_function(name: &str, raw_ptr: RawFnPtr, sig_id: u64)`
  - `get_function(name: &str) -> RawFnPtr`
  - Optional `get_function_by_id(sig_id: u64, name: &str)` for faster lookup.
- **Type registry**:
  - `register_type(name: &str, meta: TypeMeta)` where `TypeMeta` includes size, align, layout tag, droppers/cloners, and optional stringifier.
  - `get_type(name: &str) -> TypeMeta`
- **Dynamic kinds**:
  - Keep existing dynamic ptr store; add helpers to declare opaque kinds with stable names and drop fns.
- **Safety**:
  - All registration copies only POD metadata into the VM. Function pointers remain callable via Rust ABI. Opaque state lives on the plugin side; VM only stores droppers and names.

## Environments and kik CLI

### Environment model

- One active environment in a session, discovered in order:
  1) Explicit activation (`kik activate <name>` sets `KAYTON_ACTIVE_ENV`).
  2) Project-local (".venv"-like) directory in CWD: `.kayton/` when activated with `kik activate local`.
  3) Base environment (global default).
- Until explicitly asked, do not create new environments; always use the currently activated one.

### On-disk layout (Windows paths shown)

```
%LOCALAPPDATA%\Kayton\envs\<env-name>\
  bin\                # kayton, kik, tools (this env’s shims)
  toolchain\          # rust-toolchain.toml (nightly pinned), cargo config
  libs\<crate>\<ver>\<target_triple>\
    plugin.dll        # compiled dylib plugin
    manifest.json     # plugin manifest snapshot
    Cargo.toml
    Cargo.lock
    vendor\...        # full vendored sources
  metadata\
    env.json          # kayton version, abi version, target triple
    registry.json     # installed plugins index
  kernels\
    <kernel_name>\kernel.json
```

Project-local (".venv"-like):

```
<project>\.kayton\  # same structure as above (can be lighter weight), activated per-project
```

### kik commands

- `kik create <name>`: Create environment structure; pin `rust-toolchain.toml`; record Kayton version.
- `kik activate <name>`: Activate environment for current shell (sets `KAYTON_ACTIVE_ENV`, prepends env `bin` to `PATH`).
- `kik create local` + `kik activate local`: Create/activate `.kayton` in current project.
- `kik rinstall <crate> [--version <semver>] [--features ...]`:
  - Resolve version (default latest compatible).
  - Generate wrapper crate `kayton_plugin_<crate>` using `kayton_plugin_sdk` macros.
  - Set `crate-type = ["dylib"]`, enable nightly and `#![feature(abi_rust)]`.
  - `cargo vendor` the full sources; `cargo build --release` for the env’s target.
  - Write `manifest.json` from plugin build output and update `registry.json`.
- `kik uninstall <crate>`: Remove plugin entry and artifacts; keep cached sources optionally.
- `kik list`: Show installed libraries with versions; also show Kayton version and ABI/toolchain summary for the env.
- `kik kernel install [-n <kernel_name>]`: Install a Jupyter kernel using this env’s `kayton_kernel` and config. Allows custom kernel name; registers to Jupyter kernelspecs.

### Base environment

- Created during initial install or first `kik create base`.
- Used whenever no local or named env is activated.

## Execution Flow

1) User installs library: `kik rinstall reqwest` (into the active environment).
2) In a notebook/cell, user writes:
   - `rimport reqwest`
   - `from reqwest rimport Client, StatusCode`
3) Kernel receives code. Before compilation, it checks the active environment for plugin presence; if missing, returns an import error suggesting `kik rinstall reqwest`.
4) Compiler consumes manifests to bind names and typecheck.
5) Generated Rust, when run:
   - Loads `reqwest` plugin DLL.
   - Validates ABI/version, calls `kayton_plugin_register`.
   - Fetches required function/type pointers from VM.
   - Executes using only those pointers.

## Plugin Wrapper Strategy (reqwest example)

- Types: Export opaque `Client` and `Response` as dynamic kinds; export `StatusCode` as a small value type.
- Functions:
  - `Client::new() -> Client` (opaque handle)
  - `Client::get(&Client, &str) -> Dynamic(Response)`
  - `Response::status(&Response) -> StatusCode`
  - `StatusCode::as_u16(StatusCode) -> u16`
- Data exchange: Use `KaytonApi` strings and `KVec<u8>` for bodies; avoid crossing allocators with Rust-native `Vec` from user code.
- Generics: Export a handful of helper functions for basic conversions; expand over time.

## Compatibility and Versioning

- Define `KAYTON_PLUGIN_ABI_VERSION` and `kayton_api` crate version gates. Loader rejects mismatches with actionable errors.
- Environments pin `rustc` nightly and store `Cargo.lock` + vendored sources for reproducible rebuilds.
- Plugin manifests include crate name, version, target triple, and exported items.

## Testing and Validation

- Unit and integration tests:
  - Loader: open DLL, check version handshake, call register, query functions/types.
  - Compiler: parse `rimport`, resolve against a test manifest, generate startup pointer fetches.
  - End-to-end: `plugin_hello` and a minimal `reqwest` wrapper.
- Windows CI focus (DLLs, PowerShell). Later add Linux/macOS coverage for portability.

## Milestones

- **M0**: `kik` scaffolding, env layout, activate/create/list. Base env bootstrap.
- **M1**: `kayton_plugin_sdk`, loader handshake (`abi_version`, `manifest_json`, `register`). `plugin_hello` converted to `dylib` with Rust ABI.
- **M2**: rimport parsing/resolution; compile-time checks against manifests; codegen for startup loading and pointer retrieval.
- **M3**: Function/type registry in VM/API; safe wrappers; dynamic kinds polish.
- **M4**: `kik rinstall/uninstall` full pipeline (generate wrapper crate, vendor, build, record manifest/source).
- **M5**: `kik kernel install` with custom names; docs and examples.
- **M6**: First external crate (`reqwest`) minimal surface; sample notebook.

## Error Messages (examples)

- Import missing:
  - `ImportError: Library 'reqwest' is not installed in the active Kayton environment 'k01'. Run: kik rinstall reqwest`
- ABI mismatch:
  - `PluginABIError: 'reqwest' built with ABI v3 but host expects v4. Reinstall plugin or switch environment.`

---

This document scopes the first iteration to Rust-ABI `dylib` plugins, environment-pinned toolchains, and explicit exported surfaces. It prioritizes safety via `kayton_api` types, controlled generics, and robust environment/version management via `kik`.


