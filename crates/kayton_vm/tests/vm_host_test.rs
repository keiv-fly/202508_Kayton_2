use core::sync::atomic::{AtomicUsize, Ordering};
use kayton_api::ErrorKind;
use kayton_vm::{Api, KaytonVm, VmGlobalStrBuf, VmHKayGlobal, VmKaytonContext};

fn vm_and_ctx() -> (KaytonVm, VmKaytonContext) {
    let mut vm = KaytonVm::new();
    let ctx = vm.context();
    (vm, ctx)
}

#[test]
fn test_set_get_primitives_by_name_and_handle() {
    let (vm, mut ctx) = vm_and_ctx();
    let api: &Api = vm.api();

    // u64
    let h_u64: VmHKayGlobal = (api.set_global_u64)(&mut ctx, "n", 42).unwrap();
    assert_eq!((api.get_global_u64)(&mut ctx, "n").unwrap(), 42);
    assert_eq!((api.get_global_u64_by_handle)(&mut ctx, h_u64).unwrap(), 42);

    // overwrite existing
    (api.set_global_u64)(&mut ctx, "n", 100).unwrap();
    assert_eq!((api.get_global_u64)(&mut ctx, "n").unwrap(), 100);

    // u8
    let h_u8: VmHKayGlobal = (api.set_global_u8)(&mut ctx, "b", 7).unwrap();
    assert_eq!((api.get_global_u8)(&mut ctx, "b").unwrap(), 7);
    assert_eq!((api.get_global_u8_by_handle)(&mut ctx, h_u8).unwrap(), 7);

    (api.set_global_u8)(&mut ctx, "b", 9).unwrap();
    assert_eq!((api.get_global_u8)(&mut ctx, "b").unwrap(), 9);

    // f64
    let h_f64: VmHKayGlobal = (api.set_global_f64)(&mut ctx, "x", 3.5).unwrap();
    assert_eq!((api.get_global_f64)(&mut ctx, "x").unwrap(), 3.5);
    assert_eq!(
        (api.get_global_f64_by_handle)(&mut ctx, h_f64).unwrap(),
        3.5
    );

    // f32
    let h_f32: VmHKayGlobal = (api.set_global_f32)(&mut ctx, "y", 2.25).unwrap();
    assert_eq!((api.get_global_f32)(&mut ctx, "y").unwrap(), 2.25);
    assert_eq!(
        (api.get_global_f32_by_handle)(&mut ctx, h_f32).unwrap(),
        2.25
    );
}

#[test]
fn test_static_str_set_get() {
    let (vm, mut ctx) = vm_and_ctx();
    let api: &Api = vm.api();

    let s: &'static str = "hello";
    let h = (api.set_global_static_str)(&mut ctx, "greet", s).unwrap();
    assert_eq!((api.get_global_static_str)(&mut ctx, "greet").unwrap(), s);
    assert_eq!(
        (api.get_global_static_str_by_handle)(&mut ctx, h).unwrap(),
        s
    );

    let s2: &'static str = "world";
    (api.set_global_static_str)(&mut ctx, "greet", s2).unwrap();
    assert_eq!((api.get_global_static_str)(&mut ctx, "greet").unwrap(), s2);
}

#[test]
fn test_str_buf_set_get_copies_without_double_drop() {
    let (vm, mut ctx) = vm_and_ctx();
    let api: &Api = vm.api();

    let buf = VmGlobalStrBuf::new("abc".to_string());
    let h = (api.set_global_str_buf)(&mut ctx, "buf", buf).unwrap();

    // get by name returns a by-value copy; we can drop it safely
    let got1 = (api.get_global_str_buf)(&mut ctx, "buf").unwrap();
    assert_eq!(got1.as_str(), Some("abc"));

    // get by handle also returns a value copy
    let got2 = (api.get_global_str_buf_by_handle)(&mut ctx, h).unwrap();
    assert_eq!(got2.as_str(), Some("abc"));

    // overwrite with new buffer and ensure value changes
    let _h2 =
        (api.set_global_str_buf)(&mut ctx, "buf", VmGlobalStrBuf::new("xyz".to_string())).unwrap();
    let got3 = (api.get_global_str_buf)(&mut ctx, "buf").unwrap();
    assert_eq!(got3.as_str(), Some("xyz"));
}

#[test]
fn test_error_paths_wrong_kind_and_not_found() {
    let (vm, mut ctx) = vm_and_ctx();
    let api: &Api = vm.api();

    // NotFound for missing name
    let err = (api.get_global_u64)(&mut ctx, "missing").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);

    // Create as u64, then try reading as u8 -> Generic/wrong kind
    (api.set_global_u64)(&mut ctx, "v", 1).unwrap();
    let err2 = (api.get_global_u8)(&mut ctx, "v").unwrap_err();
    assert_eq!(err2.kind(), ErrorKind::Generic);
}

#[test]
fn test_dynamic_kinds_register_set_get_and_drop() {
    use core::ffi::c_void;

    static DROPPED: AtomicUsize = AtomicUsize::new(0);
    unsafe extern "C" fn drop_ptr(p: *mut c_void) {
        if !p.is_null() {
            // free the boxed value and mark dropped
            let _boxed: Box<i32> = unsafe { Box::from_raw(p as *mut i32) };
            DROPPED.fetch_add(1, Ordering::SeqCst);
        }
    }

    {
        let (vm, mut ctx) = vm_and_ctx();
        let api: &Api = vm.api();

        let kind = (api.register_dynamic_kind)(&mut ctx, "MyPtr", drop_ptr);

        // allocate a dynamic pointer and store it
        let p: *mut c_void = Box::into_raw(Box::new(123_i32)) as *mut c_void;
        let h = (api.set_global_dyn_ptr)(&mut ctx, kind, "ptr", p).unwrap();

        // get by name returns pointer and kind
        let (got_ptr, got_kind) = (api.get_global_dyn_ptr)(&mut ctx, "ptr").unwrap();
        assert_eq!(got_kind, kind);
        assert_eq!(got_ptr, p);

        // get by handle returns raw pointer
        let got_by_handle = (api.get_global_dyn_ptr_by_handle)(&mut ctx, h).unwrap();
        assert_eq!(got_by_handle, p);

        // overwrite should drop old value once
        let p2: *mut c_void = Box::into_raw(Box::new(456_i32)) as *mut c_void;
        let _h2 = (api.set_global_dyn_ptr)(&mut ctx, kind, "ptr", p2).unwrap();
    }

    // after vm goes out of scope, dynamic stores drop remaining pointers
    // Expect two drops: one from overwrite of first value, one when VM dropped and releases p2
    assert_eq!(DROPPED.load(Ordering::SeqCst), 2);
}
