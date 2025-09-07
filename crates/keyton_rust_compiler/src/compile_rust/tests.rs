use super::*;
use libloading::Library;
use std::sync::Mutex;

static CAPTURED_STDOUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static CAPTURED_LAST_INT: Mutex<Option<i64>> = Mutex::new(None);

extern "C" fn report_int(name_ptr: *const u8, name_len: usize, value: i64) {
    unsafe {
        let name =
            std::str::from_utf8(std::slice::from_raw_parts(name_ptr, name_len)).unwrap_or("");
        if name == "__last" {
            CAPTURED_LAST_INT.lock().unwrap().replace(value);
        }
    }
}

extern "C" fn report_str(name_ptr: *const u8, name_len: usize, str_ptr: *const u8, str_len: usize) {
    unsafe {
        let name =
            std::str::from_utf8(std::slice::from_raw_parts(name_ptr, name_len)).unwrap_or("");
        let s = std::slice::from_raw_parts(str_ptr, str_len);
        if name == "__stdout" {
            CAPTURED_STDOUT.lock().unwrap().extend_from_slice(s);
        } else if name == "__last" {
            CAPTURED_STDOUT.lock().unwrap().extend_from_slice(s);
        }
    }
}

fn take_last_int() -> i64 {
    if let Some(v) = CAPTURED_LAST_INT.lock().unwrap().take() {
        return v;
    }
    let s = {
        let mut lock = CAPTURED_STDOUT.lock().unwrap();
        let s = String::from_utf8(lock.clone()).expect("utf8");
        lock.clear();
        s
    };
    s.trim().parse().expect("int")
}

#[test]
fn compile_and_run_print_all_values() {
    let src = r#"print("Hello from dylib")
x = 12
x = x + 30
print(x)
fn my_sum(x, y):
    x + y
print(my_sum(1,2))"#;
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let output = {
        let mut lock = CAPTURED_STDOUT.lock().unwrap();
        let s = String::from_utf8(lock.clone()).expect("utf8");
        lock.clear();
        s
    };
    assert_eq!(output.trim_end(), "Hello from dylib\n42\n3");
}

#[test]
fn compile_and_run_math() {
    let src = r#"x = 12
x = x + 30
x
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let value = take_last_int();
    assert_eq!(value, 42);
}

#[test]
fn compile_and_run_user_function_sum() {
    let src = r#"x = 1
y = 2

fn my_sum(x, y):
    x + y

z = my_sum(x,y)
z
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let value = take_last_int();
    assert_eq!(value, 3);
}

#[test]
fn compile_and_run_for_loop_sum() {
    let src = r#"s = 0
for x in 0..3:
    s += x
s
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let value = take_last_int();
    assert_eq!(value, 3);
}

#[test]
fn compile_and_run_vec_append_sum() {
    let src = "a = []\na.append(2)\na.append(3)\nres = a.sum()\nres\n";
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let value = take_last_int();
    assert_eq!(value, 5);
}

#[test]
fn compile_and_run_if_else_true() {
    let src = r#"x = True
y = 0
if x:
    y = 1
else:
    y = 2
y
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let value = take_last_int();
    assert_eq!(value, 1);
}

#[test]
fn compile_and_run_if_else_false() {
    let src = r#"x = False
y = 0
if x:
    y = 1
else:
    y = 2
y
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let value = take_last_int();
    assert_eq!(value, 2);
}

#[test]
fn compile_and_run_typed_vec_append_sum() {
    let src = "a: Vec<i64> = []\na.append(2)\na.append(3)\nres = a.sum()\nres\n";
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
        CAPTURED_STDOUT.lock().unwrap().clear();
        CAPTURED_LAST_INT.lock().unwrap().take();
        set_reporters(report_int, report_str);
        run();
    }
    let value = take_last_int();
    assert_eq!(value, 5);
}
