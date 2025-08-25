use crate::{HKayGlobal, KaytonContext, KaytonError, types::GlobalStrBuf};

/// Set/overwrite a named static string global, return handle.
pub type SetGlobalStaticStrFn =
    fn(ctx: &mut KaytonContext, name: &str, value: &'static str) -> Result<HKayGlobal, KaytonError>;

/// Read an existing static string global by name.
pub type GetGlobalStaticStrFn =
    fn(ctx: &mut KaytonContext, name: &str) -> Result<&'static str, KaytonError>;

/// Fast path: read a static string global by handle.
pub type GetGlobalStaticStrByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<&'static str, KaytonError>;

/// Set/overwrite a named GlobalStrBuf global, return handle.
pub type SetGlobalStrBufFn =
    fn(ctx: &mut KaytonContext, name: &str, value: GlobalStrBuf) -> Result<HKayGlobal, KaytonError>;

/// Read an existing GlobalStrBuf global by name.
pub type GetGlobalStrBufFn =
    fn(ctx: &mut KaytonContext, name: &str) -> Result<GlobalStrBuf, KaytonError>;

/// Fast path: read a GlobalStrBuf global by handle.
pub type GetGlobalStrBufByHandleFn =
    fn(ctx: &mut KaytonContext, h: HKayGlobal) -> Result<GlobalStrBuf, KaytonError>;
