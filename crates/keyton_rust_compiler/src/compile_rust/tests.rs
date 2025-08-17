use super::*;
use libloading::Library;

#[test]
fn compile_and_run_print() {
    let src = r#"print("Hello from dylib")"#;
    let lib_path = compile_lang_source_to_dylib(src).expect("compile to dylib");
    unsafe {
        let lib = Library::new(&lib_path).expect("load dylib");
        let func: libloading::Symbol<unsafe extern "C" fn()> =
            lib.get(b"run").expect("find run symbol");
        func();
    }
}

#[test]
fn compile_and_run_math() {
    let src = r#"x = 12
x = x + 30
print(x)
"#;
    let lib_path = compile_lang_source_to_dylib(src).expect("compile to dylib");
    unsafe {
        let lib = Library::new(&lib_path).expect("load dylib");
        let func: libloading::Symbol<unsafe extern "C" fn()> =
            lib.get(b"run").expect("find run symbol");
        func();
    }
}
