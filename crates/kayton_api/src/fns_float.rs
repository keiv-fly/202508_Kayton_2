use crate::{HKayGlobal, KaytonContext, KaytonStatus};

/// Set/overwrite a named f64 global, return handle via out param.
pub type SetGlobalF64Fn = extern "C" fn(
    ctx: *mut KaytonContext,
    name_ptr: *const u8,
    name_len: usize,
    value: f64,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus;

/// Read an existing f64 global by handle.
pub type GetGlobalF64Fn =
    extern "C" fn(ctx: *mut KaytonContext, handle: HKayGlobal, out_value: *mut f64) -> KaytonStatus;

/// Set/overwrite a named f32 global, return handle via out param.
pub type SetGlobalF32Fn = extern "C" fn(
    ctx: *mut KaytonContext,
    name_ptr: *const u8,
    name_len: usize,
    value: f32,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus;

/// Read an existing f32 global by handle.
pub type GetGlobalF32Fn =
    extern "C" fn(ctx: *mut KaytonContext, handle: HKayGlobal, out_value: *mut f32) -> KaytonStatus;
