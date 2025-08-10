use core::ffi::c_void;

/// Single flat vtable (HPy-style). Append new fields at the end only!
#[repr(C)]
pub struct KaytonApi {
    /// sizeof(KaytonApi). Plugins can feature-detect by comparing this value.
    pub size: u64,

    // === Integer block: u64 + u8 (append-only; keep order stable) ===
    pub set_global_u64: crate::fns_int::SetGlobalU64Fn,
    pub get_global_u64: crate::fns_int::GetGlobalU64Fn,

    pub set_global_u8: crate::fns_int::SetGlobalU8Fn,
    pub get_global_u8: crate::fns_int::GetGlobalU8Fn,

    // === Float block: f64 + f32 (append-only; keep order stable) ===
    pub set_global_f64: crate::fns_float::SetGlobalF64Fn,
    pub get_global_f64: crate::fns_float::GetGlobalF64Fn,

    pub set_global_f32: crate::fns_float::SetGlobalF32Fn,
    pub get_global_f32: crate::fns_float::GetGlobalF32Fn,

    // --- Reserved for future expansion (keep at end; append more as needed) ---
    pub _reserved0: *const c_void,
    pub _reserved1: *const c_void,
    pub _reserved2: *const c_void,
    pub _reserved3: *const c_void,
}

impl crate::types::KaytonContext {
    /// Convenience accessor for plugins implemented in Rust.
    #[inline]
    pub fn api(&self) -> &KaytonApi {
        // Safety: host must supply a valid pointer to a KaytonApi of at least `size` bytes.
        unsafe { &*self.api }
    }
}
