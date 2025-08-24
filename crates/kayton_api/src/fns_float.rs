use crate::{HKayGlobal, KaytonContext, KaytonError};

/// Set/overwrite a named f64 global, return handle.
pub type SetGlobalF64Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: f64) -> Result<HKayGlobal, KaytonError>;

/// Read an existing f64 global by name.
pub type GetGlobalF64Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<f64, KaytonError>;

/// Set/overwrite a named f32 global, return handle.
pub type SetGlobalF32Fn =
    fn(ctx: &mut KaytonContext, name: &str, value: f32) -> Result<HKayGlobal, KaytonError>;

/// Read an existing f32 global by name.
pub type GetGlobalF32Fn = fn(ctx: &mut KaytonContext, name: &str) -> Result<f32, KaytonError>;
