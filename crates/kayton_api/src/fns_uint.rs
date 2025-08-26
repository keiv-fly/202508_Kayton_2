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

/// Set/overwrite a named u32 global, return handle.
pub type SetGlobalU32Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: u32) -> Result<HKayGlobal, KaytonError>;

/// Read an existing u32 global by name.
pub type GetGlobalU32Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<u32, KaytonError>;

/// Fast path: read a u32 global by handle.
pub type GetGlobalU32ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<u32, KaytonError>;

/// Set/overwrite a named u16 global, return handle.
pub type SetGlobalU16Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: u16) -> Result<HKayGlobal, KaytonError>;

/// Read an existing u16 global by name.
pub type GetGlobalU16Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<u16, KaytonError>;

/// Fast path: read a u16 global by handle.
pub type GetGlobalU16ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<u16, KaytonError>;

/// Set/overwrite a named u128 global, return handle.
pub type SetGlobalU128Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: u128) -> Result<HKayGlobal, KaytonError>;

/// Read an existing u128 global by name.
pub type GetGlobalU128Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<u128, KaytonError>;

/// Fast path: read a u128 global by handle.
pub type GetGlobalU128ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<u128, KaytonError>;

/// Set/overwrite a named usize global, return handle.
pub type SetGlobalUsizeFn =
    fn(ctx: &mut KaytonContext, name: &str, value: usize) -> Result<HKayGlobal, KaytonError>;

/// Read an existing usize global by name.
pub type GetGlobalUsizeFn = fn(ctx: &mut KaytonContext, name: &str) -> Result<usize, KaytonError>;

/// Fast path: read a usize global by handle.
pub type GetGlobalUsizeByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<usize, KaytonError>;
