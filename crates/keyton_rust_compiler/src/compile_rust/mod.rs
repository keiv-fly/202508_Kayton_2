use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::hir::lower_program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::rhir::convert_to_rhir;
use crate::rust_codegen::generate_rust_code;
use crate::shir::resolve_program;
use crate::thir::typecheck_program;

/// Build a temporary Rust crate that compiles to a `dylib` and returns the built library path.
/// The provided Rust code should be a full program body that currently contains `fn main()`.
/// We wrap it into a library exposing `run()` to be called from dynamic loading.
pub fn compile_generated_rust_to_dylib(source_code: &str) -> anyhow::Result<PathBuf> {
    let temp_dir = tempfile::tempdir()?;
    let crate_dir = temp_dir.path();

    // Create Cargo.toml configured for dylib
    let cargo_toml = r#"[package]
name = "temp_dynlib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["dylib"]

[dependencies]
"#;
    fs::write(crate_dir.join("Cargo.toml"), cargo_toml)?;

    // Transform the generated main program into a library that exposes `run()`
    // 1) Define a local `println!` macro that accepts a single expression and uses `{}` formatting.
    // 2) Replace `fn main()` with `#[no_mangle] pub extern "C" fn run()`.
    let replaced = source_code.replace("fn main() {", "#[no_mangle]\npub extern \"C\" fn run() {");

    // If replacement did not occur for some reason, add a `run` wrapper that calls `main()`
    let lib_body = if replaced == source_code {
        format!(
            "{}\n\n#[no_mangle]\npub extern \"C\" fn run() {{\n    main();\n}}\n",
            source_code
        )
    } else {
        replaced
    };

    // Prepend compatibility macro for println! and REPL reporting hooks
    let macro_header = r#"
macro_rules! println {
    ($e:expr) => {{
        let s = ::std::format!("{}\n", $e);
        unsafe { report_str("__stdout", &s); }
        // Also write to process stdout for local runs
        ::std::print!("{}", s);
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        let s = ::std::format!(concat!($fmt, "\n"), $($arg)*);
        unsafe { report_str("__stdout", &s); }
        ::std::print!("{}", s);
    }};
}

// ----- REPL host reporting hooks (set by host via kayton_set_reporters) -----
#[allow(non_camel_case_types)]
type ReportIntFn = extern "C" fn(name_ptr: *const u8, name_len: usize, value: i64);
#[allow(non_camel_case_types)]
type ReportStrFn = extern "C" fn(name_ptr: *const u8, name_len: usize, str_ptr: *const u8, str_len: usize);

static mut REPORT_INT: Option<ReportIntFn> = None;
static mut REPORT_STR: Option<ReportStrFn> = None;

#[no_mangle]
pub extern "C" fn kayton_set_reporters(int_fn: ReportIntFn, str_fn: ReportStrFn) {
    unsafe {
        REPORT_INT = Some(int_fn);
        REPORT_STR = Some(str_fn);
    }
}

#[inline]
unsafe fn report_int(name: &str, value: i64) {
    if let Some(f) = REPORT_INT { f(name.as_ptr(), name.len(), value); }
}
#[inline]
unsafe fn report_str(name: &str, s: &str) {
    if let Some(f) = REPORT_STR { f(name.as_ptr(), name.len(), s.as_ptr(), s.len()); }
}
"#;

    let lib_src = format!("{}\n{}", macro_header, lib_body);

    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("lib.rs"), lib_src)?;

    // Build the dylib
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--release");
    cmd.current_dir(&crate_dir);

    let output = cmd.output()?;
    if !output.status.success() {
        let mut err = String::new();
        err.push_str("cargo build failed\n");
        err.push_str(&String::from_utf8_lossy(&output.stderr));
        err.push_str(&String::from_utf8_lossy(&output.stdout));
        return Err(anyhow::anyhow!(err));
    }

    // Locate produced dylib
    let target_dir = crate_dir.join("target").join("release");

    #[cfg(target_os = "windows")]
    let pattern = "temp_dynlib.dll";
    #[cfg(target_os = "linux")]
    let pattern = "libtemp_dynlib.so";
    #[cfg(target_os = "macos")]
    let pattern = "libtemp_dynlib.dylib";

    let lib_path = target_dir.join(pattern);
    if lib_path.exists() {
        // Copy to a persistent temp location so the path remains valid after TempDir drops
        let ext = lib_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let dest = unique_temp_lib_path(ext);
        fs::copy(&lib_path, &dest)?;
        return Ok(dest);
    }

    // Fallback: scan directory for any .dll/.so/.dylib
    let lib = fs::read_dir(&target_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            if let Some(ext) = p.extension() {
                let e = ext.to_string_lossy().to_ascii_lowercase();
                e == "dll" || e == "so" || e == "dylib"
            } else {
                false
            }
        })
        .ok_or_else(|| anyhow::anyhow!("built library not found in target/release"))?;
    let ext = lib.extension().and_then(|s| s.to_str()).unwrap_or("");
    let dest = unique_temp_lib_path(ext);
    fs::copy(&lib, &dest)?;
    Ok(dest)
}

fn unique_temp_lib_path(ext: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    let filename = if ext.is_empty() {
        format!("keyton_dynlib_{}_{}", nanos, c)
    } else {
        format!("keyton_dynlib_{}_{}.{}", nanos, c, ext)
    };
    std::env::temp_dir().join(filename)
}

/// End-to-end: take source from our language, generate Rust, build dylib, return path.
pub fn compile_lang_source_to_dylib(source: &str) -> anyhow::Result<PathBuf> {
    let tokens = Lexer::new(source).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    let mut resolved = resolve_program(&hir);
    let typed = typecheck_program(&mut resolved);
    if !typed.report.errors.is_empty() {
        return Err(anyhow::anyhow!(format!(
            "type errors: {:?}",
            typed.report.errors
        )));
    }
    let rhir_program = convert_to_rhir(&typed, &resolved);
    let rust_code = generate_rust_code(&rhir_program, &resolved);

    compile_generated_rust_to_dylib(&rust_code.source_code)
}

#[cfg(test)]
mod tests;
