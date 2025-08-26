use core::ptr::null_mut;

use core::ffi::c_void;
use kayton_api::api::KaytonApi;
use kayton_api::types::{GlobalStrBuf, HKayRef, KaytonContext, KaytonError};

// Dummy implementations to populate the vtable
fn set_u64(_ctx: &mut KaytonContext, _name: &str, _value: u64) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef {
        kind: 0,
        index: 0xABCD,
    })
}

fn get_u64(_ctx: &mut KaytonContext, _name: &str) -> Result<u64, KaytonError> {
    Ok(42)
}

fn set_u8(_ctx: &mut KaytonContext, _name: &str, _value: u8) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 7 })
}

fn get_u8(_ctx: &mut KaytonContext, _name: &str) -> Result<u8, KaytonError> {
    Ok(7)
}

fn set_f64(_ctx: &mut KaytonContext, _name: &str, _value: f64) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 1 })
}

fn get_f64(_ctx: &mut KaytonContext, _name: &str) -> Result<f64, KaytonError> {
    Ok(3.14)
}

fn set_f32(_ctx: &mut KaytonContext, _name: &str, _value: f32) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 2 })
}

fn get_f32(_ctx: &mut KaytonContext, _name: &str) -> Result<f32, KaytonError> {
    Ok(2.71)
}

fn set_static_str(
    _ctx: &mut KaytonContext,
    _name: &str,
    _value: &'static str,
) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef {
        kind: 0,
        index: 0x1234,
    })
}

fn get_static_str(_ctx: &mut KaytonContext, _name: &str) -> Result<&'static str, KaytonError> {
    Ok("test_string")
}

fn set_global_str_buf(
    _ctx: &mut KaytonContext,
    _name: &str,
    _value: GlobalStrBuf,
) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef {
        kind: 0,
        index: 0x5678,
    })
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
) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef {
        kind: 0,
        index: 0xBEEF,
    })
}

fn get_global_dyn_ptr(
    _ctx: &mut KaytonContext,
    _name: &str,
) -> Result<(*mut c_void, u32), KaytonError> {
    Ok((core::ptr::null_mut(), 1000))
}

fn get_global_dyn_ptr_by_handle(
    _ctx: &mut KaytonContext,
    _h: HKayRef,
) -> Result<*mut c_void, KaytonError> {
    Ok(core::ptr::null_mut())
}

fn get_u64_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<u64, KaytonError> {
    Ok(42)
}

fn get_u8_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<u8, KaytonError> {
    Ok(7)
}

fn get_f64_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<f64, KaytonError> {
    Ok(3.14)
}

fn get_f32_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<f32, KaytonError> {
    Ok(2.71)
}

fn get_static_str_by_handle(
    _ctx: &mut KaytonContext,
    _h: HKayRef,
) -> Result<&'static str, KaytonError> {
    Ok("test_string")
}

fn get_str_buf_by_handle(
    _ctx: &mut KaytonContext,
    _h: HKayRef,
) -> Result<GlobalStrBuf, KaytonError> {
    let test_str = "test_buffer".to_string();
    Ok(GlobalStrBuf::new(test_str))
}

// ---- New integer and bool dummies ----
fn set_u32(_ctx: &mut KaytonContext, _name: &str, _value: u32) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 1 })
}
fn get_u32(_ctx: &mut KaytonContext, _name: &str) -> Result<u32, KaytonError> {
    Ok(32)
}
fn get_u32_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<u32, KaytonError> {
    Ok(32)
}

fn set_u16(_ctx: &mut KaytonContext, _name: &str, _value: u16) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 2 })
}
fn get_u16(_ctx: &mut KaytonContext, _name: &str) -> Result<u16, KaytonError> {
    Ok(16)
}
fn get_u16_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<u16, KaytonError> {
    Ok(16)
}

fn set_u128(_ctx: &mut KaytonContext, _name: &str, _value: u128) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 3 })
}
fn get_u128(_ctx: &mut KaytonContext, _name: &str) -> Result<u128, KaytonError> {
    Ok(128)
}
fn get_u128_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<u128, KaytonError> {
    Ok(128)
}

fn set_usize(_ctx: &mut KaytonContext, _name: &str, _value: usize) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 4 })
}
fn get_usize(_ctx: &mut KaytonContext, _name: &str) -> Result<usize, KaytonError> {
    Ok(64)
}
fn get_usize_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<usize, KaytonError> {
    Ok(64)
}

fn set_i8(_ctx: &mut KaytonContext, _name: &str, _value: i8) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 5 })
}
fn get_i8(_ctx: &mut KaytonContext, _name: &str) -> Result<i8, KaytonError> {
    Ok(-8)
}
fn get_i8_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<i8, KaytonError> {
    Ok(-8)
}

fn set_i16(_ctx: &mut KaytonContext, _name: &str, _value: i16) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 6 })
}
fn get_i16(_ctx: &mut KaytonContext, _name: &str) -> Result<i16, KaytonError> {
    Ok(-16)
}
fn get_i16_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<i16, KaytonError> {
    Ok(-16)
}

fn set_i32(_ctx: &mut KaytonContext, _name: &str, _value: i32) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 7 })
}
fn get_i32(_ctx: &mut KaytonContext, _name: &str) -> Result<i32, KaytonError> {
    Ok(-32)
}
fn get_i32_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<i32, KaytonError> {
    Ok(-32)
}

fn set_i64(_ctx: &mut KaytonContext, _name: &str, _value: i64) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 8 })
}
fn get_i64(_ctx: &mut KaytonContext, _name: &str) -> Result<i64, KaytonError> {
    Ok(-64)
}
fn get_i64_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<i64, KaytonError> {
    Ok(-64)
}

fn set_i128(_ctx: &mut KaytonContext, _name: &str, _value: i128) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 9 })
}
fn get_i128(_ctx: &mut KaytonContext, _name: &str) -> Result<i128, KaytonError> {
    Ok(-128)
}
fn get_i128_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<i128, KaytonError> {
    Ok(-128)
}

fn set_isize(_ctx: &mut KaytonContext, _name: &str, _value: isize) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 10 })
}
fn get_isize(_ctx: &mut KaytonContext, _name: &str) -> Result<isize, KaytonError> {
    Ok(-32)
}
fn get_isize_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<isize, KaytonError> {
    Ok(-32)
}

fn set_bool(_ctx: &mut KaytonContext, _name: &str, _value: bool) -> Result<HKayRef, KaytonError> {
    Ok(HKayRef { kind: 0, index: 11 })
}
fn get_bool(_ctx: &mut KaytonContext, _name: &str) -> Result<bool, KaytonError> {
    Ok(true)
}
fn get_bool_by_handle(_ctx: &mut KaytonContext, _h: HKayRef) -> Result<bool, KaytonError> {
    Ok(true)
}

#[test]
fn context_api_accessor_and_calls() {
    // Dummy interners and tuples (not exercised here)
    fn intern_u64(_ctx: &mut KaytonContext, _v: u64) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 1, index: 1 })
    }
    fn intern_u8(_ctx: &mut KaytonContext, _v: u8) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 2, index: 2 })
    }
    fn intern_f64(_ctx: &mut KaytonContext, _v: f64) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 3, index: 3 })
    }
    fn intern_f32(_ctx: &mut KaytonContext, _v: f32) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 4, index: 4 })
    }
    fn intern_static_str(
        _ctx: &mut KaytonContext,
        _s: &'static str,
    ) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 5, index: 5 })
    }
    fn intern_str_buf(_ctx: &mut KaytonContext, _s: &str) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 6, index: 6 })
    }
    fn intern_dyn_ptr(
        _ctx: &mut KaytonContext,
        _k: u32,
        _p: *mut c_void,
    ) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 7, index: 7 })
    }

    fn set_tuple_from_handles(
        _ctx: &mut KaytonContext,
        _name: &str,
        _items: *const HKayRef,
        _len: usize,
    ) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 8, index: 0 })
    }
    fn get_global_tuple_len(_ctx: &mut KaytonContext, _name: &str) -> Result<usize, KaytonError> {
        Ok(0)
    }
    fn get_tuple_len_by_handle(
        _ctx: &mut KaytonContext,
        _h: HKayRef,
    ) -> Result<usize, KaytonError> {
        Ok(0)
    }
    fn get_global_tuple_item(
        _ctx: &mut KaytonContext,
        _name: &str,
        _idx: usize,
    ) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 0, index: 0 })
    }
    fn get_tuple_item_by_index(
        _ctx: &mut KaytonContext,
        _h: HKayRef,
        _idx: usize,
    ) -> Result<HKayRef, KaytonError> {
        Ok(HKayRef { kind: 0, index: 0 })
    }
    fn read_tuple_into_slice_by_handle(
        _ctx: &mut KaytonContext,
        _h: HKayRef,
        _out: *mut HKayRef,
        _cap: usize,
    ) -> Result<usize, KaytonError> {
        Ok(0)
    }

    let api = KaytonApi {
        size: core::mem::size_of::<KaytonApi>() as u64,
        set_global_u64: set_u64,
        get_global_u64: get_u64,
        get_global_u64_by_handle: get_u64_by_handle,
        set_global_u8: set_u8,
        get_global_u8: get_u8,
        get_global_u8_by_handle: get_u8_by_handle,
        set_global_f64: set_f64,
        get_global_f64: get_f64,
        get_global_f64_by_handle: get_f64_by_handle,
        set_global_f32: set_f32,
        get_global_f32: get_f32,
        get_global_f32_by_handle: get_f32_by_handle,
        set_global_static_str: set_static_str,
        get_global_static_str: get_static_str,
        get_global_static_str_by_handle: get_static_str_by_handle,
        set_global_str_buf: set_global_str_buf,
        get_global_str_buf: get_global_str_buf,
        get_global_str_buf_by_handle: get_str_buf_by_handle,
        register_dynamic_kind: register_dynamic_kind,
        set_global_dyn_ptr: set_global_dyn_ptr,
        get_global_dyn_ptr: get_global_dyn_ptr,
        get_global_dyn_ptr_by_handle: get_global_dyn_ptr_by_handle,
        set_global_u32: set_u32,
        get_global_u32: get_u32,
        get_global_u32_by_handle: get_u32_by_handle,
        set_global_u16: set_u16,
        get_global_u16: get_u16,
        get_global_u16_by_handle: get_u16_by_handle,
        set_global_u128: set_u128,
        get_global_u128: get_u128,
        get_global_u128_by_handle: get_u128_by_handle,
        set_global_usize: set_usize,
        get_global_usize: get_usize,
        get_global_usize_by_handle: get_usize_by_handle,
        set_global_i8: set_i8,
        get_global_i8: get_i8,
        get_global_i8_by_handle: get_i8_by_handle,
        set_global_i16: set_i16,
        get_global_i16: get_i16,
        get_global_i16_by_handle: get_i16_by_handle,
        set_global_i32: set_i32,
        get_global_i32: get_i32,
        get_global_i32_by_handle: get_i32_by_handle,
        set_global_i64: set_i64,
        get_global_i64: get_i64,
        get_global_i64_by_handle: get_i64_by_handle,
        set_global_i128: set_i128,
        get_global_i128: get_i128,
        get_global_i128_by_handle: get_i128_by_handle,
        set_global_isize: set_isize,
        get_global_isize: get_isize,
        get_global_isize_by_handle: get_isize_by_handle,
        set_global_bool: set_bool,
        get_global_bool: get_bool,
        get_global_bool_by_handle: get_bool_by_handle,
        intern_u64,
        intern_u8,
        intern_f64,
        intern_f32,
        intern_static_str,
        intern_str_buf,
        intern_dyn_ptr,
        set_global_tuple_from_handles: set_tuple_from_handles,
        get_global_tuple_len: get_global_tuple_len,
        get_tuple_len_by_handle: get_tuple_len_by_handle,
        get_global_tuple_item: get_global_tuple_item,
        get_tuple_item_by_index: get_tuple_item_by_index,
        read_tuple_into_slice_by_handle: read_tuple_into_slice_by_handle,
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
    assert_eq!(
        handle,
        HKayRef {
            kind: 0,
            index: 0xABCD
        }
    );

    let out_u64 = (api_ref.get_global_u64)(&mut ctx, "x").unwrap();
    assert_eq!(out_u64, 42);

    // Clean up heap allocation
    unsafe {
        drop(Box::from_raw(api_ptr_const as *mut KaytonApi));
    }
}
