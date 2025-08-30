use kayton_api::types::{HKayRef, KaytonError};

use super::HostState;
use crate::kinds::{pack_handle, unpack_handle};
use kayton_api::kinds::{KIND_U8, KIND_U16, KIND_U32, KIND_U64, KIND_U128, KIND_USIZE};

impl HostState {
    // Built-in setters/getters for unsigned integers
    pub fn set_u64(&mut self, name: &str, value: u64) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_U64 {
                self.u64s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.u64s.len() as u32;
        self.u64s.push(value);
        let h = pack_handle(KIND_U64, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_u64(&self, name: &str) -> Result<u64, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_U64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_u64_by_handle(&self, h: HKayRef) -> Result<u64, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_u8(&mut self, name: &str, value: u8) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_U8 {
                self.u8s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.u8s.len() as u32;
        self.u8s.push(value);
        let h = pack_handle(KIND_U8, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_u8(&self, name: &str) -> Result<u8, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_U8 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u8s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_u8_by_handle(&self, h: HKayRef) -> Result<u8, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U8 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u8s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_u32(&mut self, name: &str, value: u32) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_U32 {
                self.u32s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.u32s.len() as u32;
        self.u32s.push(value);
        let h = pack_handle(KIND_U32, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_u32(&self, name: &str) -> Result<u32, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_U32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_u32_by_handle(&self, h: HKayRef) -> Result<u32, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_u16(&mut self, name: &str, value: u16) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_U16 {
                self.u16s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.u16s.len() as u32;
        self.u16s.push(value);
        let h = pack_handle(KIND_U16, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_u16(&self, name: &str) -> Result<u16, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_U16 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u16s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_u16_by_handle(&self, h: HKayRef) -> Result<u16, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U16 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u16s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_u128(&mut self, name: &str, value: u128) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_U128 {
                self.u128s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.u128s.len() as u32;
        self.u128s.push(value);
        let h = pack_handle(KIND_U128, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_u128(&self, name: &str) -> Result<u128, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_U128 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u128s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_u128_by_handle(&self, h: HKayRef) -> Result<u128, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U128 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u128s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_usize(&mut self, name: &str, value: usize) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_USIZE {
                self.usizes[idx as usize] = value;
                return h;
            }
        }
        let idx = self.usizes.len() as u32;
        self.usizes.push(value);
        let h = pack_handle(KIND_USIZE, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_usize(&self, name: &str) -> Result<usize, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_USIZE {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.usizes
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_usize_by_handle(&self, h: HKayRef) -> Result<usize, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_USIZE {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.usizes
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
}
