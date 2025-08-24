use core::ptr::{null, null_mut};

use kayton_api::api::KaytonApi;
use kayton_api::types::{HKayGlobal, KaytonContext, KaytonStatus};

// Dummy implementations to populate the vtable
fn set_u64(_ctx: &mut KaytonContext, _name: &str, _value: u64) -> Result<HKayGlobal, KaytonStatus> {
    Ok(HKayGlobal(0xABCD))
}

fn get_u64(_ctx: &mut KaytonContext, _name: &str) -> Result<u64, KaytonStatus> {
    Ok(42)
}

fn set_u8(_ctx: &mut KaytonContext, _name: &str, _value: u8) -> Result<HKayGlobal, KaytonStatus> {
    Ok(HKayGlobal(7))
}

fn get_u8(_ctx: &mut KaytonContext, _name: &str) -> Result<u8, KaytonStatus> {
    Ok(7)
}

fn set_f64(_ctx: &mut KaytonContext, _name: &str, _value: f64) -> Result<HKayGlobal, KaytonStatus> {
    Ok(HKayGlobal(1))
}

fn get_f64(_ctx: &mut KaytonContext, _name: &str) -> Result<f64, KaytonStatus> {
    Ok(3.14)
}

fn set_f32(_ctx: &mut KaytonContext, _name: &str, _value: f32) -> Result<HKayGlobal, KaytonStatus> {
    Ok(HKayGlobal(2))
}

fn get_f32(_ctx: &mut KaytonContext, _name: &str) -> Result<f32, KaytonStatus> {
    Ok(2.71)
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
