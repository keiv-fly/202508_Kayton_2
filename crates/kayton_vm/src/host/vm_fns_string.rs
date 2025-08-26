use kayton_api::types::{GlobalStrBuf, HKayGlobal, KaytonError};

use crate::kinds::{pack_handle, unpack_handle, KIND_STATICSTR, KIND_STRBUF};
use super::HostState;

impl HostState {
    // Built-in setters/getters for strings
    pub fn set_static_str(&mut self, name: &str, value: &'static str) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_STATICSTR {
                self.static_strs[idx as usize] = value;
                return h;
            }
        }
        let idx = self.static_strs.len() as u32;
        self.static_strs.push(value);
        let h = pack_handle(KIND_STATICSTR, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_static_str(&self, name: &str) -> Result<&'static str, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_STATICSTR {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.static_strs
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_static_str_by_handle(&self, h: HKayGlobal) -> Result<&'static str, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_STATICSTR {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.static_strs
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_str_buf(&mut self, name: &str, value: GlobalStrBuf) -> HKayGlobal {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_STRBUF {
                // Overwrite in place; let previous value drop after move
                if let Some(slot) = self.str_bufs.get_mut(idx as usize) {
                    *slot = value;
                }
                return h;
            }
        }
        let idx = self.str_bufs.len() as u32;
        self.str_bufs.push(value);
        let h = pack_handle(KIND_STRBUF, idx);
        self.bind_name(name, h);
        h
    }
    
    pub fn get_str_buf(&self, name: &str) -> Result<GlobalStrBuf, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (k, idx) = unpack_handle(h);
        if k != KIND_STRBUF {
            return Err(KaytonError::generic("wrong kind"));
        }
        if let Some(sb) = self.str_bufs.get(idx as usize) {
            Ok(GlobalStrBuf::from_raw(sb.ptr, sb.len, sb.capacity))
        } else {
            Err(KaytonError::generic("index out of range"))
        }
    }

    pub fn get_str_buf_by_handle(&self, h: HKayGlobal) -> Result<GlobalStrBuf, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_STRBUF {
            return Err(KaytonError::generic("wrong kind"));
        }
        if let Some(sb) = self.str_bufs.get(idx as usize) {
            Ok(GlobalStrBuf::from_raw(sb.ptr, sb.len, sb.capacity))
        } else {
            Err(KaytonError::generic("index out of range"))
        }
    }
}
