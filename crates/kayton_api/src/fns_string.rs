use crate::{HKayGlobal, KaytonContext, KaytonError, types::GlobalStrBuf};

/// Set/overwrite a named static string global, return handle.
pub type SetGlobalStaticStrFn =
    fn(ctx: &mut KaytonContext, name: &str, value: &'static str) -> Result<HKayGlobal, KaytonError>;

/// Read an existing static string global by name.
pub type GetGlobalStaticStrFn =
    fn(ctx: &mut KaytonContext, name: &str) -> Result<&'static str, KaytonError>;

/// Set/overwrite a named GlobalStrBuf global, return handle.
pub type SetGlobalStrBufFn =
    fn(ctx: &mut KaytonContext, name: &str, value: GlobalStrBuf) -> Result<HKayGlobal, KaytonError>;

/// Read an existing GlobalStrBuf global by name.
pub type GetGlobalStrBufFn =
    fn(ctx: &mut KaytonContext, name: &str) -> Result<GlobalStrBuf, KaytonError>;
