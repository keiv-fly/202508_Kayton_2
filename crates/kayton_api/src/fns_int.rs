use crate::{HKayGlobal, KaytonContext, KaytonError};

/// Set/overwrite a named u64 global, return handle.
pub type SetGlobalU64Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: u64) -> Result<HKayGlobal, KaytonError>;

/// Read an existing u64 global by name.
pub type GetGlobalU64Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<u64, KaytonError>;

/// Fast path: read a u64 global by handle.
pub type GetGlobalU64ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<u64, KaytonError>;

/// Set/overwrite a named u8 global, return handle.
pub type SetGlobalU8Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: u8) -> Result<HKayGlobal, KaytonError>;

/// Read an existing u8 global by name.
pub type GetGlobalU8Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<u8, KaytonError>;

/// Fast path: read a u8 global by handle.
pub type GetGlobalU8ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<u8, KaytonError>;
