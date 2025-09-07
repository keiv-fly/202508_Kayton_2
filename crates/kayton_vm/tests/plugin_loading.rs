use kayton_vm::KaytonVm;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn load_plugin_and_call_function() {
    // Build the test plugin
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let status = Command::new("cargo")
        .args(["build", "-p", "plugin_hello"])
        .current_dir(&workspace)
        .status()
        .expect("failed to build plugin");
    assert!(status.success());

    // Determine plugin path within workspace target directory
    #[cfg(target_os = "linux")]
    let libname = "libplugin_hello.so";
    #[cfg(target_os = "macos")]
    let libname = "libplugin_hello.dylib";
    #[cfg(target_os = "windows")]
    let libname = "plugin_hello.dll";
    let plugin_path = workspace.join("target").join("debug").join(libname);
    assert!(plugin_path.exists(), "plugin not found at {:?}", plugin_path);

    let mut vm = KaytonVm::new();
    vm.load_plugin_from_path(&plugin_path)
        .expect("plugin should load");

    // Retrieve the function pointer and call it
    let ptr = vm.get_function_ptr("add").expect("fn ptr not found");
    let func: unsafe extern "Rust" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(ptr) };
    let result = unsafe { func(2, 3) };
    assert_eq!(result, 5);

    // Verify type metadata registration
    let meta = vm.get_type_meta("MyType").expect("type not registered");
    assert_eq!(meta.size, core::mem::size_of::<i64>());
}
