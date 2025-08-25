use core::ffi::c_void;

/// Single flat vtable (HPy-style).
#[repr(C)]
pub struct KaytonApi {
    /// sizeof(KaytonApi). Plugins can feature-detect by comparing this value.
    pub size: u64,

    pub set_global_u64: crate::fns_int::SetGlobalU64Fn,
    pub get_global_u64: crate::fns_int::GetGlobalU64Fn,
    pub get_global_u64_by_handle: crate::fns_int::GetGlobalU64ByHandleFn,

    pub set_global_u8: crate::fns_int::SetGlobalU8Fn,
    pub get_global_u8: crate::fns_int::GetGlobalU8Fn,
    pub get_global_u8_by_handle: crate::fns_int::GetGlobalU8ByHandleFn,

    pub set_global_f64: crate::fns_float::SetGlobalF64Fn,
    pub get_global_f64: crate::fns_float::GetGlobalF64Fn,
    pub get_global_f64_by_handle: crate::fns_float::GetGlobalF64ByHandleFn,

    pub set_global_f32: crate::fns_float::SetGlobalF32Fn,
    pub get_global_f32: crate::fns_float::GetGlobalF32Fn,
    pub get_global_f32_by_handle: crate::fns_float::GetGlobalF32ByHandleFn,

    pub set_global_static_str: crate::fns_string::SetGlobalStaticStrFn,
    pub get_global_static_str: crate::fns_string::GetGlobalStaticStrFn,
    pub get_global_static_str_by_handle: crate::fns_string::GetGlobalStaticStrByHandleFn,

    pub set_global_str_buf: crate::fns_string::SetGlobalStrBufFn,
    pub get_global_str_buf: crate::fns_string::GetGlobalStrBufFn,
    pub get_global_str_buf_by_handle: crate::fns_string::GetGlobalStrBufByHandleFn,

    pub register_dynamic_kind: crate::fns_dynamic::RegisterDynamicKindFn,
    pub set_global_dyn_ptr: crate::fns_dynamic::SetGlobalDynPtrFn,
    pub get_global_dyn_ptr: crate::fns_dynamic::GetGlobalDynPtrFn,
    pub get_global_dyn_ptr_by_handle: crate::fns_dynamic::GetGlobalDynPtrByHandleFn,
}

impl crate::types::KaytonContext {
    /// Convenience accessor for plugins implemented in Rust.
    #[inline]
    pub fn api(&self) -> &KaytonApi {
        // Safety: host must supply a valid pointer to a KaytonApi of at least `size` bytes.
        unsafe { &*self.api }
    }

    // === Convenience wrappers for by-handle getters ===
    #[inline]
    pub fn get_u64_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<u64, crate::types::KaytonError> {
        (self.api().get_global_u64_by_handle)(self, h)
    }
    #[inline]
    pub fn get_u8_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<u8, crate::types::KaytonError> {
        (self.api().get_global_u8_by_handle)(self, h)
    }
    #[inline]
    pub fn get_f64_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<f64, crate::types::KaytonError> {
        (self.api().get_global_f64_by_handle)(self, h)
    }
    #[inline]
    pub fn get_f32_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<f32, crate::types::KaytonError> {
        (self.api().get_global_f32_by_handle)(self, h)
    }
    #[inline]
    pub fn get_static_str_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<&'static str, crate::types::KaytonError> {
        (self.api().get_global_static_str_by_handle)(self, h)
    }
    #[inline]
    pub fn get_str_buf_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<crate::types::GlobalStrBuf, crate::types::KaytonError> {
        (self.api().get_global_str_buf_by_handle)(self, h)
    }
}
