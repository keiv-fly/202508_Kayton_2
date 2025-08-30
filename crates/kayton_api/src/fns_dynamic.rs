use core::ffi::c_void;

use crate::kinds::KindId;
use crate::{HKayRef, KaytonContext, KaytonError};

/// Drop fn for dynamic-kind pointers
pub type DynDropFn = unsafe extern "C" fn(ptr: *mut c_void);

/// Register a new dynamic kind. Returns its KindId.
pub type RegisterDynamicKindFn =
    fn(ctx: &mut KaytonContext, name: &'static str, drop_fn: DynDropFn) -> KindId;

/// Set/overwrite a named dynamic pointer, return handle.
pub type SetGlobalDynPtrFn = fn(
    ctx: &mut KaytonContext,
    kind: KindId,
    name: &str,
    value: *mut c_void,
) -> Result<HKayRef, KaytonError>;

/// Resolve by name and get the raw pointer (and KindId).
pub type GetGlobalDynPtrFn =
    fn(ctx: &mut KaytonContext, name: &str) -> Result<(*mut c_void, KindId), KaytonError>;

/// Fast path by handle
pub type GetGlobalDynPtrByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayRef) -> Result<*mut c_void, KaytonError>;
