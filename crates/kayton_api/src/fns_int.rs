use crate::{HKayGlobal, KaytonContext, KaytonStatus};

/// Set/overwrite a named u64 global, return handle via out param.
pub type SetGlobalU64Fn = extern "C" fn(
    ctx: *mut KaytonContext,
    name_ptr: *const u8,
    name_len: usize,
    value: u64,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus;

/// Read an existing u64 global by handle.
pub type GetGlobalU64Fn =
    extern "C" fn(ctx: *mut KaytonContext, handle: HKayGlobal, out_value: *mut u64) -> KaytonStatus;

/// Set/overwrite a named u8 global, return handle via out param.
pub type SetGlobalU8Fn = extern "C" fn(
    ctx: *mut KaytonContext,
    name_ptr: *const u8,
    name_len: usize,
    value: u8,
    out_handle: *mut HKayGlobal,
) -> KaytonStatus;

/// Read an existing u8 global by handle.
pub type GetGlobalU8Fn =
    extern "C" fn(ctx: *mut KaytonContext, handle: HKayGlobal, out_value: *mut u8) -> KaytonStatus;
