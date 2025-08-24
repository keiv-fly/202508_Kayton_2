use crate::{HKayGlobal, KaytonContext, KaytonStatus};

/// Set/overwrite a named u64 global, return handle.
pub type SetGlobalU64Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: u64) -> Result<HKayGlobal, KaytonStatus>;

/// Read an existing u64 global by name.
pub type GetGlobalU64Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<u64, KaytonStatus>;

/// Set/overwrite a named u8 global, return handle.
pub type SetGlobalU8Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: u8) -> Result<HKayGlobal, KaytonStatus>;

/// Read an existing u8 global by name.
pub type GetGlobalU8Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<u8, KaytonStatus>;
