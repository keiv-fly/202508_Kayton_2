use core::ptr::{null, null_mut};

use kayton_api::api::KaytonApi;
use kayton_api::types::{HKayGlobal, KaytonContext, KaytonStatus};

// Dummy implementations to populate the vtable
extern "C" fn set_u64(
    _ctx: *mut KaytonContext,
    _name_ptr: *const u8,
    _name_len: usize,
    _value: u64,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus {
    unsafe {
        if !out_handle.is_null() {
            *out_handle = HKayGlobal(0xABCD);
        }
    }
    KaytonStatus::Ok
}

extern "C" fn get_u64(
    _ctx: *mut KaytonContext,
    _handle: HKayGlobal,
    out_value: *mut u64,
) -> KaytonStatus {
    unsafe {
        if !out_value.is_null() {
            *out_value = 42;
        }
    }
    KaytonStatus::Ok
}

extern "C" fn set_u8(
    _ctx: *mut KaytonContext,
    _name_ptr: *const u8,
    _name_len: usize,
    _value: u8,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus {
    unsafe {
        if !out_handle.is_null() {
            *out_handle = HKayGlobal(7);
        }
    }
    KaytonStatus::Ok
}

extern "C" fn get_u8(
    _ctx: *mut KaytonContext,
    _handle: HKayGlobal,
    out_value: *mut u8,
) -> KaytonStatus {
    unsafe {
        if !out_value.is_null() {
            *out_value = 7;
        }
    }
    KaytonStatus::Ok
}

extern "C" fn set_f64(
    _ctx: *mut KaytonContext,
    _name_ptr: *const u8,
    _name_len: usize,
    _value: f64,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus {
    unsafe {
        if !out_handle.is_null() {
            *out_handle = HKayGlobal(1);
        }
    }
    KaytonStatus::Ok
}

extern "C" fn get_f64(
    _ctx: *mut KaytonContext,
    _handle: HKayGlobal,
    out_value: *mut f64,
) -> KaytonStatus {
    unsafe {
        if !out_value.is_null() {
            *out_value = 3.14;
        }
    }
    KaytonStatus::Ok
}

extern "C" fn set_f32(
    _ctx: *mut KaytonContext,
    _name_ptr: *const u8,
    _name_len: usize,
    _value: f32,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus {
    unsafe {
        if !out_handle.is_null() {
            *out_handle = HKayGlobal(2);
        }
    }
    KaytonStatus::Ok
}

extern "C" fn get_f32(
    _ctx: *mut KaytonContext,
    _handle: HKayGlobal,
    out_value: *mut f32,
) -> KaytonStatus {
    unsafe {
        if !out_value.is_null() {
            *out_value = 2.71;
        }
    }
    KaytonStatus::Ok
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
    let mut handle = HKayGlobal(0);
    let status = (api_ref.set_global_u64)(
        &mut ctx as *mut KaytonContext,
        b"x".as_ptr(),
        1,
        123,
        &mut handle,
    );
    assert_eq!(status, KaytonStatus::Ok);

    let mut out_u64 = 0u64;
    let status = (api_ref.get_global_u64)(&mut ctx as *mut KaytonContext, handle, &mut out_u64);
    assert_eq!(status, KaytonStatus::Ok);
    assert_eq!(out_u64, 42);

    // Clean up heap allocation
    unsafe {
        drop(Box::from_raw(api_ptr_const as *mut KaytonApi));
    }
}
