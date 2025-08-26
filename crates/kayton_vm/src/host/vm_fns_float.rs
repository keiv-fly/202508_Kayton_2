use kayton_api::types::{HKayRef, KaytonError};

use super::HostState;
use crate::kinds::{KIND_F32, KIND_F64, pack_handle, unpack_handle};

impl HostState {
    // Built-in setters/getters for floating point numbers
    pub fn set_f64(&mut self, name: &str, value: f64) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_F64 {
                self.f64s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.f64s.len() as u32;
        self.f64s.push(value);
        let h = pack_handle(KIND_F64, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_f64(&self, name: &str) -> Result<f64, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_F64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.f64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_f64_by_handle(&self, h: HKayRef) -> Result<f64, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_F64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.f64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_f32(&mut self, name: &str, value: f32) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_F32 {
                self.f32s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.f32s.len() as u32;
        self.f32s.push(value);
        let h = pack_handle(KIND_F32, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_f32(&self, name: &str) -> Result<f32, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_F32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.f32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_f32_by_handle(&self, h: HKayRef) -> Result<f32, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_F32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.f32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
}
