/// Single flat vtable (HPy-style).
#[repr(C)]
pub struct KaytonApi {
    /// sizeof(KaytonApi). Plugins can feature-detect by comparing this value.
    pub size: u64,

    pub set_global_u64: crate::fns_uint::SetGlobalU64Fn,
    pub get_global_u64: crate::fns_uint::GetGlobalU64Fn,
    pub get_global_u64_by_handle: crate::fns_uint::GetGlobalU64ByHandleFn,

    pub set_global_u8: crate::fns_uint::SetGlobalU8Fn,
    pub get_global_u8: crate::fns_uint::GetGlobalU8Fn,
    pub get_global_u8_by_handle: crate::fns_uint::GetGlobalU8ByHandleFn,

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

    // ---- Newly added integer and bool types ----
    pub set_global_u32: crate::fns_uint::SetGlobalU32Fn,
    pub get_global_u32: crate::fns_uint::GetGlobalU32Fn,
    pub get_global_u32_by_handle: crate::fns_uint::GetGlobalU32ByHandleFn,

    pub set_global_u16: crate::fns_uint::SetGlobalU16Fn,
    pub get_global_u16: crate::fns_uint::GetGlobalU16Fn,
    pub get_global_u16_by_handle: crate::fns_uint::GetGlobalU16ByHandleFn,

    pub set_global_u128: crate::fns_uint::SetGlobalU128Fn,
    pub get_global_u128: crate::fns_uint::GetGlobalU128Fn,
    pub get_global_u128_by_handle: crate::fns_uint::GetGlobalU128ByHandleFn,

    pub set_global_usize: crate::fns_uint::SetGlobalUsizeFn,
    pub get_global_usize: crate::fns_uint::GetGlobalUsizeFn,
    pub get_global_usize_by_handle: crate::fns_uint::GetGlobalUsizeByHandleFn,

    pub set_global_i8: crate::fns_sint::SetGlobalI8Fn,
    pub get_global_i8: crate::fns_sint::GetGlobalI8Fn,
    pub get_global_i8_by_handle: crate::fns_sint::GetGlobalI8ByHandleFn,

    pub set_global_i16: crate::fns_sint::SetGlobalI16Fn,
    pub get_global_i16: crate::fns_sint::GetGlobalI16Fn,
    pub get_global_i16_by_handle: crate::fns_sint::GetGlobalI16ByHandleFn,

    pub set_global_i32: crate::fns_sint::SetGlobalI32Fn,
    pub get_global_i32: crate::fns_sint::GetGlobalI32Fn,
    pub get_global_i32_by_handle: crate::fns_sint::GetGlobalI32ByHandleFn,

    pub set_global_i64: crate::fns_sint::SetGlobalI64Fn,
    pub get_global_i64: crate::fns_sint::GetGlobalI64Fn,
    pub get_global_i64_by_handle: crate::fns_sint::GetGlobalI64ByHandleFn,

    pub set_global_i128: crate::fns_sint::SetGlobalI128Fn,
    pub get_global_i128: crate::fns_sint::GetGlobalI128Fn,
    pub get_global_i128_by_handle: crate::fns_sint::GetGlobalI128ByHandleFn,

    pub set_global_isize: crate::fns_sint::SetGlobalIsizeFn,
    pub get_global_isize: crate::fns_sint::GetGlobalIsizeFn,
    pub get_global_isize_by_handle: crate::fns_sint::GetGlobalIsizeByHandleFn,

    pub set_global_bool: crate::fns_sint::SetGlobalBoolFn,
    pub get_global_bool: crate::fns_sint::GetGlobalBoolFn,
    pub get_global_bool_by_handle: crate::fns_sint::GetGlobalBoolByHandleFn,
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

    #[inline]
    pub fn get_u32_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<u32, crate::types::KaytonError> {
        (self.api().get_global_u32_by_handle)(self, h)
    }
    #[inline]
    pub fn get_u16_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<u16, crate::types::KaytonError> {
        (self.api().get_global_u16_by_handle)(self, h)
    }
    #[inline]
    pub fn get_u128_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<u128, crate::types::KaytonError> {
        (self.api().get_global_u128_by_handle)(self, h)
    }
    #[inline]
    pub fn get_usize_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<usize, crate::types::KaytonError> {
        (self.api().get_global_usize_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i8_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<i8, crate::types::KaytonError> {
        (self.api().get_global_i8_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i16_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<i16, crate::types::KaytonError> {
        (self.api().get_global_i16_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i32_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<i32, crate::types::KaytonError> {
        (self.api().get_global_i32_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i64_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<i64, crate::types::KaytonError> {
        (self.api().get_global_i64_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i128_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<i128, crate::types::KaytonError> {
        (self.api().get_global_i128_by_handle)(self, h)
    }
    #[inline]
    pub fn get_isize_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<isize, crate::types::KaytonError> {
        (self.api().get_global_isize_by_handle)(self, h)
    }
    #[inline]
    pub fn get_bool_by_handle(
        &mut self,
        h: crate::types::HKayGlobal,
    ) -> Result<bool, crate::types::KaytonError> {
        (self.api().get_global_bool_by_handle)(self, h)
    }
}
