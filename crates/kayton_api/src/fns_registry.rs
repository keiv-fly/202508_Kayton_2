use crate::types::{KaytonContext, KaytonError, RawFnPtr, TypeMeta};

/// Register a raw Rust-ABI function pointer under a stable name with an optional signature id.
pub type RegisterFunctionFn = fn(
    ctx: &mut KaytonContext,
    name: &str,
    raw_ptr: RawFnPtr,
    sig_id: u64,
) -> Result<(), KaytonError>;

/// Get a previously registered raw function pointer by stable name.
pub type GetFunctionFn = fn(ctx: &mut KaytonContext, name: &str) -> Result<RawFnPtr, KaytonError>;

/// Register a type metadata under a stable name.
pub type RegisterTypeFn =
    fn(ctx: &mut KaytonContext, name: &str, meta: TypeMeta) -> Result<(), KaytonError>;

/// Get a previously registered type metadata by stable name.
pub type GetTypeFn = fn(ctx: &mut KaytonContext, name: &str) -> Result<TypeMeta, KaytonError>;

// ---- Signature helpers (compact validation primitives) ----

/// A simple rolling-64 signature helper for Rust type/ABI descriptors.
#[inline]
pub const fn sig64_mix(prev: u64, x: u64) -> u64 {
    let a = prev ^ x.wrapping_mul(0x9E3779B185EBCA87);
    a.rotate_left(27).wrapping_mul(5).wrapping_add(0x52DCE729)
}

#[inline]
pub const fn sig64_finish(x: u64) -> u64 {
    let mut v = x ^ (x >> 33);
    v = v.wrapping_mul(0xff51afd7ed558ccd);
    v ^= v >> 33;
    v = v.wrapping_mul(0xc4ceb9fe1a85ec53);
    v ^ (v >> 33)
}
