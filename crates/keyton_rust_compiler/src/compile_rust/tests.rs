use super::*;
use libloading::Library;
use std::sync::Mutex;

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

#[test]
fn compile_and_run_user_function_sum() {
    let src = r#"x = 1
y = 2

fn my_sum(x, y):
    x + y

z = my_sum(x,y)
print(z)
"#;
    let lib_path = compile_lang_source_to_dylib(src).expect("compile to dylib");
    unsafe {
        let lib = Library::new(&lib_path).expect("load dylib");
        let func: libloading::Symbol<unsafe extern "C" fn()> =
            lib.get(b"run").expect("find run symbol");
        func();
    }
}

static CAPTURED: Mutex<Vec<u8>> = Mutex::new(Vec::new());

extern "C" fn report_int(_name_ptr: *const u8, _name_len: usize, _value: i64) {}

extern "C" fn report_str(name_ptr: *const u8, name_len: usize, str_ptr: *const u8, str_len: usize) {
    unsafe {
        let name =
            std::str::from_utf8(std::slice::from_raw_parts(name_ptr, name_len)).unwrap_or("");
        if name == "__stdout" {
            let s = std::slice::from_raw_parts(str_ptr, str_len);
            CAPTURED.lock().unwrap().extend_from_slice(s);
        }
    }
}

#[test]
fn compile_and_run_for_loop_sum() {
    let src = r#"s = 0
for x in 0..3:
    s += x
print(s)
"#;
    let lib_path = compile_lang_source_to_dylib(src).expect("compile to dylib");
    unsafe {
        let lib = Library::new(&lib_path).expect("load dylib");
        let set_reporters: libloading::Symbol<
            unsafe extern "C" fn(
                extern "C" fn(*const u8, usize, i64),
                extern "C" fn(*const u8, usize, *const u8, usize),
            ),
        > = lib
            .get(b"kayton_set_reporters")
            .expect("find reporters symbol");
        let run: libloading::Symbol<unsafe extern "C" fn()> =
            lib.get(b"run").expect("find run symbol");
        CAPTURED.lock().unwrap().clear();
        set_reporters(report_int, report_str);
        run();
    }
    let output = String::from_utf8(CAPTURED.lock().unwrap().clone()).expect("utf8");
    assert_eq!(output.trim(), "3");
}

#[test]
fn compile_and_run_vec_append_sum() {
    let src = "a = []\na.append(2)\na.append(3)\nres = a.sum()\nprint(res)\n";
    let lib_path = compile_lang_source_to_dylib(src).expect("compile to dylib");
    unsafe {
        let lib = Library::new(&lib_path).expect("load dylib");
        let set_reporters: libloading::Symbol<
            unsafe extern "C" fn(
                extern "C" fn(*const u8, usize, i64),
                extern "C" fn(*const u8, usize, *const u8, usize),
            ),
        > = lib
            .get(b"kayton_set_reporters")
            .expect("find reporters symbol");
        let run: libloading::Symbol<unsafe extern "C" fn()> =
            lib.get(b"run").expect("find run symbol");
        CAPTURED.lock().unwrap().clear();
        set_reporters(report_int, report_str);
        run();
    }
    let output = String::from_utf8(CAPTURED.lock().unwrap().clone()).expect("utf8");
    assert_eq!(output.trim(), "5");
}

#[test]
fn compile_and_run_typed_vec_append_sum() {
    let src = "a: Vec<i64> = []\na.append(2)\na.append(3)\nres = a.sum()\nprint(res)\n";
    let lib_path = compile_lang_source_to_dylib(src).expect("compile to dylib");
    unsafe {
        let lib = Library::new(&lib_path).expect("load dylib");
        let set_reporters: libloading::Symbol<
            unsafe extern "C" fn(
                extern "C" fn(*const u8, usize, i64),
                extern "C" fn(*const u8, usize, *const u8, usize),
            ),
        > = lib
            .get(b"kayton_set_reporters")
            .expect("find reporters symbol");
        let run: libloading::Symbol<unsafe extern "C" fn()> =
            lib.get(b"run").expect("find run symbol");
        CAPTURED.lock().unwrap().clear();
        set_reporters(report_int, report_str);
        run();
    }
    let output = String::from_utf8(CAPTURED.lock().unwrap().clone()).expect("utf8");
    assert_eq!(output.trim(), "5");
}
