## Goal

Enable importing Rust crates at runtime in Keyton using rimport syntax:

```
rimport reqwest
from reqwest rimport Client, StatusCode
```

Then use their functions/types inside Jupyter cells. The import should:
- Load a compiled DLL for the crate.
- Register functions, types, and generic instantiations with the Kayton VM via kayton_api.
- Expose usable bindings to the Keyton compiler/codegen so calls compile into Rust that links dynamically to the DLL at execution time.

## Current Architecture (as of this repo)

- Interactive execution compiles Keyton code to Rust, builds a temporary dylib, then loads and invokes `run()` via `libloading`.
  - Key modules: `keyton_rust_compiler::compile_rust::compile_generated_rust_to_dylib`, `kayton_interactive_shared::execute_prepared`.
  - Host reporting hooks: the temporary dylib exposes `kayton_set_reporters`. The host sets VM pointers so `println!` and last-expression values can be reported back via `kayton_vm`.

- Kayton VM surface is exposed through `kayton_api` (no_std), with a single flat vtable `KaytonApi` providing getters/setters for primitives, strings, tuples, `KVec`, and a dynamic pointer kind subsystem.
  - Dynamic kinds: `register_dynamic_kind`, `set/get/drop_global_dyn_ptr` store opaque pointers with a custom drop function in `HostState` via `DynKindStore`.
  - All VM access is through a `KaytonContext { host_data, api }` passed across the FFI boundary.

- Jupyter kernel and REPL glue lives in `kayton_kernel` and `kayton_interactive_shared`.
  - On execute: parse → typecheck → lower → generate Rust → compile to dylib → load → set reporters → call `run()`.
  - Stdout and last expression result are surfaced via VM globals and callbacks.

- Plugin example: `plugin_hello` is built as `cdylib` and depends on `kayton_api`. It indicates the intended model for external DLLs talking to the VM.

## Desired rimport Capability

High-level requirements:
- Given a crate name (e.g., `reqwest`), compile and load a DLL plugin that:
  - Registers its exported types/traits/functions/macros with the VM.
  - Emits binding stubs so Keyton code can call into those functions and construct/use those types.
- Generics support: pre-instantiate generic functions/types for the following shapes: `i64`, `f64`, `&str`, `String`, `Vec<i64>`, `Vec<f64>`, and a `Dynamic` type (opaque handles registered with VM).

Non-goals for the first increment:
- Full Rust trait resolution inside Keyton. Instead, provide explicit, named instantiations exported by the plugin.
- Full macro import. Initial plan focuses on functions and data types; macro-like helpers can be surfaced as functions or codegen templates.

