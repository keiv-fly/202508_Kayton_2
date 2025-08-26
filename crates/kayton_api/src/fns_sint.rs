use crate::{HKayGlobal, KaytonContext, KaytonError};

/// Set/overwrite a named i8 global, return handle.
pub type SetGlobalI8Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: i8) -> Result<HKayGlobal, KaytonError>;

/// Read an existing i8 global by name.
pub type GetGlobalI8Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<i8, KaytonError>;

/// Fast path: read an i8 global by handle.
pub type GetGlobalI8ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<i8, KaytonError>;

/// Set/overwrite a named i16 global, return handle.
pub type SetGlobalI16Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: i16) -> Result<HKayGlobal, KaytonError>;

/// Read an existing i16 global by name.
pub type GetGlobalI16Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<i16, KaytonError>;

/// Fast path: read an i16 global by handle.
pub type GetGlobalI16ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<i16, KaytonError>;

/// Set/overwrite a named i32 global, return handle.
pub type SetGlobalI32Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: i32) -> Result<HKayGlobal, KaytonError>;

/// Read an existing i32 global by name.
pub type GetGlobalI32Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<i32, KaytonError>;

/// Fast path: read an i32 global by handle.
pub type GetGlobalI32ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<i32, KaytonError>;

/// Set/overwrite a named i64 global, return handle.
pub type SetGlobalI64Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: i64) -> Result<HKayGlobal, KaytonError>;

/// Read an existing i64 global by name.
pub type GetGlobalI64Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<i64, KaytonError>;

/// Fast path: read an i64 global by handle.
pub type GetGlobalI64ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<i64, KaytonError>;

/// Set/overwrite a named i128 global, return handle.
pub type SetGlobalI128Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: i128) -> Result<HKayGlobal, KaytonError>;

/// Read an existing i128 global by name.
pub type GetGlobalI128Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<i128, KaytonError>;

/// Fast path: read an i128 global by handle.
pub type GetGlobalI128ByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<i128, KaytonError>;

/// Set/overwrite a named isize global, return handle.
pub type SetGlobalIsizeFn =
    fn(ctx: &mut KaytonContext, name: &str, value: isize) -> Result<HKayGlobal, KaytonError>;

/// Read an existing isize global by name.
pub type GetGlobalIsizeFn = fn(ctx: &mut KaytonContext, name: &str) -> Result<isize, KaytonError>;

/// Fast path: read an isize global by handle.
pub type GetGlobalIsizeByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<isize, KaytonError>;

/// Set/overwrite a named bool global, return handle.
pub type SetGlobalBoolFn =
    fn(ctx: &mut KaytonContext, name: &str, value: bool) -> Result<HKayGlobal, KaytonError>;

/// Read an existing bool global by name.
pub type GetGlobalBoolFn = fn(ctx: &mut KaytonContext, name: &str) -> Result<bool, KaytonError>;

/// Fast path: read a bool global by handle.
pub type GetGlobalBoolByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<bool, KaytonError>;
