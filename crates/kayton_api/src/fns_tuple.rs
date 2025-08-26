use crate::{HKayRef, KaytonContext, KaytonError};

/// Create/overwrite a named tuple global from a slice of handles.
/// Returns the global handle for the tuple (kind = KIND_TUPLE).
pub type SetGlobalTupleFromHandlesFn = fn(
    ctx: &mut KaytonContext,
    name: &str,
    items: *const HKayRef,
    len: usize,
) -> Result<HKayRef, KaytonError>;

/// Length of a tuple (by name).
pub type GetGlobalTupleLenFn =
    fn(ctx: &mut KaytonContext, name: &str) -> Result<usize, KaytonError>;

/// Length of a tuple (fast path by handle).
pub type GetTupleLenByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayRef) -> Result<usize, KaytonError>;

/// Random access: get element i as a handle (by name).
pub type GetGlobalTupleItemFn =
    fn(ctx: &mut KaytonContext, name: &str, index: usize) -> Result<HKayRef, KaytonError>;

/// Random access: get element i as a handle (fast path by handle).
pub type GetGlobalTupleItemByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayRef, index: usize) -> Result<HKayRef, KaytonError>;

/// Bulk read: copy tuple elements into caller's buffer. Returns written count.
pub type ReadTupleIntoSliceByHandleFn = fn(
    ctx: &mut KaytonContext,
    h: HKayRef,
    out: *mut HKayRef,
    cap: usize,
) -> Result<usize, KaytonError>;
