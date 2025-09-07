## rimport: High‑level Plan (dylib + Rust ABI)

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
- Use the unstable Rust ABI across the DLL boundary (`extern "Rust"`). Pin rustc toolchain/version and target triple to guarantee ABI compatibility between the plugin and the generated temp crate.
- Windows is primary (ensure DLL search/load works via explicit preloading).

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
  - A stable Rust module surface `pub mod exports` that re-exports the requested items and exposes wrapper fns for generics and method calls used from Keyton.
- Build the plugin once per (crate name, version, feature set, requested exports) tuple and cache it under `target/kayton_plugins/<crate>/<hash>/`.

3) Host loading & registration flow (interactive):
- Before compiling the user cell, ensure all required plugins exist and are up to date; build missing ones.
- Preload each plugin `.dll` with `libloading` using absolute paths.
- Call `kayton_plugin_register` from each plugin, passing a `VmKaytonContext` to let the plugin:
  - Register dynamic kinds for its opaque types (e.g., `reqwest::Client`).
  - Publish function pointers and type constructors into VM globals (for dynamic usage when needed).
- After plugin preloading/registration, compile and load the cell’s temporary `dylib` and call `run()` (existing flow).

4) Codegen for rimported calls:
- Generate Rust that links to plugin wrappers via normal Rust item paths (Rust ABI), not raw symbol lookup:
  - Add `[dependencies]` entries pointing at the plugin crates (by absolute `path`), and `use kayton_rplugin__<crate>::exports::*;` in the generated lib.
  - Calls like `Client::new()` or `StatusCode::OK` resolve into the plugin’s `exports` module (which internally re-exports from the real crate or wraps as needed).
- For operations that need the VM (e.g., persisting/retrieving across cells), call plugin wrapper fns that internally use the `KaytonContext` given at registration time via VM globals or exported thunks (see “State passing”).

---

### Plugin ABI and Surface

Required exports in every plugin crate (`dylib`, Rust ABI):
- `#[no_mangle] pub extern "Rust" fn kayton_plugin_register(ctx: &mut KaytonContext)`
  - Responsibilities:
    - Register dynamic kinds for opaque types the plugin wants to persist across cells.
    - Publish named globals in the VM for:
      - Function pointers for dynamic-call fallbacks.
      - Type constructor functions that build opaque values and store them.
    - Optionally publish a manifest JSON string describing exported items.

- `pub mod exports { ... }` (Rust module, not a C ABI surface)
  - `pub use real_crate::{RequestedType, RequestedEnum, ...};`
  - For functions and methods used from Keyton, provide thin `pub` wrappers with `extern "Rust"` or normal Rust calling convention if only used intralib. Example naming:
    - `pub fn __reqwest__client_new() -> reqwest::Client` (for direct use in generated code)
    - `pub extern "Rust" fn __reqwest__client_new_dyn(ctx: &mut KaytonContext) -> HKayRef` (for dynamic fallback that stores the value in VM).

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
- Emit `Cargo.toml` with `[lib] crate-type = ["dylib"]` (unchanged for the temp crate) and add plugin crates under `[dependencies]` (absolute `path`).
- Insert `use kayton_rplugin__<crate>::exports::*;` at the top of the generated lib, and compile calls/paths accordingly.
- Provide a dynamic-call fallback helper only when needed (see below).

---

### Host Changes (kayton_interactive_shared)

1) Import planning:
- During `prepare_input`, after resolve/typecheck, collect `RequiredPlugins` and the set of exported items used by the cell.

2) Plugin build/cache:
- For each required plugin key `(crate, version, features, exports)`,
  - Ensure a generated plugin crate is present under `target/kayton_plugins/<crate>/<hash>/`.
  - If missing or stale, generate its `Cargo.toml` and `src/lib.rs`, then `cargo build --release` it.

3) Preload + register:
- Before loading the cell `dylib`, call `Library::new` on each plugin DLL path (absolute), then look up and call `kayton_plugin_register` with a `VmKaytonContext` (use existing `set_report_host_from_ctx` pattern to construct/validate the context pointers).

4) Compile cell with plugin deps:
- Call `compile_generated_rust_to_dylib_with_deps(source, ExtraDeps)` (new API) which writes plugin `[dependencies]` into the temp crate’s `Cargo.toml`.

5) Execution order:
- Preload/register plugins → build/load cell `dylib` → set reporters → `run()`.

---

### State Passing and Dynamic Fallback

Goal: Generated code should rarely need VM access directly. Prefer direct Rust calls to plugin `exports` wrappers. For cases that require VM interaction (persist across cells, large data), use dynamic fallback:

- Provide a tiny host helper exported to generated crates via a proc macro or inline module that can access the `KaytonContext` through existing reporter plumbing if/when needed (e.g., an exported `extern "C" fn kayton_get_context_ptr() -> (host_data, api_ptr)` routed via `HOST_PTRS`).
- Alternatively, have plugin wrappers that need a `KaytonContext` pull it from VM globals established at `kayton_plugin_register` time (plugin stores a stable handle/pointer via `set_global_dyn_ptr`).

Initial MVP can avoid this by limiting persisted values to ints/strings and performing dynamic-kind registration only for future extensions.

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
- Wrap plugin wrapper bodies in `std::panic::catch_unwind` when crossing the dynamic boundary; convert panics to `KaytonError` or abort with a clear message.
- Provide a `kayton_plugin_manifest()` JSON export listing:
  - plugin name, version, rustc info
  - exported types/functions and generic instantiations

---

### Generating Plugin Crates

Inputs:
- Crate name (e.g., `reqwest`), optional version/feature set
- Items to export (types, functions), and list of generic instantiations

Outputs:
- `Cargo.toml` (dylib), `src/lib.rs` with:
  - `exports` module: `pub use`/wrapper fns for direct Rust calls from generated code
  - `kayton_plugin_register` that registers dynamic kinds and publishes any dynamic-call helpers
  - Optional `kayton_plugin_manifest` for diagnostics

Caching:
- Directory layout: `target/kayton_plugins/<crate>/<hash>/` where `<hash>` covers requested items + versions + features.

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

### MVP Scope

- End-to-end `rimport reqwest` and `from reqwest rimport Client, StatusCode`:
  - Generate and build `kayton_rplugin__reqwest` (`dylib`).
  - Preload + register plugin.
  - Emit code to use `StatusCode` and construct `Client` in the cell.
  - No persistence of `Client` across cells in v1; focus on successful calls and compilation.

Follow-ups:
- Extend prelude/epilogue to persist dynamic values using VM handles.
- Add more generic instantiations and method surface.
- Add reporting of complex values (e.g., via `Display`/`Debug` to `__stdout`).

---

### Milestones & Tasks

1) Compiler front-end
- Parse/resolve `rimport` forms; produce `RequiredPlugins`.

2) Plugin generator
- Emit `dylib` crate (Cargo.toml + lib.rs) with `exports` and `kayton_plugin_register`.
- Implement generic instantiation templates for the initial shapes.

3) Host integration
- Build/cache plugins; preload and invoke `kayton_plugin_register`.
- Extend compile pipeline: pass plugin deps to the temp crate and `use` their `exports`.

4) Diagnostics & compatibility
- Embed and validate build metadata; fail fast on ABI mismatches.
- Helpful error messages when a symbol is missing or a plugin fails to load.

5) Tests (Windows focus)
- Unit: plugin generator output; registration calls; dynamic kind drop fns.
- Integration: rimport reqwest; call simple functions; ensure DLL preloading works.
- Nextest pipeline: compile/run cells that use imported types; verify outputs.

---

### Risks and Mitigations

- Rust ABI across DLLs: Pin toolchain; validate metadata; keep plugin and temp crate in lockstep; use `dylib` not `cdylib`.
- Windows DLL discovery: Preload via absolute paths; optionally copy DLLs next to the temp DLL.
- Generic explosion: restrict to explicit shapes in MVP; expand incrementally.
- Performance: cache built plugins; reuse across sessions when metadata matches.


