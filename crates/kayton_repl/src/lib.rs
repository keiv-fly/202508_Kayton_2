use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use kayton_vm::{Api, KaytonVm, VmGlobalStrBuf, VmKaytonContext};
use keyton_rust_compiler::compile_rust::compile_generated_rust_to_dylib;
use keyton_rust_compiler::diagnostics::format_type_error;
use keyton_rust_compiler::hir::lower_program;
use keyton_rust_compiler::lexer::Lexer;
use keyton_rust_compiler::parser::Parser;
use keyton_rust_compiler::rhir::{RustProgram, convert_to_rhir};
use keyton_rust_compiler::rust_codegen::{CodeGenerator, RustCode};
use keyton_rust_compiler::shir::{resolve_program, sym::SymbolId};
use keyton_rust_compiler::thir::typecheck_program;
use libloading::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplVarKind {
    Int,
    Str,
}

// Store raw host pointers as usize to avoid Send/Sync bounds on statics
static HOST_PTRS: OnceLock<(usize, usize)> = OnceLock::new(); // (host_data, api_ptr)
static REPL_GLOBALS: OnceLock<Mutex<HashMap<String, ReplVarKind>>> = OnceLock::new();

fn init_globals_once() {
    REPL_GLOBALS.get_or_init(|| Mutex::new(HashMap::new()));
}

fn set_report_host_from_ctx(ctx: &VmKaytonContext) {
    let host_data = ctx.host_data as usize;
    let api_ptr = ctx.api as usize;
    let _ = HOST_PTRS.set((host_data, api_ptr));
}

// Host-side reporter functions that the compiled dylib will call to save globals into VM
type ReportIntFn = extern "C" fn(name_ptr: *const u8, name_len: usize, value: i64);
type ReportStrFn =
    extern "C" fn(name_ptr: *const u8, name_len: usize, str_ptr: *const u8, str_len: usize);

extern "C" fn host_report_int(name_ptr: *const u8, name_len: usize, value: i64) {
    unsafe {
        let name_slice = core::slice::from_raw_parts(name_ptr, name_len);
        if let Ok(name) = core::str::from_utf8(name_slice) {
            if let Some((host_data, api_ptr)) = HOST_PTRS.get().copied() {
                let mut ctx = VmKaytonContext {
                    abi_version: 1,
                    host_data: host_data as *mut core::ffi::c_void,
                    api: api_ptr as *const Api,
                };
                let api: &Api = ctx.api();
                let _ = (api.set_global_u64)(&mut ctx, name, value as u64);
                if let Some(globals) = REPL_GLOBALS.get() {
                    if let Ok(mut m) = globals.lock() {
                        m.insert(name.to_string(), ReplVarKind::Int);
                    }
                }
            }
        }
    }
}

extern "C" fn host_report_str(
    name_ptr: *const u8,
    name_len: usize,
    str_ptr: *const u8,
    str_len: usize,
) {
    unsafe {
        let name_slice = core::slice::from_raw_parts(name_ptr, name_len);
        let str_slice = core::slice::from_raw_parts(str_ptr, str_len);
        if let (Ok(name), Ok(val)) = (
            core::str::from_utf8(name_slice),
            core::str::from_utf8(str_slice),
        ) {
            if let Some((host_data, api_ptr)) = HOST_PTRS.get().copied() {
                let mut ctx = VmKaytonContext {
                    abi_version: 1,
                    host_data: host_data as *mut core::ffi::c_void,
                    api: api_ptr as *const Api,
                };
                let api: &Api = ctx.api();
                let buf = VmGlobalStrBuf::new(val.to_string());
                let _ = (api.set_global_str_buf)(&mut ctx, name, buf);
                if let Some(globals) = REPL_GLOBALS.get() {
                    if let Ok(mut m) = globals.lock() {
                        m.insert(name.to_string(), ReplVarKind::Str);
                    }
                }
            }
        }
    }
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
) -> (HashSet<SymbolId>, Vec<String>, Vec<String>) {
    // Gather used variables and assigned variables (by SymbolId)
    let mut used_syms: HashSet<SymbolId> = HashSet::new();
    let mut assigned_syms: HashSet<SymbolId> = HashSet::new();

    // Traverse the R-level program is possible, but we also can rely on typed.var_types indices.
    // Here we inspect program.rhir to find names used in assignment and usage by scanning expressions.
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

    for stmt in &program.rhir {
        match stmt {
            RStmt::Assign { sym, expr, .. } => {
                assigned_syms.insert(*sym);
                collect_expr_syms(expr, &mut used_syms);
            }
            RStmt::ExprStmt { expr, .. } => collect_expr_syms(expr, &mut used_syms),
        }
    }

    // Map SymbolId -> var name
    let sym_infos = &resolved.symbols.infos;
    let mut prelude_lines: Vec<String> = Vec::new();
    let mut pre_assigned: HashSet<SymbolId> = HashSet::new();
    let mut epilogue_lines: Vec<String> = Vec::new();

    // Load known globals used this line
    if let Some(globals) = REPL_GLOBALS.get() {
        if let Ok(g) = globals.lock() {
            let api: &Api = vm.api(); // using vm.api is fine; we still pass &mut ctx to calls
            for sym in used_syms.iter() {
                let name = &sym_infos[sym.0 as usize].name;
                if let Some(kind) = g.get(name) {
                    match kind {
                        ReplVarKind::Int => {
                            if let Ok(val) = (api.get_global_u64)(ctx, name) {
                                prelude_lines.push(format!("let mut {} = {};", name, val));
                                pre_assigned.insert(*sym);
                            }
                        }
                        ReplVarKind::Str => {
                            if let Ok(buf) = (api.get_global_str_buf)(ctx, name) {
                                if let Some(s) = buf.as_str() {
                                    let lit = escape_rust_string_literal(s);
                                    prelude_lines.push(format!("let mut {} = \"{}\";", name, lit));
                                    pre_assigned.insert(*sym);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Save assigned variables at end of run
    for sym in assigned_syms.iter() {
        let name = &sym_infos[sym.0 as usize].name;
        // Best effort: decide by prior knowledge; default to int
        let kind = REPL_GLOBALS
            .get()
            .and_then(|m| m.lock().ok())
            .and_then(|m| m.get(name).cloned())
            .unwrap_or(ReplVarKind::Int);
        match kind {
            ReplVarKind::Int => {
                epilogue_lines.push(format!(
                    "unsafe {{ report_int(\"{}\", {} as i64); }}",
                    name, name
                ));
            }
            ReplVarKind::Str => {
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

/// Run the Kayton REPL loop
pub fn run_repl() -> Result<()> {
    init_globals_once();

    let mut vm = KaytonVm::new();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input_counter: usize = 0;
    loop {
        write!(stdout, ">>> ")?;
        stdout.flush()?;

        let mut line = String::new();
        let n = stdin.read_line(&mut line)?;
        if n == 0 {
            break;
        }

        let source = line.trim_end_matches(&['\n', '\r'][..]);
        if source.is_empty() {
            continue;
        }

        // Parse / typecheck
        let tokens = Lexer::new(source).tokenize();
        let ast = Parser::new(tokens).parse_program();
        let hir = lower_program(ast);
        let mut resolved = resolve_program(&hir);
        let typed = typecheck_program(&mut resolved);
        if !typed.report.errors.is_empty() {
            let mut printed = false;
            for err in &typed.report.errors {
                let file_label = format!("<kayton-input-{}>", input_counter);
                if let Some(msg) = format_type_error(source, &resolved, err, &file_label) {
                    eprintln!("{}", msg);
                    printed = true;
                    break;
                }
            }
            if !printed {
                eprintln!("Type errors: {:?}", typed.report.errors);
            }
            input_counter += 1;
            continue;
        }
        let rhir_program = convert_to_rhir(&typed, &resolved);

        // Prepare VM context for loading
        let mut ctx = vm.context();

        let (pre_assigned, prelude_lines, epilogue_lines) =
            build_prelude_and_epilogue(&vm, &mut ctx, &resolved, &rhir_program);

        // Update REPL_GLOBALS for assigned vars with types inferred now
        if let Some(globals) = REPL_GLOBALS.get() {
            if let Ok(mut g) = globals.lock() {
                for (sid, ty) in typed.var_types.iter() {
                    let name = &resolved.symbols.infos[sid.0 as usize].name;
                    let kind = match ty {
                        keyton_rust_compiler::shir::sym::Type::Str => ReplVarKind::Str,
                        _ => ReplVarKind::Int,
                    };
                    g.insert(name.clone(), kind);
                }
            }
        }

        let rust_code = generate_injected_rust(
            &resolved,
            &rhir_program,
            &pre_assigned,
            &prelude_lines,
            &epilogue_lines,
        );

        // Compile dylib
        match compile_generated_rust_to_dylib(&rust_code.source_code) {
            Ok(path) => unsafe {
                let lib = Library::new(&path).with_context(|| format!("load dylib: {:?}", path))?;

                // Set reporter hooks
                type SetReportersFn = unsafe extern "C" fn(ReportIntFn, ReportStrFn);
                if let Ok(setters) = lib.get::<SetReportersFn>(b"kayton_set_reporters") {
                    // Stash host pointers used by reporters
                    set_report_host_from_ctx(&ctx);
                    setters(
                        host_report_int as ReportIntFn,
                        host_report_str as ReportStrFn,
                    );
                }

                let func: libloading::Symbol<unsafe extern "C" fn()> =
                    lib.get(b"run").context("find run symbol")?;
                func();

                // lib dropped here
            },
            Err(err) => {
                eprintln!("Compile error: {}", err);
            }
        }

        input_counter += 1;
    }

    Ok(())
}
