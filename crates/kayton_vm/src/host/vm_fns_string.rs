use kayton_api::KVec;
use kayton_api::types::{GlobalStrBuf, HKayRef, KaytonError};

use super::HostState;
use crate::kinds::{pack_handle, unpack_handle};
use kayton_api::kinds::{KIND_KVEC, KIND_STATICSTR, KIND_STRBUF};

impl HostState {
    // Built-in setters/getters for strings
    pub fn set_static_str(&mut self, name: &str, value: &'static str) -> HKayRef {
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

    pub fn get_static_str_by_handle(&self, h: HKayRef) -> Result<&'static str, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_STATICSTR {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.static_strs
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn set_str_buf(&mut self, name: &str, value: GlobalStrBuf) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_STRBUF {
                // Overwrite in place with Option; drop previous value if present
                if let Some(slot) = self.str_bufs.get_mut(idx as usize) {
                    if let Some(old) = slot.take() {
                        drop(old);
                    }
                    *slot = Some(value);
                }
                return h;
            }
        }
        let idx = self.str_bufs.len() as u32;
        self.str_bufs.push(Some(value));
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
        if let Some(Some(sb)) = self.str_bufs.get(idx as usize) {
            Ok(GlobalStrBuf::from_raw(sb.ptr, sb.len, sb.capacity))
        } else {
            Err(KaytonError::generic("index out of range"))
        }
    }

    pub fn get_str_buf_by_handle(&self, h: HKayRef) -> Result<GlobalStrBuf, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_STRBUF {
            return Err(KaytonError::generic("wrong kind"));
        }
        if let Some(Some(sb)) = self.str_bufs.get(idx as usize) {
            Ok(GlobalStrBuf::from_raw(sb.ptr, sb.len, sb.capacity))
        } else {
            Err(KaytonError::generic("index out of range"))
        }
    }

    pub fn drop_str_buf_by_handle(&mut self, h: HKayRef) -> Result<(), KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_STRBUF {
            return Err(KaytonError::generic("wrong kind"));
        }
        let i = idx as usize;
        if let Some(slot) = self.str_bufs.get_mut(i) {
            if let Some(old) = slot.take() {
                drop(old);
            }
            Ok(())
        } else {
            Err(KaytonError::generic("index out of range"))
        }
    }
}

impl HostState {
    // KVec APIs
    pub fn set_kvec(&mut self, name: &str, value: KVec) -> HKayRef {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == KIND_KVEC {
                if let Some(slot) = self.kvecs.get_mut(idx as usize) {
                    // KVec is a by-value descriptor; just overwrite
                    *slot = Some(value);
                }
                return h;
            }
        }
        let idx = self.kvecs.len() as u32;
        self.kvecs.push(Some(value));
        let h = pack_handle(KIND_KVEC, idx);
        self.bind_name(name, h);
        h
    }

    pub fn get_kvec(&self, name: &str) -> Result<KVec, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        self.get_kvec_by_handle(h)
    }

    pub fn get_kvec_by_handle(&self, h: HKayRef) -> Result<KVec, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_KVEC {
            return Err(KaytonError::generic("wrong kind"));
        }
        if let Some(Some(v)) = self.kvecs.get(idx as usize) {
            // Return a copy of the descriptor without drop_fn to avoid double-free
            Ok(KVec::from_raw(v.ptr, v.len, v.capacity, v.kind))
        } else {
            Err(KaytonError::generic("index out of range"))
        }
    }

    pub fn drop_kvec_by_handle(&mut self, h: HKayRef) -> Result<(), KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_KVEC {
            return Err(KaytonError::generic("wrong kind"));
        }
        let i = idx as usize;
        if let Some(slot) = self.kvecs.get_mut(i) {
            // KVec is just a descriptor; dropping means removing the entry.
            // The actual memory behind ptr is owned elsewhere or by the user.
            *slot = None;
            Ok(())
        } else {
            Err(KaytonError::generic("index out of range"))
        }
    }
}
