use kayton_api::types::{HKayGlobal, KaytonError};

use crate::kinds::{pack_handle, unpack_handle, KIND_I8, KIND_I16, KIND_I32, KIND_I64, KIND_I128, KIND_ISIZE, KIND_BOOL};
use super::HostState;

impl HostState {
    // Built-in setters/getters for signed integers and bool
    pub fn set_i8(&mut self, name: &str, value: i8) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_I8 {
                self.i8s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.i8s.len() as u32;
        self.i8s.push(value);
        let h = pack_handle(KIND_I8, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_i8(&self, name: &str) -> Result<i8, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_I8 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i8s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
    
    pub fn get_i8_by_handle(&self, h: HKayGlobal) -> Result<i8, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I8 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i8s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_i16(&mut self, name: &str, value: i16) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_I16 {
                self.i16s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.i16s.len() as u32;
        self.i16s.push(value);
        let h = pack_handle(KIND_I16, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_i16(&self, name: &str) -> Result<i16, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_I16 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i16s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
    
    pub fn get_i16_by_handle(&self, h: HKayGlobal) -> Result<i16, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I16 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i16s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_i32(&mut self, name: &str, value: i32) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_I32 {
                self.i32s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.i32s.len() as u32;
        self.i32s.push(value);
        let h = pack_handle(KIND_I32, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_i32(&self, name: &str) -> Result<i32, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_I32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
    
    pub fn get_i32_by_handle(&self, h: HKayGlobal) -> Result<i32, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_i64(&mut self, name: &str, value: i64) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_I64 {
                self.i64s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.i64s.len() as u32;
        self.i64s.push(value);
        let h = pack_handle(KIND_I64, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_i64(&self, name: &str) -> Result<i64, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_I64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
    
    pub fn get_i64_by_handle(&self, h: HKayGlobal) -> Result<i64, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_i128(&mut self, name: &str, value: i128) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_I128 {
                self.i128s[idx as usize] = value;
                return h;
            }
        }
        let idx = self.i128s.len() as u32;
        self.i128s.push(value);
        let h = pack_handle(KIND_I128, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_i128(&self, name: &str) -> Result<i128, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_I128 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i128s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
    
    pub fn get_i128_by_handle(&self, h: HKayGlobal) -> Result<i128, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I128 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i128s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_isize(&mut self, name: &str, value: isize) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_ISIZE {
                self.isizes[idx as usize] = value;
                return h;
            }
        }
        let idx = self.isizes.len() as u32;
        self.isizes.push(value);
        let h = pack_handle(KIND_ISIZE, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_isize(&self, name: &str) -> Result<isize, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_ISIZE {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.isizes
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
    
    pub fn get_isize_by_handle(&self, h: HKayGlobal) -> Result<isize, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_ISIZE {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.isizes
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_bool(&mut self, name: &str, value: bool) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_BOOL {
                self.bools[idx as usize] = value;
                return h;
            }
        }
        let idx = self.bools.len() as u32;
        self.bools.push(value);
        let h = pack_handle(KIND_BOOL, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_bool(&self, name: &str) -> Result<bool, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_BOOL {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.bools
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
    
    pub fn get_bool_by_handle(&self, h: HKayGlobal) -> Result<bool, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_BOOL {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.bools
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
}
