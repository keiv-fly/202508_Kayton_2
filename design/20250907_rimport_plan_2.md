## rimport and Environments Plan (dylib + Rust ABI)

### Objective

Enable dynamic import of Rust crates into Keyton using:

```
rimport reqwest
from reqwest rimport Client, StatusCode
```

Imported items become usable in Jupyter/REPL cells. At runtime we:
- Build/load a plugin DLL for the crate (crate-type = "dylib").
- Use the Kayton VM API (`kayton_api`) to register dynamic kinds, functions, and pre-instantiated generics.
- Generate Rust that links to the plugin via Rust ABI and calls its exported wrappers.

Constraints and choices:
- Use `dylib` (not `cdylib`).
- Use the unstable Rust ABI across the DLL boundary (`extern "Rust"`). Pin `rustc` toolchain/version and target triple to guarantee ABI compatibility between the plugin and the generated temp crate.
- Windows is primary; ensure DLL preloading works with absolute paths.

---

### Architecture Overview

1) Parser/typing extensions (Keyton compiler):
- Parse `rimport <crate>` and `from <crate> rimport A, B, ...` into import descriptors.
- Thread imports through SHIR/THIR so codegen knows which modules/symbols come from plugins.
- Represent imported items as external symbols that resolve to a plugin crate path and exported Rust items.

2) Plugin crate generator (on-demand):
- For a requested Rust crate (e.g., `reqwest`) and selected items, generate a wrapper crate `kayton_rplugin__<crate>` with:
  - `[lib] crate-type = ["dylib"]` (not `cdylib`).
  - Dependencies: `kayton_api` and the target crate (e.g., `reqwest`).
  - `#[no_mangle] pub extern "Rust" fn kayton_plugin_register(ctx: &mut kayton_api::KaytonContext)` which registers types/functions with the VM.
  - A stable Rust module surface `pub mod exports` that re-exports requested items and exposes wrapper functions for generics/methods used from Keyton.
- Build the plugin once per (crate name, version, feature set, requested exports) tuple and cache it under `target/kayton_plugins/<crate>/<hash>/` (or inside the active environment; see Environments).

3) Host loading & registration flow (interactive):
- Before compiling the user cell, ensure all required plugins exist and are up to date; build missing ones (using the active environment’s toolchain and registry).
- Preload each plugin `.dll` with `libloading` using absolute paths.
- Call `kayton_plugin_register` from each plugin, passing a `KaytonContext` to let the plugin:
  - Register dynamic kinds for its opaque types (e.g., `reqwest::Client`).
  - Publish function pointers or thunks via VM globals when a dynamic fallback is required.
- After plugin registration, compile and load the cell’s temporary `dylib` and call `run()` (existing flow).

4) Codegen for rimported calls:
- Generate Rust that links to plugin wrappers via normal Rust item paths (Rust ABI), not raw symbol lookup:
  - Add `[dependencies]` entries pointing at the plugin crates (by absolute `path` into the environment), and `use kayton_rplugin__<crate>::exports::*;` in the generated lib.
  - Calls like `Client::new()` or `StatusCode::OK` resolve into the plugin’s `exports` module (which re-exports/wraps the real crate).
- For operations that need the VM (e.g., persisting/retrieving across cells), call plugin wrapper functions that internally use the `KaytonContext` set during registration.

---

### Plugin ABI and Surface

Required exports in every plugin crate (`dylib`, Rust ABI):
- `#[no_mangle] pub extern "Rust" fn kayton_plugin_register(ctx: &mut KaytonContext)`
  - Responsibilities:
    - Register dynamic kinds for opaque types the plugin wants to persist across cells.
    - Publish named globals in the VM for dynamic-call fallbacks or state handles if required.
    - Optionally publish a manifest JSON string describing exported items and metadata.

- `pub mod exports { ... }` (Rust module, not a C ABI surface)
  - `pub use real_crate::{RequestedType, RequestedEnum, ...};`
  - Thin `pub` wrappers for functions and methods used from Keyton, with signatures consumable by generated code.
  - Where VM handles are needed, provide `extern "Rust"` wrappers that take `&mut KaytonContext` and marshal to/from dynamic kinds.

Notes:
- Using Rust ABI across DLLs requires consistent rustc + target + dependency versions. We will pin and verify these at runtime by embedding build metadata in the plugin and in the temp crate, and failing fast on mismatches.
- We avoid `cdylib` so Rust symbol names/types are preserved for the dependent generated crate to link against.

---

### Generics Strategy (initial shapes)

Scope for pre-instantiation: `i64`, `f64`, `&str`, `String`, `Vec<i64>`, `Vec<f64>`, and an opaque `Dynamic` handle.

In the plugin:
- For generic fns/types intended for Keyton, create concrete wrapper functions/types per shape, e.g.:
  - `pub fn vec_i64_new() -> Vec<i64>`
  - `pub fn vec_f64_push(v: &mut Vec<f64>, x: f64)`
- Where a value must cross the VM boundary or be persisted, export `extern "Rust"` wrappers that convert to/from VM handles using dynamic kinds.
- Document and re-export these in `exports` so codegen can call them by name without reflection.

---

### Compiler Changes (Keyton)

1) Parser:
- Extend grammar with `rimport` statements and a `from ... rimport ...` form.

2) SHIR/Resolver:
- Introduce symbol kinds for external modules/items.
- Gather all rimports into a `RequiredPlugins` collection with per-crate lists of requested items and generic instantiations.

3) THIR/Typecheck:
- Map external items to either concrete Rust types (when safe to expose) or to an internal `Dynamic` opaque type tracked via VM handles.
- For MVP, allow direct use of simple re-exported Rust types (e.g., enums like `StatusCode`) and treat complex structs as `Dynamic` unless explicitly supported.

4) Rust codegen:
- Accept an `ExtraDeps` list: `[(plugin_crate_name, path)]`.
- Emit `Cargo.toml` with `[lib] crate-type = ["dylib"]` (unchanged for the temp crate) and add plugin crates under `[dependencies]` (absolute `path` into the active environment).
- Insert `use kayton_rplugin__<crate>::exports::*;` at the top of the generated lib, and compile calls/paths accordingly.
- Provide a dynamic-call fallback helper only when needed.

---

### Host Changes (kayton_interactive_shared)

1) Import planning:
- During `prepare_input`, after resolve/typecheck, collect `RequiredPlugins` and the set of exported items used by the cell.

2) Plugin build/cache (delegated to active environment):
- For each required plugin key `(crate, version, features, exports)`, ensure a generated plugin crate is present under the active environment and build its `dylib`.
- If missing or stale, generate its `Cargo.toml` and `src/lib.rs`, then build with the environment’s pinned toolchain.

3) Preload + register:
- Before loading the cell `dylib`, call `Library::new` on each plugin DLL path (absolute), then look up and call `kayton_plugin_register` with a `KaytonContext`.

4) Compile cell with plugin deps:
- Call `compile_generated_rust_to_dylib_with_deps(source, ExtraDeps)` which writes plugin `[dependencies]` into the temp crate’s `Cargo.toml` using absolute paths into the active environment.

5) Execution order:
- Preload/register plugins → build/load cell `dylib` → set reporters → `run()`.

---

### Windows DLL Details

- Build plugin as `dylib`; filename will be derived from the crate name.
- Preload each plugin using an absolute path with `libloading::Library::new` before loading the cell `dylib`. On Windows this ensures the loader can resolve the plugin imports when the cell `dylib` is loaded.
- Optionally copy the plugin DLL next to the compiled temp DLL to satisfy the default search path if preloading is not used.

---

### Versioning, Safety, and Diagnostics

- Rust ABI across DLLs is not stable. Enforce:
  - Pin toolchain channel and exact `rustc -VV`.
  - Embed a metadata string (compiler version, target, crate graph hash) into both plugin and generated crate. During `kayton_plugin_register`, validate compatibility with the host and reject on mismatch.
- Wrap plugin wrapper bodies in `std::panic::catch_unwind` when crossing the dynamic boundary; convert panics to a VM error or abort with a clear message.
- Provide a `kayton_plugin_manifest()` JSON export listing:
  - plugin name, version, rustc info
  - exported types/functions and generic instantiations

---

### Environments and `kik` CLI

Goal: Hermetic per-project runtime containing Kayton, pinned toolchain, installed rimport libraries (with sources), and compiled plugin DLLs. Managed by the `kik` command.

Default base location (similar to local conda envs):
- Windows: `%USERPROFILE%\\.kayton\\envs\\<env-name>`
- Unix: `$HOME/.kayton/envs/<env-name>`

Environment layout (proposed):
- `env.toml` — metadata: env name, platform triple, kayton version, rustc info (from `rustc -VV`), creation time.
- `toolchain/` — pinned toolchain marker, e.g., `rust-toolchain.toml` and optional `rustup` override files.
- `bin/` — `kayton.exe`, `kayton_kernel.exe`, `kik.exe` shims for this environment.
- `crates-src/<crate>/<version>/` — full unpacked Rust source for each installed crate (vendored from crates.io).
- `plugins/<crate>/<hash>/` — generated wrapper crate `kayton_rplugin__<crate>` with `[lib] crate-type = ["dylib"]`.
- `dlls/<crate>/<hash>/` — built plugin DLL outputs.
- `cargo/` — optional per-env Cargo home/registry/cache to ensure reproducible builds offline.

`kik` commands:
- `kik create <env>`: create environment directory structure; record `env.toml`; pin toolchain; install Kayton binaries into `bin/`.
- `kik activate <env>`: activate environment.
  - Windows PowerShell: emit and/or execute an activation script `Activate.ps1` that sets `KAYTON_ENV`, prepends `<env>/bin` to `PATH`, sets `CARGO_HOME` and `RUSTUP_TOOLCHAIN` (or uses `rust-toolchain.toml`).
  - Implementation detail: `kik activate` writes to stdout the necessary `Set-Item` commands; users can run `kik activate <env> | Invoke-Expression` if not installed as a profile function. When `kik` is a `.ps1` shim, it can mutate session env directly.
- `kik rinstall <crate>[==<semver>] [--features ...]`:
  - Resolve version (crates.io), fetch crate tarball, vendor into `crates-src/<crate>/<version>/`.
  - Synthesize wrapper plugin crate under `plugins/<crate>/<hash>/` (hash covers version, features, requested exports policy).
  - Build `dylib` plugin with env’s toolchain; place DLL in `dlls/<crate>/<hash>/`.
  - Record entry in `env.toml` (or `packages.toml`): name, version, features, hash, DLL path, plugin manifest, and source path.
- `kik uninstall <crate>`: remove metadata entry; optionally prune `dlls/` and `plugins/`/`crates-src/` trees if no other references (leave tarballs/caches if desired).
- `kik list`: list Kayton version, toolchain, target triple, and all installed libraries with versions and installation hashes.
- `kik kernel install [--name <display-name>]`:
  - Install a Jupyter kernel spec that points to this environment’s `kayton_kernel.exe`.
  - Default name: `Kayton (<env>)`. Custom name allowed via `--name`.
  - Write `kernel.json` to the user kernels directory (Windows: `%APPDATA%\\jupyter\\kernels\\kayton-<env>`). Ensure `argv` references `<env>/bin/kayton_kernel.exe` (and includes `--env <env>` if needed).

Runtime use with environments:
- The interactive host consults the active environment to resolve plugin crate paths for codegen and to locate DLLs for preloading.
- If a crate is referenced by `rimport` but not installed in the active environment, surface a diagnostic inviting `kik rinstall <crate>` (or trigger an on-demand install with user opt-in).

Version/ABI management per environment:
- Each environment stores the exact `rustc -VV`, target triple, and Kayton version. `kik` enforces that plugin builds and ephemeral cell builds use the same toolchain.
- On load, `kayton_plugin_register` validates the metadata in the DLL against the host and env; mismatch → clear error with remediation steps.

Source retention requirement:
- The environment MUST retain the exact Rust sources for each installed library. `kik rinstall` vendors crates into `crates-src/` and records checksums.

---

### Generating Plugin Crates (in environments)

Inputs:
- Crate name (e.g., `reqwest`), resolved version/feature set from the environment.
- Items to export (types, functions), and list of generic instantiations (MVP may export a curated subset or re-export entire public items where safe).

Outputs per `(crate, version, features, exports)` key:
- `Cargo.toml` (dylib), `src/lib.rs` with:
  - `exports` module: `pub use`/wrapper functions for direct Rust calls from generated code.
  - `kayton_plugin_register` that registers dynamic kinds and publishes any dynamic-call helpers.
  - Optional `kayton_plugin_manifest` for diagnostics.

Caching and layout (inside env):
- `plugins/<crate>/<hash>/` wrapper sources; `dlls/<crate>/<hash>/<plugin>.dll` binaries.
- Hash covers version + features + exports policy + toolchain metadata.

---

### Codegen Examples (conceptual)

- User cell:
  - `rimport reqwest` → generated Rust: `use kayton_rplugin__reqwest::exports::*;`
  - `from reqwest rimport Client, StatusCode` → generated Rust references `Client`, `StatusCode` from `exports`.

- Plugin `exports` (sketch):
  - `pub use reqwest::StatusCode;`
  - `pub use reqwest::blocking::Client;`
  - `pub fn client_new() -> Client { Client::new() }`
  - For dynamic fallback: `pub extern "Rust" fn client_new_dyn(ctx: &mut KaytonContext) -> HKayRef { /* allocate, register, return handle */ }`

---

### Milestones & Tasks

1) Compiler front-end
- Parse/resolve `rimport` forms; produce `RequiredPlugins`.

2) Environment + `kik` CLI
- Implement `kik create/activate` with env layout and toolchain pinning.
- Implement `kik rinstall/uninstall/list` with source vendoring and plugin build to `dylib`.
- Implement `kik kernel install` with custom display name.

3) Plugin generator
- Emit `dylib` crate (Cargo.toml + lib.rs) with `exports` and `kayton_plugin_register`.
- Implement generic instantiation templates for the initial shapes.

4) Host integration
- Build/cache plugins in the active environment; preload and invoke `kayton_plugin_register`.
- Extend compile pipeline: pass plugin deps to the temp crate and `use` their `exports`.

5) Diagnostics & compatibility
- Embed and validate build metadata; fail fast on ABI mismatches.
- Helpful error messages when a symbol is missing or a plugin fails to load.

6) Tests (Windows focus)
- Unit: plugin generator output; registration calls; dynamic kind drop fns; env metadata parsing.
- Integration: `rimport reqwest`; call simple functions; ensure DLL preloading works from env; `kik` flows (create/activate/rinstall/list/uninstall; kernel install registers kernel spec).

---

### Risks and Mitigations

- Rust ABI across DLLs: Pin toolchain; validate metadata; keep plugin and temp crate in lockstep; use `dylib` not `cdylib`.
- Windows DLL discovery: Preload via absolute paths; optionally copy plugin DLLs next to the temp DLL.
- Generic explosion: restrict to explicit shapes in MVP; expand incrementally.
- Performance: cache built plugins per environment; reuse across sessions when metadata matches.
- Activation reliability on PowerShell: provide a `.ps1` activator that can be dot-sourced; document `| Invoke-Expression` fallback.
