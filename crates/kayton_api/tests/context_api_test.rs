use core::ptr::{null, null_mut};

use core::ffi::c_void;
use kayton_api::api::KaytonApi;
use kayton_api::types::{GlobalStrBuf, HKayGlobal, KaytonContext, KaytonError};

// Dummy implementations to populate the vtable
fn set_u64(_ctx: &mut KaytonContext, _name: &str, _value: u64) -> Result<HKayGlobal, KaytonError> {
    Ok(HKayGlobal(0xABCD))
}

fn get_u64(_ctx: &mut KaytonContext, _name: &str) -> Result<u64, KaytonError> {
    Ok(42)
}

fn set_u8(_ctx: &mut KaytonContext, _name: &str, _value: u8) -> Result<HKayGlobal, KaytonError> {
    Ok(HKayGlobal(7))
}

fn get_u8(_ctx: &mut KaytonContext, _name: &str) -> Result<u8, KaytonError> {
    Ok(7)
}

fn set_f64(_ctx: &mut KaytonContext, _name: &str, _value: f64) -> Result<HKayGlobal, KaytonError> {
    Ok(HKayGlobal(1))
}

fn get_f64(_ctx: &mut KaytonContext, _name: &str) -> Result<f64, KaytonError> {
    Ok(3.14)
}

fn set_f32(_ctx: &mut KaytonContext, _name: &str, _value: f32) -> Result<HKayGlobal, KaytonError> {
    Ok(HKayGlobal(2))
}

fn get_f32(_ctx: &mut KaytonContext, _name: &str) -> Result<f32, KaytonError> {
    Ok(2.71)
}

fn set_static_str(
    _ctx: &mut KaytonContext,
    _name: &str,
    _value: &'static str,
) -> Result<HKayGlobal, KaytonError> {
    Ok(HKayGlobal(0x1234))
}

fn get_static_str(_ctx: &mut KaytonContext, _name: &str) -> Result<&'static str, KaytonError> {
    Ok("test_string")
}

fn set_global_str_buf(
    _ctx: &mut KaytonContext,
    _name: &str,
    _value: GlobalStrBuf,
) -> Result<HKayGlobal, KaytonError> {
    Ok(HKayGlobal(0x5678))
}

fn get_global_str_buf(_ctx: &mut KaytonContext, _name: &str) -> Result<GlobalStrBuf, KaytonError> {
    let test_str = "test_buffer".to_string();
    Ok(GlobalStrBuf::new(test_str))
}

fn register_dynamic_kind(
    _ctx: &mut KaytonContext,
    _name: &'static str,
    _drop_fn: unsafe extern "C" fn(*mut c_void),
) -> u32 {
    1000
}

fn set_global_dyn_ptr(
    _ctx: &mut KaytonContext,
    _kind: u32,
    _name: &str,
    _value: *mut c_void,
) -> Result<HKayGlobal, KaytonError> {
    Ok(HKayGlobal(0xDEAD_BEEF))
}

fn get_global_dyn_ptr(
    _ctx: &mut KaytonContext,
    _name: &str,
) -> Result<(*mut c_void, u32), KaytonError> {
    Ok((core::ptr::null_mut(), 1000))
}

fn get_global_dyn_ptr_by_handle(
    _ctx: &mut KaytonContext,
    _h: HKayGlobal,
) -> Result<*mut c_void, KaytonError> {
    Ok(core::ptr::null_mut())
}

#[test]
fn context_api_accessor_and_calls() {
    let api = KaytonApi {
        size: core::mem::size_of::<KaytonApi>() as u64,
        set_global_u64: set_u64,
        get_global_u64: get_u64,
        set_global_u8: set_u8,
        get_global_u8: get_u8,
        set_global_f64: set_f64,
        get_global_f64: get_f64,
        set_global_f32: set_f32,
        get_global_f32: get_f32,
        set_global_static_str: set_static_str,
        get_global_static_str: get_static_str,
        set_global_str_buf: set_global_str_buf,
        get_global_str_buf: get_global_str_buf,
        register_dynamic_kind: register_dynamic_kind,
        set_global_dyn_ptr: set_global_dyn_ptr,
        get_global_dyn_ptr: get_global_dyn_ptr,
        get_global_dyn_ptr_by_handle: get_global_dyn_ptr_by_handle,
        _reserved0: null(),
        _reserved1: null(),
        _reserved2: null(),
        _reserved3: null(),
    };

    let api_box = Box::new(api);
    let api_ptr_const = &*api_box as *const KaytonApi; // save for drop later

    let mut ctx = KaytonContext {
        abi_version: 1,
        host_data: null_mut(),
        api: Box::into_raw(api_box),
    };

    // Work with a raw copy of the pointer to avoid borrowing `ctx`
    let api_ptr = ctx.api;
    let api_ref = unsafe { &*api_ptr };
    assert_eq!(api_ref.size, core::mem::size_of::<KaytonApi>() as u64);

    // Exercise a few function pointers
    let handle = (api_ref.set_global_u64)(&mut ctx, "x", 123).unwrap();
    assert_eq!(handle, HKayGlobal(0xABCD));

    let out_u64 = (api_ref.get_global_u64)(&mut ctx, "x").unwrap();
    assert_eq!(out_u64, 42);

    // Clean up heap allocation
    unsafe {
        drop(Box::from_raw(api_ptr_const as *mut KaytonApi));
    }
}
