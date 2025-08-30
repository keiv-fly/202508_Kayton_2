use core::ffi::c_void;

use kayton_api::kinds::KindId;
use kayton_api::types::{HKayRef, KaytonError};

use super::HostState;
use crate::kinds::pack_handle;
use kayton_api::kinds::{KIND_F32, KIND_F64, KIND_STATICSTR, KIND_STRBUF, KIND_U8, KIND_U64};

impl HostState {
    pub fn intern_u64(&mut self, value: u64) -> Result<HKayRef, KaytonError> {
        let idx = self.u64s.len() as u32;
        self.u64s.push(value);
        Ok(pack_handle(KIND_U64, idx))
    }

    pub fn intern_u8(&mut self, value: u8) -> Result<HKayRef, KaytonError> {
        let idx = self.u8s.len() as u32;
        self.u8s.push(value);
        Ok(pack_handle(KIND_U8, idx))
    }

    pub fn intern_f64(&mut self, value: f64) -> Result<HKayRef, KaytonError> {
        let idx = self.f64s.len() as u32;
        self.f64s.push(value);
        Ok(pack_handle(KIND_F64, idx))
    }

    pub fn intern_f32(&mut self, value: f32) -> Result<HKayRef, KaytonError> {
        let idx = self.f32s.len() as u32;
        self.f32s.push(value);
        Ok(pack_handle(KIND_F32, idx))
    }

    pub fn intern_static_str(&mut self, s: &'static str) -> Result<HKayRef, KaytonError> {
        let idx = self.static_strs.len() as u32;
        self.static_strs.push(s);
        Ok(pack_handle(KIND_STATICSTR, idx))
    }

    pub fn intern_str_buf(&mut self, s: &str) -> Result<HKayRef, KaytonError> {
        let buf = kayton_api::types::GlobalStrBuf::new(s.to_string());
        let idx = self.str_bufs.len() as u32;
        self.str_bufs.push(buf);
        Ok(pack_handle(KIND_STRBUF, idx))
    }

    pub fn intern_dyn_ptr(
        &mut self,
        dyn_kind: KindId,
        ptr: *mut c_void,
    ) -> Result<HKayRef, KaytonError> {
        let store = self
            .dyn_kinds
            .get_mut(&dyn_kind)
            .ok_or_else(|| KaytonError::generic("unknown dynamic kind"))?;
        let idx = store.push(ptr);
        Ok(pack_handle(dyn_kind, idx))
    }
}
