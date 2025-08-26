use core::ffi::c_void;

use crate::{HKayRef, KaytonContext, KaytonError};

/// Intern unnamed u64 value; returns a handle valid for VM lifetime
pub type InternU64Fn = fn(ctx: &mut KaytonContext, value: u64) -> Result<HKayRef, KaytonError>;

/// Intern unnamed u8 value; returns a handle
pub type InternU8Fn = fn(ctx: &mut KaytonContext, value: u8) -> Result<HKayRef, KaytonError>;

/// Intern unnamed f64 value; returns a handle
pub type InternF64Fn = fn(ctx: &mut KaytonContext, value: f64) -> Result<HKayRef, KaytonError>;

/// Intern unnamed f32 value; returns a handle
pub type InternF32Fn = fn(ctx: &mut KaytonContext, value: f32) -> Result<HKayRef, KaytonError>;

/// Intern a static string; returns a handle
pub type InternStaticStrFn =
    fn(ctx: &mut KaytonContext, s: &'static str) -> Result<HKayRef, KaytonError>;

/// Intern a string buffer from &str; returns a handle
pub type InternStrBufFn = fn(ctx: &mut KaytonContext, s: &str) -> Result<HKayRef, KaytonError>;

/// Intern a dynamic pointer of a previously registered kind; returns a handle
pub type InternDynPtrFn =
    fn(ctx: &mut KaytonContext, dyn_kind: u32, ptr: *mut c_void) -> Result<HKayRef, KaytonError>;
