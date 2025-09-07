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

## Proposed ABI for rimport DLLs

All rimportable crates must build a companion `cdylib` that exposes a small, stable C ABI for discovery and registration.

Symbols the host expects:
- `#[no_mangle] extern "C" fn kayton_plugin_init(ctx: *mut KaytonContext)`
  - Called once after loading the plugin. The plugin uses the vtable to:
    - Register dynamic kinds for opaque types it owns (e.g., HTTP `Client`).
    - Set globals to function pointers or callable shims, or publish a registry table.
- `#[no_mangle] extern "C" fn kayton_plugin_name() -> *const u8` and `..._len() -> usize`
  - Optional metadata for display and debugging.
- Optional `kayton_plugin_version` metadata.

Discovery of functions/types:
- The plugin will expose a registry inside the DLL (Rust side), not by scanning. Two options for the registry export:
  1) Exported C ABI function returning a static JSON/CBOR blob describing functions/types and their stable symbol names.
  2) Exported typed table via FFI with arrays of entries. Simpler and no allocator crossing.

Recommended: typed table approach for zero-copy enums/strings:
- Export symbols:
  - `kayton_registry_fns(ptr_out: *mut *const FnEntry, len_out: *mut usize)`
  - `kayton_registry_types(ptr_out: *mut *const TypeEntry, len_out: *mut usize)`

Where:
- `FnEntry { name: *const u8, name_len: usize, symbol: *const u8, symbol_len: usize, arity: u32, param_kinds: [ParamKind; N_MAX], ret_kind: RetKind }`
- `TypeEntry { name: *const u8, name_len: usize, kind: TypeKind, size: usize, alignment: usize, dynamic_kind_id_hint: u32 }`
- `ParamKind`/`RetKind` enumerate the supported concrete instantiations: I64, F64, StaticStr, StringBuf, VecI64, VecF64, Dynamic.

Function resolution:
- For each `FnEntry`, the host will lookup and cache a DLL symbol with name `symbol` via `libloading`, store a trampoline, and expose a Keyton-callable intrinsic.
- The trampoline has a fixed C ABI and marshals arguments from VM handles to native, using `KaytonApi` getters and conversions. Return values are written back into VM globals or as a handle.

Type registration:
- Opaque types (e.g., `reqwest::Client`) register a dynamic kind: plugin calls `register_dynamic_kind("reqwest::Client", drop_fn)` and stores the kind id internally; host stores it in a name → kind map for Keyton.
- Constructing such types happens through exported functions (e.g., `reqwest::Client::new() -> Dynamic`), which allocate and return a pointer that the host stores in the dynamic kind store.

## Data Flow at Runtime

1) User writes:
   - `rimport reqwest`
   - `from reqwest rimport Client, StatusCode`

2) The interactive engine detects rimport statements during `prepare_input` preprocessing and executes an import step before compiling user code:
   - Resolve crate name to a DLL path. Strategies:
     - If the plugin is prebuilt and found in a configured search path (e.g., `~/.kayton/plugins/reqwest.dll`), load it.
     - Else, synthesize a small crate that depends on `reqwest` and exposes the rimport plugin ABI, then `cargo build -r` to produce the DLL and cache it.

3) Load DLL using `libloading::Library::new`.
   - Call `kayton_plugin_init(ctx_ptr)` passing a `KaytonContext` with VM pointers.
   - Query registry tables to discover available functions/types/instantiations.
   - Populate an in-memory module namespace for `reqwest` with entries mapping to call shims.

4) Keyton code generation for calls into imported modules:
   - The parser recognizes `rimport` statements and records module/type names.
   - The resolver binds names under a module scope (e.g., `reqwest::Client`), validating that they exist in the imported registry.
   - The Rust code generator, when encountering a call of an imported function, emits a call to a host-provided shim symbol rather than generating Rust generics directly. At runtime, the shim inside the ephemeral program DLL calls into the imported plugin DLL via `extern "C"` function pointers captured in a global.

5) Execution:
   - Before invoking `run()` of the ephemeral program DLL, the host injects initialization calls that pass the VM context and the previously loaded plugin function pointers. This can be done by exposing a `kayton_link_plugins` function in the ephemeral DLL and calling it prior to `run()`; or by storing function pointers in the VM’s dynamic store/globals that the ephemeral code can read via `KaytonApi`.

## Generics Strategy

- Rust generics cannot be dynamically instantiated at runtime; they must be monomorphized at compile time in the plugin.
- Each plugin crate should expose concrete instantiations for the supported set:
  - Suffix or mangle into distinct symbols, e.g., `map_i64`, `map_f64`, `map_string`, `map_vec_i64`, `map_dynamic`.
  - The registry associates the high-level function name with the available concrete variants and their `ParamKind`/`RetKind` signatures.
- The Keyton typechecker will choose the appropriate variant based on the Keyton types; ambiguous cases can require explicit casts or a `dynamic` variant.

## Example: reqwest

Plugin crate (separate repo or generated adapter) exposes:
- Type: `reqwest::Client` as dynamic kind `REQWEST_CLIENT` with `drop_fn` that frees the boxed client.
- Functions:
  - `reqwest_client_new() -> Dynamic`
  - `reqwest_get_string(url: StringBuf) -> StringBuf` (simple convenience)
  - `reqwest_client_get(client: Dynamic, url: StringBuf) -> Dynamic` returning a `Response` opaque type.
  - `response_status(resp: Dynamic) -> I64` and `status_is_success(code: I64) -> Bool`.
- These are synchronous blocking wrappers using `reqwest::blocking` to avoid async complexity in the first cut.

Keyton usage:
```
rimport reqwest
let: Str = reqwest.get_string("https://example.com")
```

## Parser and Resolver Changes

- Lexer: no keyword tokens for `rimport` today; add `RImportKw` and `FromKw`.
- Parser: add two new statements:
  - `RImport { module: Ident }`
  - `RImportFrom { module: Ident, names: Vec<Ident> }`
- Resolver: maintain an import table for the compilation unit, populated before type checking. Names introduced by `RImportFrom` are added to the symbol table as externs with types from the plugin registry.
- Type system: introduce `Dynamic` and opaque imported types as distinct `Type` variants, checked only at call boundaries and assignments where allowed.

## Codegen Changes

- Prelude injection: before user stmts, insert calls to a host-provided import linker that writes function pointers for imported entries into static externs or VM globals the generated program can read.
- Function calls that resolve to imported symbols are emitted as calls to `extern "C"` functions whose addresses are filled prior to `run()` via a `set_*` initializer.
  - Simpler alternative: emit calls through a tiny host FFI layer inside the ephemeral DLL that fetches the plugin function pointer from VM globals (by name) and `transmute` to the expected signature.

Recommended first iteration (simpler and robust):
- The host keeps all plugin Libraries loaded and caches symbol pointers.
- The host exposes a small set of stable shims in `kayton_api` that allow the ephemeral DLL to invoke a plugin function by an integer ID:
  - `extern "C" fn kayton_call_fn(ctx: *mut KaytonContext, fn_id: u64, args_ptr: *const HKayRef, argc: usize) -> HKayRef`
- The code generator emits calls to `kayton_call_fn` with a compile-time constant `fn_id` assigned by the import resolver.
- Marshaling is centralized in the host side implementation behind `kayton_call_fn` (using the cached signature from the registry), avoiding per-program linking.

## VM/Host Additions in kayton_api and kayton_vm

- Add to `KaytonApi`:
  - `register_plugin_fn(name: &'static str, signature: Signature) -> u64` returns `fn_id`.
  - `call_plugin_fn(fn_id: u64, args_ptr: *const HKayRef, argc: usize) -> Result<HKayRef, KaytonError>`
  - Optional: `get_type_id(name: &'static str) -> KindId` for opaque kinds.

- In `HostState`:
  - A registry: `fn_id -> { signature, marshaler, dll_symbol_ptr }` and `name -> fn_id`.
  - A registry for imported types: `name -> KindId`.

## Kernel / Interactive Flow Updates

1) On `prepare_input`, pre-scan for rimport stmts:
   - For each module, ensure it is loaded (build/load DLL if missing).
   - Query `kayton_registry_*` and register all functions/types with unique `fn_id`s via new API calls.
   - Store a per-session import map `module -> { exported names }`.

2) Resolver consults the session import map to bind symbols.

3) Codegen emits `kayton_call_fn` calls with concrete `fn_id`s and arguments/returns are marshaled by VM host.

## Error Handling

- Missing DLL: surface a compile-time error in the cell explaining how to install/build the plugin.
- Missing symbol or unsupported generic: precise diagnostics listing available variants.
- Runtime errors within plugin: return `KaytonError::generic` with message; Jupyter displays stderr.

## Security Considerations

- Loading arbitrary DLLs is unsafe; default to a trusted plugin directory or explicit user opt-in.
- Consider a feature flag to disable rimport in locked-down environments.

## Phased Implementation Plan

Phase 1 (functional core):
- Add parser support for `rimport`/`from ... rimport ...`.
- Add `Dynamic` type to type system; allow it in calls and as return.
- Add `KaytonApi` entries for plugin function registry and invocation.
- Implement host-side registry and `kayton_call_fn` marshaler.
- Build a minimal `reqwest_adapter` plugin DLL exposing:
  - `client_new() -> Dynamic`
  - `get_string(url: StringBuf) -> StringBuf`
- End-to-end demo in Jupyter cell.

Phase 2 (types and more functions):
- Add opaque `Client` and `Response` kinds; constructor and simple getters.
- Implement `from reqwest rimport Client, StatusCode` name binding.

Phase 3 (generics and vectors):
- Support `Vec<i64>`/`Vec<f64>` parameters and returns; mapping to `KVec` or tuple of handles.
- Add more reqwest helpers; basic headers API.

Phase 4 (traits/macros, optional):
- Traits surfaced as capability sets on Dynamic kinds.
- Macro-like helpers as compile-time intrinsics.

## Open Questions / Risks

- How to represent macros faithfully? Proposed defer.
- Async: for `reqwest`, the blocking client avoids runtime executors; async support would require an executor in the host or in the generated program DLL.
- Versioning of the `KaytonApi` ABI: bump `abi_version` and `KaytonApi::size` checks for safety.
- Portability: Windows `.dll`, Linux `.so`, macOS `.dylib` naming handled already in temp compilation path; mirror for plugin search.


