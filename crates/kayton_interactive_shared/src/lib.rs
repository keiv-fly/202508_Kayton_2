use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use core::ffi::c_void;
use kayton_vm::{
    Api, KaytonVm, ReportIntFn, ReportStrFn, VmKaytonContext, host_report_int, host_report_str,
    set_report_host_from_ctx, set_stdout_callback,
};
use keyton_rust_compiler::compile_rust::compile_generated_rust_to_dylib;
use keyton_rust_compiler::diagnostics::format_type_error;
use keyton_rust_compiler::hir::lower_program;
use keyton_rust_compiler::lexer::Lexer;
use keyton_rust_compiler::parser::Parser;
use keyton_rust_compiler::rhir::{RustProgram, convert_to_rhir};
use keyton_rust_compiler::rimport::env::discover_plugin_dll_path;
use keyton_rust_compiler::rust_codegen::{CodeGenerator, RustCode};
use keyton_rust_compiler::shir::{resolve_program, sym::SymbolId};
use libloading::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    Int,
    Str,
}

pub struct InteractiveState {
    vm: KaytonVm,
    /// Map of user-visible variable names to simple kinds for prelude/epilogue decisions
    pub globals: HashMap<String, VarKind>,
    /// Persisted function definitions across inputs
    pub stored_functions: Vec<String>,
    /// Monotonic counter for labelling inputs in diagnostics
    pub input_counter: usize,
}

impl InteractiveState {
    pub fn new() -> Self {
        Self {
            vm: KaytonVm::new(),
            globals: HashMap::new(),
            stored_functions: Vec::new(),
            input_counter: 0,
        }
    }

    pub fn vm(&self) -> &KaytonVm {
        &self.vm
    }
    pub fn vm_mut(&mut self) -> &mut KaytonVm {
        &mut self.vm
    }
}

pub struct PreparedCode {
    pub full_source: String,
    pub rust: RustCode,
}

fn escape_rust_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn build_prelude_and_epilogue(
    vm: &KaytonVm,
    ctx: &mut VmKaytonContext,
    resolved: &keyton_rust_compiler::shir::resolver::ResolvedProgram,
    program: &RustProgram,
    globals: &HashMap<String, VarKind>,
) -> (HashSet<SymbolId>, Vec<String>, Vec<String>) {
    use keyton_rust_compiler::rhir::types::{RExpr, RStmt};
    fn collect_expr_syms(e: &RExpr, out: &mut HashSet<SymbolId>) {
        match e {
            RExpr::Name { sym, .. } => {
                out.insert(*sym);
            }
            RExpr::Binary { left, right, .. } => {
                collect_expr_syms(left, out);
                collect_expr_syms(right, out);
            }
            RExpr::MacroCall { args, .. } => {
                for a in args {
                    collect_expr_syms(a, out);
                }
            }
            RExpr::InterpolatedString { parts, .. } => {
                for p in parts {
                    if let keyton_rust_compiler::rhir::types::RStringPart::Expr { expr, .. } = p {
                        collect_expr_syms(expr, out);
                    }
                }
            }
            _ => {}
        }
    }

    let mut used_syms: HashSet<SymbolId> = HashSet::new();
    let mut assigned_syms: HashSet<SymbolId> = HashSet::new();
    fn walk_stmt(
        stmt: &RStmt,
        used_syms: &mut HashSet<SymbolId>,
        assigned_syms: &mut HashSet<SymbolId>,
    ) {
        match stmt {
            RStmt::RImportModule { .. } | RStmt::RImportItems { .. } => {}
            RStmt::Assign { sym, expr, .. } => {
                assigned_syms.insert(*sym);
                collect_expr_syms(expr, used_syms);
            }
            RStmt::ExprStmt { expr, .. } => collect_expr_syms(expr, used_syms),
            RStmt::ForRange {
                start, end, body, ..
            } => {
                collect_expr_syms(start, used_syms);
                collect_expr_syms(end, used_syms);
                for s in body {
                    walk_stmt(s, used_syms, assigned_syms);
                }
            }
            RStmt::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                collect_expr_syms(cond, used_syms);
                for s in then_branch {
                    walk_stmt(s, used_syms, assigned_syms);
                }
                for s in else_branch {
                    walk_stmt(s, used_syms, assigned_syms);
                }
            }
        }
    }

    for stmt in &program.rhir {
        walk_stmt(stmt, &mut used_syms, &mut assigned_syms);
    }

    let sym_infos = &resolved.symbols.infos;
    let mut prelude_lines: Vec<String> = Vec::new();
    let mut pre_assigned: HashSet<SymbolId> = HashSet::new();
    let mut epilogue_lines: Vec<String> = Vec::new();

    let api: &Api = vm.api();
    for sym in used_syms.iter() {
        let name = &sym_infos[sym.0 as usize].name;
        if let Some(kind) = globals.get(name) {
            match kind {
                VarKind::Int => {
                    match (api.get_global_u64)(ctx, name) {
                        Ok(val) => prelude_lines.push(format!("let mut {} = {};", name, val)),
                        Err(_) => prelude_lines.push(format!("let mut {} = 0;", name)),
                    }
                    pre_assigned.insert(*sym);
                }
                VarKind::Str => {
                    match (api.get_global_str_buf)(ctx, name) {
                        Ok(buf) => {
                            if let Some(s) = buf.as_str() {
                                let lit = escape_rust_string_literal(s);
                                prelude_lines.push(format!("let mut {} = \"{}\";", name, lit));
                            } else {
                                prelude_lines.push(format!("let mut {} = \"\";", name));
                            }
                        }
                        Err(_) => prelude_lines.push(format!("let mut {} = \"\";", name)),
                    }
                    pre_assigned.insert(*sym);
                }
            }
        }
    }

    for sym in assigned_syms.iter() {
        let name = &sym_infos[sym.0 as usize].name;
        let kind = globals.get(name).copied().unwrap_or(VarKind::Int);
        match kind {
            VarKind::Int => {
                epilogue_lines.push(format!(
                    "unsafe {{ report_int(\"{}\", {} as i64); }}",
                    name, name
                ));
            }
            VarKind::Str => {
                epilogue_lines.push(format!("unsafe {{ report_str(\"{}\", {}); }}", name, name));
            }
        }
    }

    (pre_assigned, prelude_lines, epilogue_lines)
}

fn generate_injected_rust(
    resolved: &keyton_rust_compiler::shir::resolver::ResolvedProgram,
    rhir_program: &RustProgram,
    pre_assigned: &HashSet<SymbolId>,
    prelude_lines: &[String],
    epilogue_lines: &[String],
) -> RustCode {
    let mut codegen = CodeGenerator::new(resolved);
    codegen.generate_code_with_preassigned_and_prelude(
        rhir_program,
        pre_assigned,
        prelude_lines,
        epilogue_lines,
    )
}

/// Prepare a single-line or block input for execution: parse, typecheck, generate Rust, build dylib, and return Rust code.
pub fn prepare_input(
    state: &mut InteractiveState,
    first_line_no_crlf: &str,
) -> Result<PreparedCode> {
    let mut full_source = String::new();
    if !state.stored_functions.is_empty() {
        for def in &state.stored_functions {
            full_source.push_str(def);
            if !def.ends_with('\n') {
                full_source.push('\n');
            }
            full_source.push('\n');
        }
    }
    full_source.push_str(first_line_no_crlf);

    let tokens = Lexer::new(&full_source).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);

    let mut predeclared: Vec<(String, keyton_rust_compiler::shir::sym::Type)> = Vec::new();
    for (name, kind) in state.globals.iter() {
        let ty = match kind {
            VarKind::Str => keyton_rust_compiler::shir::sym::Type::Str,
            VarKind::Int => keyton_rust_compiler::shir::sym::Type::I64,
        };
        predeclared.push((name.clone(), ty));
    }

    let typed = keyton_rust_compiler::thir::typecheck_program_with_env(&mut resolved, &predeclared);
    if !typed.report.errors.is_empty() {
        for err in &typed.report.errors {
            let file_label = format!("<kayton-input-{}>", state.input_counter);
            if let Some(msg) = format_type_error(&full_source, &resolved, err, &file_label) {
                return Err(anyhow::anyhow!(msg));
            }
        }
        return Err(anyhow::anyhow!(format!(
            "Type errors: {:?}",
            typed.report.errors
        )));
    }

    let rhir_program = convert_to_rhir(&typed, &resolved);

    let mut ctx = state.vm_mut().context();
    let (pre_assigned, prelude_lines, epilogue_lines) = build_prelude_and_epilogue(
        state.vm(),
        &mut ctx,
        &resolved,
        &rhir_program,
        &state.globals,
    );

    for (sid, ty) in typed.var_types.iter() {
        let name = &resolved.symbols.infos[sid.0 as usize].name;
        let kind = match ty {
            keyton_rust_compiler::shir::sym::Type::Str => VarKind::Str,
            _ => VarKind::Int,
        };
        state.globals.insert(name.clone(), kind);
    }

    let rust_code = generate_injected_rust(
        &resolved,
        &rhir_program,
        &pre_assigned,
        &prelude_lines,
        &epilogue_lines,
    );

    Ok(PreparedCode {
        full_source,
        rust: rust_code,
    })
}

/// Execute previously prepared code: compiles to a dylib and runs it, updating VM via reporter hooks.
pub fn execute_prepared(state: &mut InteractiveState, prepared: &PreparedCode) -> Result<()> {
    match compile_generated_rust_to_dylib(&prepared.rust.source_code) {
        Ok(path) => unsafe {
            let lib = Library::new(&path).with_context(|| format!("load dylib: {:?}", path))?;

            // Set reporter hooks from VM context
            type SetReportersFn = unsafe extern "C" fn(ReportIntFn, ReportStrFn);
            if let Ok(setters) = lib.get::<SetReportersFn>(b"kayton_set_reporters") {
                let mut ctx = state.vm_mut().context();
                set_report_host_from_ctx(&mut ctx);
                setters(
                    host_report_int as ReportIntFn,
                    host_report_str as ReportStrFn,
                );
            }

            // Set VM hooks for plugin loading and function pointer lookups
            type LoadPluginFn = extern "C" fn(module_ptr: *const u8, module_len: usize) -> i32;
            type GetFunctionPtrFn =
                extern "C" fn(name_ptr: *const u8, name_len: usize) -> *const c_void;
            type SetVmHooksFn = unsafe extern "C" fn(LoadPluginFn, GetFunctionPtrFn);
            if let Ok(set_vm_hooks) = lib.get::<SetVmHooksFn>(b"kayton_set_vm_hooks") {
                set_current_vm_ptr(state.vm_mut());
                set_vm_hooks(load_plugin_host, get_function_ptr_host);
            }

            let func: libloading::Symbol<unsafe extern "C" fn()> =
                lib.get(b"run").context("find run symbol")?;
            func();
        },
        Err(err) => {
            return Err(anyhow::anyhow!(format!("Compile error: {}", err)));
        }
    }
    Ok(())
}

/// Execute prepared code and stream stdout in real time via provided callback.
/// The callback is invoked with text chunks exactly as reported by `println!` (including newlines).
pub fn execute_prepared_streaming<F>(
    state: &mut InteractiveState,
    prepared: &PreparedCode,
    mut on_stdout: F,
) -> Result<()>
where
    F: FnMut(&str) + 'static,
{
    thread_local! {
        static STDOUT_SINK: std::cell::RefCell<Option<Box<dyn FnMut(&str)>>> =
            std::cell::RefCell::new(None);
    }

    extern "C" fn forward_stdout(text_ptr: *const u8, text_len: usize) {
        unsafe {
            let slice = core::slice::from_raw_parts(text_ptr, text_len);
            if let Ok(s) = core::str::from_utf8(slice) {
                STDOUT_SINK.with(|slot| {
                    if let Some(cb) = &mut *slot.borrow_mut() {
                        cb(s);
                    }
                });
            }
        }
    }

    match compile_generated_rust_to_dylib(&prepared.rust.source_code) {
        Ok(path) => unsafe {
            let lib = Library::new(&path).with_context(|| format!("load dylib: {:?}", path))?;

            // Set reporter hooks from VM context
            type SetReportersFn = unsafe extern "C" fn(ReportIntFn, ReportStrFn);
            if let Ok(setters) = lib.get::<SetReportersFn>(b"kayton_set_reporters") {
                let mut ctx = state.vm_mut().context();
                set_report_host_from_ctx(&mut ctx);
                setters(
                    host_report_int as ReportIntFn,
                    host_report_str as ReportStrFn,
                );
            }

            // Set VM hooks for plugin loading and function pointer lookups
            type LoadPluginFn = extern "C" fn(module_ptr: *const u8, module_len: usize) -> i32;
            type GetFunctionPtrFn =
                extern "C" fn(name_ptr: *const u8, name_len: usize) -> *const c_void;
            type SetVmHooksFn = unsafe extern "C" fn(LoadPluginFn, GetFunctionPtrFn);
            if let Ok(set_vm_hooks) = lib.get::<SetVmHooksFn>(b"kayton_set_vm_hooks") {
                set_current_vm_ptr(state.vm_mut());
                set_vm_hooks(load_plugin_host, get_function_ptr_host);
            }

            // Install stdout streaming callback
            STDOUT_SINK.with(|slot| {
                *slot.borrow_mut() = Some(Box::new(move |s: &str| on_stdout(s)));
            });
            set_stdout_callback(Some(forward_stdout));

            let func: libloading::Symbol<unsafe extern "C" fn()> =
                lib.get(b"run").context("find run symbol")?;
            func();

            // Clear callback after execution
            set_stdout_callback(None);
            STDOUT_SINK.with(|slot| {
                *slot.borrow_mut() = None;
            });
        },
        Err(err) => {
            return Err(anyhow::anyhow!(format!("Compile error: {}", err)));
        }
    }
    Ok(())
}

/// Kernel helper: set or clear the VM stdout streaming callback.
/// Exposed to avoid the kernel depending directly on `kayton_vm`.
pub type OnStdoutFn = extern "C" fn(text_ptr: *const u8, text_len: usize);
pub fn set_stdout_callback_thunk(cb: Option<OnStdoutFn>) {
    set_stdout_callback(cb);
}

/// Retrieve and clear the captured program stdout accumulated in `__stdout`.
/// Returns the string that was captured since the last drain.
pub fn take_stdout(state: &mut InteractiveState) -> String {
    // Access VM context and API
    let mut ctx = state.vm_mut().context();
    let api: &Api = state.vm().api();

    // Read current buffer
    let s = (api.get_global_str_buf)(&mut ctx, "__stdout")
        .ok()
        .and_then(|sb| sb.as_str().map(|t| t.to_string()))
        .unwrap_or_default();

    // Clear buffer for subsequent cells
    let _ = (api.set_global_static_str)(&mut ctx, "__stdout", "");

    s
}

// -------- VM hooks implementation for generated code --------
// We hold a raw pointer to the active KaytonVm while executing user code so we can route callbacks.
static mut CURRENT_VM_PTR: Option<*mut KaytonVm> = None;

fn set_current_vm_ptr(vm: &mut KaytonVm) {
    unsafe {
        CURRENT_VM_PTR = Some(vm as *mut KaytonVm);
    }
}

extern "C" fn load_plugin_host(module_ptr: *const u8, module_len: usize) -> i32 {
    unsafe {
        let slice = core::slice::from_raw_parts(module_ptr, module_len);
        if let Ok(module) = core::str::from_utf8(slice) {
            if let Some(vm_ptr) = CURRENT_VM_PTR {
                let vm = &mut *vm_ptr;
                match discover_plugin_dll_path(module) {
                    Ok(p) => match vm.load_plugin_from_path(&p) {
                        Ok(_) => 0,
                        Err(_) => 2,
                    },
                    Err(_) => 1,
                }
            } else {
                3
            }
        } else {
            4
        }
    }
}

extern "C" fn get_function_ptr_host(name_ptr: *const u8, name_len: usize) -> *const c_void {
    unsafe {
        let slice = core::slice::from_raw_parts(name_ptr, name_len);
        if let Ok(name) = core::str::from_utf8(slice) {
            if let Some(vm_ptr) = CURRENT_VM_PTR {
                let vm = &mut *vm_ptr;
                if let Some(p) = vm.get_function_ptr(name) {
                    return p as *const c_void;
                }
            }
        }
        core::ptr::null()
    }
}
