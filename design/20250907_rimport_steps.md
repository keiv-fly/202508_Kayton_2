# rimport Implementation Steps (2025-09-07)

This document provides a detailed roadmap for implementing the rimport system as designed in `20250907_rimport_plan.md`.

## Implementation Roadmap

### Phase 1: Foundation (M0-M1)

#### 1. Create kik CLI crate with basic structure and commands
- **Location**: `crates/kik/`
- **Tasks**:
  - Create `Cargo.toml` with CLI dependencies (`clap`, `anyhow`, `serde`)
  - Implement command structure: `create`, `activate`, `list`, `rinstall`, `uninstall`, `kernel install`
  - Add environment discovery logic (base, named, local)
  - Set up Windows PowerShell command structure
  - Add help text and error handling
- **Dependencies**: None
- **Estimated effort**: 2-3 days

#### 2. Implement environment discovery and management in kik
- **Location**: `crates/kik/src/env.rs`
- **Tasks**:
  - Environment layout: `%LOCALAPPDATA%\Kayton\envs\<env-name>\`
  - Environment activation via `KAYTON_ACTIVE_ENV` env var
  - Local environment support: `.kayton/` directory
  - Environment metadata: `env.json`, `registry.json`
  - Environment creation and validation
- **Dependencies**: Step 1
- **Estimated effort**: 3-4 days

#### 3. Create kayton_plugin_sdk crate with macros and types
- **Location**: `crates/kayton_plugin_sdk/`
- **Tasks**:
  - Define plugin contract: `kayton_plugin_abi_version()`, `kayton_plugin_manifest_json()`, `kayton_plugin_register()`
  - Create macros for exporting functions and types
  - Manifest generation for plugin metadata
  - Rust ABI compatibility helpers
  - Plugin wrapper generation utilities
- **Dependencies**: None
- **Estimated effort**: 4-5 days

### Phase 2: VM and API Extensions (M1-M3)

#### 4. Extend kayton_api with function and type registries
- **Location**: `crates/kayton_api/src/`
- **Tasks**:
  - Add function registry: `register_function()`, `get_function()`
  - Add type registry: `register_type()`, `get_type()`
  - Define `TypeMeta` struct with size, align, drop functions
  - Add signature validation helpers
  - Update `KaytonApi` vtable with new functions
- **Dependencies**: Step 3
- **Estimated effort**: 3-4 days

#### 5. Extend kayton_vm with plugin loading and symbol management
- **Location**: `crates/kayton_vm/src/`
- **Tasks**:
  - Plugin loading via `libloading`
  - ABI version validation
  - Function/type pointer caching
  - Dynamic kind management for opaque types
  - Plugin lifecycle management
  - Host-side registry storage
- **Dependencies**: Step 4
- **Estimated effort**: 4-5 days

### Phase 3: Compiler Integration (M2)

#### 6. Add rimport parsing to keyton_rust_compiler
- **Location**: `crates/keyton_rust_compiler/src/`
- **Tasks**:
  - Extend lexer for `rimport` and `from ... rimport ...` syntax
  - Add AST nodes for import statements
  - Update parser to handle import declarations
  - Add import statement to HIR
- **Dependencies**: None
- **Estimated effort**: 2-3 days

#### 7. Implement rimport name resolution and typechecking
- **Location**: `crates/keyton_rust_compiler/src/`
- **Tasks**:
  - Environment-aware name resolution
  - Plugin manifest loading and validation
  - Type checking against plugin signatures
  - Import error handling with remediation messages
  - Integration with existing typechecker
- **Dependencies**: Steps 2, 6
- **Estimated effort**: 4-5 days

#### 8. Add plugin loading codegen to Rust generator
- **Location**: `crates/keyton_rust_compiler/src/rust_codegen/`
- **Tasks**:
  - Generate plugin loading code at program start
  - Function pointer retrieval and caching
  - Safe wrapper generation for imported functions
  - Integration with existing `compile_generated_rust_to_dylib`
  - Rust ABI function declarations
- **Dependencies**: Steps 5, 7
- **Estimated effort**: 5-6 days

### Phase 4: Interactive Integration (M2-M3)

#### 9. Update kayton_interactive_shared for rimport preflight checks
- **Location**: `crates/kayton_interactive_shared/src/`
- **Tasks**:
  - Pre-compilation import validation
  - Environment plugin presence checks
  - Error reporting with `kik rinstall` guidance
  - Plugin loading orchestration
  - Integration with `prepare_input()`
- **Dependencies**: Steps 2, 7
- **Estimated effort**: 3-4 days

### Phase 5: Plugin System (M1-M4)

#### 10. Convert plugin_hello to dylib with Rust ABI
- **Location**: `crates/plugin_hello/`
- **Tasks**:
  - Change `crate-type` from `["cdylib", "rlib"]` to `["dylib"]`
  - Add plugin contract exports
  - Implement `kayton_plugin_register()` function
  - Test with new plugin loading system
  - Update tests to use new plugin system
- **Dependencies**: Steps 3, 5
- **Estimated effort**: 2-3 days

#### 11. Implement kik rinstall command for building plugins
- **Location**: `crates/kik/src/`
- **Tasks**:
  - Crate resolution and version management
  - Wrapper crate generation using `kayton_plugin_sdk`
  - `cargo vendor` for reproducible builds
  - Plugin compilation with nightly toolchain
  - Manifest generation and registry updates
  - Error handling and rollback
- **Dependencies**: Steps 2, 3, 10
- **Estimated effort**: 6-8 days

### Phase 6: Kernel Integration (M5)

#### 12. Add kik kernel install command
- **Location**: `crates/kik/src/`
- **Tasks**:
  - Jupyter kernel installation from environment
  - Custom kernel name support
  - Kernel configuration with environment paths
  - Kernel registration with Jupyter
  - Kernel removal and management
- **Dependencies**: Steps 2, 9
- **Estimated effort**: 3-4 days

### Phase 7: Testing and Examples (M6)

#### 13. Create end-to-end tests for rimport flow
- **Location**: `crates/*/tests/`
- **Tasks**:
  - Plugin loading tests
  - Function call tests
  - Type registration tests
  - Error handling tests
  - Environment switching tests
  - Integration tests across all components
- **Dependencies**: Steps 8, 11
- **Estimated effort**: 4-5 days

#### 14. Create reqwest plugin wrapper as example
- **Location**: `examples/reqwest_plugin/`
- **Tasks**:
  - Minimal reqwest surface: `Client`, `StatusCode`
  - HTTP GET functionality
  - Response handling
  - Sample notebook demonstrating usage
  - Documentation and examples
- **Dependencies**: Steps 11, 13
- **Estimated effort**: 3-4 days

## Implementation Order and Dependencies

```
Phase 1: Foundation
├── Step 1: kik CLI (no deps)
├── Step 3: plugin_sdk (no deps)
└── Step 2: env mgmt (depends on 1)

Phase 2: VM/API Extensions
├── Step 4: api registries (depends on 3)
└── Step 5: vm plugin support (depends on 4)

Phase 3: Compiler Integration
├── Step 6: rimport parsing (no deps)
├── Step 7: name resolution (depends on 2, 6)
└── Step 8: plugin codegen (depends on 5, 7)

Phase 4: Interactive Integration
└── Step 9: preflight checks (depends on 2, 7)

Phase 5: Plugin System
├── Step 10: plugin_hello conversion (depends on 3, 5)
└── Step 11: kik rinstall (depends on 2, 3, 10)

Phase 6: Kernel Integration
└── Step 12: kernel install (depends on 2, 9)

Phase 7: Testing and Examples
├── Step 13: e2e tests (depends on 8, 11)
└── Step 14: reqwest example (depends on 11, 13)
```

## Key Technical Decisions

1. **Plugin ABI**: Use `extern "Rust"` (unstable Rust ABI) with nightly toolchain pinning
2. **Safety Boundary**: All cross-boundary data uses `kayton_api` types
3. **Generics**: Pre-instantiated variants for `i64`, `f64`, `&str`, `String`, `Vec<i64>`, `Vec<f64>`, `Dynamic`
4. **Error Policy**: `rimport X` fails if not installed, with `kik rinstall X` guidance
5. **Environment Model**: Single active environment with base fallback

## Success Criteria

- [ ] `kik create k01` creates a new environment
- [ ] `kik activate k01` activates the environment
- [ ] `kik rinstall reqwest` builds and installs reqwest plugin
- [ ] `rimport reqwest` loads the plugin and registers functions
- [ ] `from reqwest rimport Client` resolves and typechecks
- [ ] Generated code calls plugin functions via pointers
- [ ] `kik kernel install` creates Jupyter kernel from environment
- [ ] End-to-end notebook execution works with imported crates

## Risk Mitigation

1. **ABI Stability**: Pin nightly toolchain per environment to ensure compatibility
2. **Plugin Loading**: Use `libloading` with proper error handling and cleanup
3. **Memory Safety**: All cross-boundary data uses `kayton_api` types only
4. **Error Handling**: Comprehensive error messages with actionable remediation
5. **Testing**: Extensive unit and integration tests for all components

## Estimated Timeline

- **Phase 1-2**: 2-3 weeks (Foundation and VM/API extensions)
- **Phase 3-4**: 2-3 weeks (Compiler and Interactive integration)
- **Phase 5-6**: 2-3 weeks (Plugin system and Kernel integration)
- **Phase 7**: 1-2 weeks (Testing and Examples)

**Total estimated effort**: 7-11 weeks for full implementation

This roadmap provides a clear, dependency-aware path from the current state to a fully functional rimport system that matches the design specifications in the plan.
