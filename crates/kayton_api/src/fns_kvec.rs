use crate::{HKayRef, KVec, KaytonContext, KaytonError};

/// Set/overwrite a named KVec global, return handle.
pub type SetGlobalKVecFn =
    fn(ctx: &mut KaytonContext, name: &str, value: KVec) -> Result<HKayRef, KaytonError>;

/// Read an existing KVec global by name.
pub type GetGlobalKVecFn = fn(ctx: &mut KaytonContext, name: &str) -> Result<KVec, KaytonError>;

/// Fast path: read a KVec global by handle.
pub type GetGlobalKVecByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayRef) -> Result<KVec, KaytonError>;

/// Drop a KVec by handle.
pub type DropGlobalKVecFn = fn(ctx: &mut KaytonContext, h: HKayRef) -> Result<(), KaytonError>;
