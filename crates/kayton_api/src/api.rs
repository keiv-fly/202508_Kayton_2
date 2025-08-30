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
    pub drop_global_str_buf: crate::fns_string::DropGlobalStrBufFn,

    pub register_dynamic_kind: crate::fns_dynamic::RegisterDynamicKindFn,
    pub set_global_dyn_ptr: crate::fns_dynamic::SetGlobalDynPtrFn,
    pub get_global_dyn_ptr: crate::fns_dynamic::GetGlobalDynPtrFn,
    pub get_global_dyn_ptr_by_handle: crate::fns_dynamic::GetGlobalDynPtrByHandleFn,
    pub drop_global_dyn_ptr: crate::fns_dynamic::DropGlobalDynPtrFn,

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

    // ---- Optional interners ----
    pub intern_u64: crate::fns_intern::InternU64Fn,
    pub intern_u8: crate::fns_intern::InternU8Fn,
    pub intern_f64: crate::fns_intern::InternF64Fn,
    pub intern_f32: crate::fns_intern::InternF32Fn,
    pub intern_static_str: crate::fns_intern::InternStaticStrFn,
    pub intern_str_buf: crate::fns_intern::InternStrBufFn,
    pub intern_dyn_ptr: crate::fns_intern::InternDynPtrFn,

    // ---- Tuples ----
    pub set_global_tuple_from_handles: crate::fns_tuple::SetGlobalTupleFromHandlesFn,
    pub get_global_tuple_len: crate::fns_tuple::GetGlobalTupleLenFn,
    pub get_tuple_len_by_handle: crate::fns_tuple::GetTupleLenByHandleFn,
    pub get_global_tuple_item: crate::fns_tuple::GetGlobalTupleItemFn,
    pub get_global_tuple_item_by_handle: crate::fns_tuple::GetGlobalTupleItemByHandleFn,
    pub read_tuple_into_slice_by_handle: crate::fns_tuple::ReadTupleIntoSliceByHandleFn,
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
        h: crate::types::HKayRef,
    ) -> Result<u64, crate::types::KaytonError> {
        (self.api().get_global_u64_by_handle)(self, h)
    }
    #[inline]
    pub fn get_u8_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<u8, crate::types::KaytonError> {
        (self.api().get_global_u8_by_handle)(self, h)
    }
    #[inline]
    pub fn get_f64_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<f64, crate::types::KaytonError> {
        (self.api().get_global_f64_by_handle)(self, h)
    }
    #[inline]
    pub fn get_f32_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<f32, crate::types::KaytonError> {
        (self.api().get_global_f32_by_handle)(self, h)
    }
    #[inline]
    pub fn get_static_str_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<&'static str, crate::types::KaytonError> {
        (self.api().get_global_static_str_by_handle)(self, h)
    }
    #[inline]
    pub fn get_str_buf_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<crate::types::GlobalStrBuf, crate::types::KaytonError> {
        (self.api().get_global_str_buf_by_handle)(self, h)
    }

    #[inline]
    pub fn get_u32_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<u32, crate::types::KaytonError> {
        (self.api().get_global_u32_by_handle)(self, h)
    }
    #[inline]
    pub fn get_u16_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<u16, crate::types::KaytonError> {
        (self.api().get_global_u16_by_handle)(self, h)
    }
    #[inline]
    pub fn get_u128_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<u128, crate::types::KaytonError> {
        (self.api().get_global_u128_by_handle)(self, h)
    }
    #[inline]
    pub fn get_usize_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<usize, crate::types::KaytonError> {
        (self.api().get_global_usize_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i8_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<i8, crate::types::KaytonError> {
        (self.api().get_global_i8_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i16_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<i16, crate::types::KaytonError> {
        (self.api().get_global_i16_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i32_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<i32, crate::types::KaytonError> {
        (self.api().get_global_i32_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i64_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<i64, crate::types::KaytonError> {
        (self.api().get_global_i64_by_handle)(self, h)
    }
    #[inline]
    pub fn get_i128_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<i128, crate::types::KaytonError> {
        (self.api().get_global_i128_by_handle)(self, h)
    }
    #[inline]
    pub fn get_isize_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<isize, crate::types::KaytonError> {
        (self.api().get_global_isize_by_handle)(self, h)
    }
    #[inline]
    pub fn get_bool_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<bool, crate::types::KaytonError> {
        (self.api().get_global_bool_by_handle)(self, h)
    }

    // ---- Tuple convenience ----
    #[inline]
    pub fn tuple_len_by_handle(
        &mut self,
        h: crate::types::HKayRef,
    ) -> Result<usize, crate::types::KaytonError> {
        (self.api().get_tuple_len_by_handle)(self, h)
    }
    #[inline]
    pub fn tuple_item_by_handle(
        &mut self,
        h: crate::types::HKayRef,
        index: usize,
    ) -> Result<crate::types::HKayRef, crate::types::KaytonError> {
        (self.api().get_global_tuple_item_by_handle)(self, h, index)
    }
}
