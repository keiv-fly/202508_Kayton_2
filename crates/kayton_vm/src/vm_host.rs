use std::boxed::Box;
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::string::String;
use std::vec::Vec;

use kayton_api::api::KaytonApi;
use kayton_api::fns_dynamic::{DynDropFn, KindId};
use kayton_api::types::{GlobalStrBuf, HKayGlobal, KaytonContext, KaytonError};

// ---------------- Kind IDs and handle packing ----------------

pub const KIND_U64: KindId = 1;
pub const KIND_U8: KindId = 2;
pub const KIND_F64: KindId = 3;
pub const KIND_F32: KindId = 4;
pub const KIND_STATICSTR: KindId = 5;
pub const KIND_STRBUF: KindId = 6;
pub const KIND_U32: KindId = 7;
pub const KIND_U16: KindId = 8;
pub const KIND_U128: KindId = 9;
pub const KIND_USIZE: KindId = 10;
pub const KIND_I8: KindId = 11;
pub const KIND_I16: KindId = 12;
pub const KIND_I32: KindId = 13;
pub const KIND_I64: KindId = 14;
pub const KIND_I128: KindId = 15;
pub const KIND_ISIZE: KindId = 16;
pub const KIND_BOOL: KindId = 17;

const KIND_SHIFT: u64 = 32;
const IDX_MASK: u64 = (1u64 << 32) - 1;

#[inline]
fn pack_handle(kind: KindId, idx: u32) -> HKayGlobal {
    HKayGlobal(((kind as u64) << KIND_SHIFT) | (idx as u64 & IDX_MASK))
}

#[inline]
fn unpack_handle(h: HKayGlobal) -> (KindId, u32) {
    let raw = h.0;
    (((raw >> KIND_SHIFT) as u32), (raw & IDX_MASK) as u32)
}

// ---------------- Dynamic kind store ----------------

pub struct DynKindStore {
    _name: &'static str,
    elems: Vec<*mut c_void>,
    drop_fn: DynDropFn,
}

impl DynKindStore {
    fn new(_name: &'static str, drop_fn: DynDropFn) -> Self {
        Self {
            _name,
            elems: Vec::new(),
            drop_fn,
        }
    }
    fn push(&mut self, ptr: *mut c_void) -> u32 {
        let idx = self.elems.len() as u32;
        self.elems.push(ptr);
        idx
    }
    fn set(&mut self, idx: u32, ptr: *mut c_void) {
        let i = idx as usize;
        if let Some(old) = self.elems.get_mut(i) {
            if let Some(to_drop) = (!old.is_null()).then_some(*old) {
                unsafe { (self.drop_fn)(to_drop) }
            }
            *old = ptr;
        }
    }
    fn get(&self, idx: u32) -> Option<*mut c_void> {
        self.elems.get(idx as usize).copied()
    }
    fn drop_all(&mut self) {
        for &p in &self.elems {
            if !p.is_null() {
                unsafe { (self.drop_fn)(p) }
            }
        }
        self.elems.clear();
    }
}

// ---------------- Host state ----------------

pub struct HostState {
    name_to_handle: BTreeMap<String, HKayGlobal>,
    handle_to_name: BTreeMap<u64, String>,

    u64s: Vec<u64>,
    u8s: Vec<u8>,
    u32s: Vec<u32>,
    u16s: Vec<u16>,
    u128s: Vec<u128>,
    usizes: Vec<usize>,
    i8s: Vec<i8>,
    i16s: Vec<i16>,
    i32s: Vec<i32>,
    i64s: Vec<i64>,
    i128s: Vec<i128>,
    isizes: Vec<isize>,
    bools: Vec<bool>,
    f64s: Vec<f64>,
    f32s: Vec<f32>,
    static_strs: Vec<&'static str>,
    str_bufs: Vec<GlobalStrBuf>,

    next_kind_id: KindId,
    dyn_kinds: BTreeMap<KindId, DynKindStore>,
}

impl HostState {
    fn new() -> Self {
        Self {
            name_to_handle: BTreeMap::new(),
            handle_to_name: BTreeMap::new(),
            u64s: Vec::new(),
            u8s: Vec::new(),
            u32s: Vec::new(),
            u16s: Vec::new(),
            u128s: Vec::new(),
            usizes: Vec::new(),
            i8s: Vec::new(),
            i16s: Vec::new(),
            i32s: Vec::new(),
            i64s: Vec::new(),
            i128s: Vec::new(),
            isizes: Vec::new(),
            bools: Vec::new(),
            f64s: Vec::new(),
            f32s: Vec::new(),
            static_strs: Vec::new(),
            str_bufs: Vec::new(),
            next_kind_id: 1000,
            dyn_kinds: BTreeMap::new(),
        }
    }

    #[inline]
    fn bind_name(&mut self, name: &str, h: HKayGlobal) {
        self.name_to_handle.insert(String::from(name), h);
        self.handle_to_name.insert(h.0, String::from(name));
    }
    #[inline]
    fn resolve(&self, name: &str) -> Option<HKayGlobal> {
        self.name_to_handle.get(name).copied()
    }

    // Built-in setters/getters
    fn set_u64(&mut self, name: &str, value: u64) -> HKayGlobal {
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
    fn get_u64(&self, name: &str) -> Result<u64, KaytonError> {
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

    fn get_u64_by_handle(&self, h: HKayGlobal) -> Result<u64, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_u8(&mut self, name: &str, value: u8) -> HKayGlobal {
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
    fn get_u8(&self, name: &str) -> Result<u8, KaytonError> {
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

    fn get_u8_by_handle(&self, h: HKayGlobal) -> Result<u8, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U8 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u8s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_u32(&mut self, name: &str, value: u32) -> HKayGlobal {
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
    fn get_u32(&self, name: &str) -> Result<u32, KaytonError> {
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
    fn get_u32_by_handle(&self, h: HKayGlobal) -> Result<u32, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_u16(&mut self, name: &str, value: u16) -> HKayGlobal {
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
    fn get_u16(&self, name: &str) -> Result<u16, KaytonError> {
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
    fn get_u16_by_handle(&self, h: HKayGlobal) -> Result<u16, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U16 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u16s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_u128(&mut self, name: &str, value: u128) -> HKayGlobal {
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
    fn get_u128(&self, name: &str) -> Result<u128, KaytonError> {
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
    fn get_u128_by_handle(&self, h: HKayGlobal) -> Result<u128, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_U128 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.u128s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_usize(&mut self, name: &str, value: usize) -> HKayGlobal {
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
    fn get_usize(&self, name: &str) -> Result<usize, KaytonError> {
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
    fn get_usize_by_handle(&self, h: HKayGlobal) -> Result<usize, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_USIZE {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.usizes
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_i8(&mut self, name: &str, value: i8) -> HKayGlobal {
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
    fn get_i8(&self, name: &str) -> Result<i8, KaytonError> {
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
    fn get_i8_by_handle(&self, h: HKayGlobal) -> Result<i8, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I8 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i8s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_i16(&mut self, name: &str, value: i16) -> HKayGlobal {
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
    fn get_i16(&self, name: &str) -> Result<i16, KaytonError> {
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
    fn get_i16_by_handle(&self, h: HKayGlobal) -> Result<i16, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I16 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i16s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_i32(&mut self, name: &str, value: i32) -> HKayGlobal {
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
    fn get_i32(&self, name: &str) -> Result<i32, KaytonError> {
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
    fn get_i32_by_handle(&self, h: HKayGlobal) -> Result<i32, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_i64(&mut self, name: &str, value: i64) -> HKayGlobal {
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
    fn get_i64(&self, name: &str) -> Result<i64, KaytonError> {
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
    fn get_i64_by_handle(&self, h: HKayGlobal) -> Result<i64, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_i128(&mut self, name: &str, value: i128) -> HKayGlobal {
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
    fn get_i128(&self, name: &str) -> Result<i128, KaytonError> {
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
    fn get_i128_by_handle(&self, h: HKayGlobal) -> Result<i128, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_I128 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.i128s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_isize(&mut self, name: &str, value: isize) -> HKayGlobal {
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
    fn get_isize(&self, name: &str) -> Result<isize, KaytonError> {
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
    fn get_isize_by_handle(&self, h: HKayGlobal) -> Result<isize, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_ISIZE {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.isizes
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_bool(&mut self, name: &str, value: bool) -> HKayGlobal {
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
    fn get_bool(&self, name: &str) -> Result<bool, KaytonError> {
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
    fn get_bool_by_handle(&self, h: HKayGlobal) -> Result<bool, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_BOOL {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.bools
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_f64(&mut self, name: &str, value: f64) -> HKayGlobal {
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
    fn get_f64(&self, name: &str) -> Result<f64, KaytonError> {
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

    fn get_f64_by_handle(&self, h: HKayGlobal) -> Result<f64, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_F64 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.f64s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_f32(&mut self, name: &str, value: f32) -> HKayGlobal {
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
    fn get_f32(&self, name: &str) -> Result<f32, KaytonError> {
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

    fn get_f32_by_handle(&self, h: HKayGlobal) -> Result<f32, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_F32 {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.f32s
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_static_str(&mut self, name: &str, value: &'static str) -> HKayGlobal {
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
    fn get_static_str(&self, name: &str) -> Result<&'static str, KaytonError> {
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

    fn get_static_str_by_handle(&self, h: HKayGlobal) -> Result<&'static str, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_STATICSTR {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.static_strs
            .get(idx as usize)
            .copied()
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    fn set_str_buf(&mut self, name: &str, value: GlobalStrBuf) -> HKayGlobal {
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
    fn get_str_buf(&self, name: &str) -> Result<GlobalStrBuf, KaytonError> {
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

    fn get_str_buf_by_handle(&self, h: HKayGlobal) -> Result<GlobalStrBuf, KaytonError> {
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

    // ---- dynamic kind management ----
    fn register_dynamic_kind(&mut self, name: &'static str, drop_fn: DynDropFn) -> KindId {
        let id = self.next_kind_id;
        self.next_kind_id = self.next_kind_id.checked_add(1).unwrap();
        self.dyn_kinds.insert(id, DynKindStore::new(name, drop_fn));
        id
    }

    fn set_dyn_by_name(
        &mut self,
        kind: KindId,
        name: &str,
        ptr: *mut c_void,
    ) -> Result<HKayGlobal, KaytonError> {
        if let Some(h) = self.resolve(name) {
            let (k, idx) = unpack_handle(h);
            if k == kind {
                if let Some(store) = self.dyn_kinds.get_mut(&kind) {
                    store.set(idx, ptr);
                    return Ok(h);
                }
                return Err(KaytonError::generic("unknown dynamic kind"));
            }
            let store = self
                .dyn_kinds
                .get_mut(&kind)
                .ok_or_else(|| KaytonError::generic("unknown dynamic kind"))?;
            let new_idx = store.push(ptr);
            let h2 = pack_handle(kind, new_idx);
            self.bind_name(name, h2);
            Ok(h2)
        } else {
            let store = self
                .dyn_kinds
                .get_mut(&kind)
                .ok_or_else(|| KaytonError::generic("unknown dynamic kind"))?;
            let idx = store.push(ptr);
            let h = pack_handle(kind, idx);
            self.bind_name(name, h);
            Ok(h)
        }
    }

    fn get_dyn_by_name(&self, name: &str) -> Result<(*mut c_void, KindId), KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        let (kind, idx) = unpack_handle(h);
        if let Some(store) = self.dyn_kinds.get(&kind) {
            let ptr = store
                .get(idx)
                .ok_or_else(|| KaytonError::generic("index out of range"))?;
            Ok((ptr, kind))
        } else {
            Err(KaytonError::generic("not a dynamic kind"))
        }
    }

    fn get_dyn_by_handle(&self, h: HKayGlobal) -> Result<*mut c_void, KaytonError> {
        let (kind, idx) = unpack_handle(h);
        let store = self
            .dyn_kinds
            .get(&kind)
            .ok_or_else(|| KaytonError::generic("not a dynamic kind"))?;
        store
            .get(idx)
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }
}

impl Drop for HostState {
    fn drop(&mut self) {
        for (_, store) in self.dyn_kinds.iter_mut() {
            store.drop_all();
        }
    }
}

// ---------------- Kayton VM wrapper ----------------

pub struct KaytonVm {
    host: Box<HostState>,
    api: Box<KaytonApi>,
}

impl KaytonVm {
    pub fn new() -> Self {
        let host = Box::new(HostState::new());

        // Instantiate API vtable
        let api = Box::new(KaytonApi {
            size: core::mem::size_of::<KaytonApi>() as u64,

            set_global_u64: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u64(name, v))
            },
            get_global_u64: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u64(name)
            },

            set_global_u8: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u8(name, v))
            },
            get_global_u8: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u8(name)
            },

            set_global_f64: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_f64(name, v))
            },
            get_global_f64: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f64(name)
            },

            set_global_f32: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_f32(name, v))
            },
            get_global_f32: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f32(name)
            },

            set_global_static_str: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_static_str(name, v))
            },
            get_global_static_str: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_static_str(name)
            },

            set_global_str_buf: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_str_buf(name, v))
            },
            get_global_str_buf: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                // Rebuild a by-value copy without drop_fn to avoid double-drop
                let sb = s.get_str_buf(name)?;
                Ok(GlobalStrBuf::from_raw(sb.ptr, sb.len, sb.capacity))
            },

            register_dynamic_kind: |ctx, name, drop_fn| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.register_dynamic_kind(name, drop_fn)
            },
            set_global_dyn_ptr: |ctx, kind, name, value| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.set_dyn_by_name(kind, name, value)
            },
            get_global_dyn_ptr: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_dyn_by_name(name)
            },
            get_global_dyn_ptr_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_dyn_by_handle(h)
            },

            get_global_u64_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u64_by_handle(h)
            },
            get_global_u8_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u8_by_handle(h)
            },

            get_global_f64_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f64_by_handle(h)
            },
            get_global_f32_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f32_by_handle(h)
            },

            get_global_static_str_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_static_str_by_handle(h)
            },
            get_global_str_buf_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_str_buf_by_handle(h)
            },

            // ---- New integer/bool functions ----
            set_global_u32: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u32(name, v))
            },
            get_global_u32: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u32(name)
            },
            get_global_u32_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u32_by_handle(h)
            },

            set_global_u16: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u16(name, v))
            },
            get_global_u16: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u16(name)
            },
            get_global_u16_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u16_by_handle(h)
            },

            set_global_u128: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u128(name, v))
            },
            get_global_u128: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u128(name)
            },
            get_global_u128_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u128_by_handle(h)
            },

            set_global_usize: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_usize(name, v))
            },
            get_global_usize: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_usize(name)
            },
            get_global_usize_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_usize_by_handle(h)
            },

            set_global_i8: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i8(name, v))
            },
            get_global_i8: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i8(name)
            },
            get_global_i8_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i8_by_handle(h)
            },

            set_global_i16: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i16(name, v))
            },
            get_global_i16: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i16(name)
            },
            get_global_i16_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i16_by_handle(h)
            },

            set_global_i32: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i32(name, v))
            },
            get_global_i32: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i32(name)
            },
            get_global_i32_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i32_by_handle(h)
            },

            set_global_i64: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i64(name, v))
            },
            get_global_i64: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i64(name)
            },
            get_global_i64_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i64_by_handle(h)
            },

            set_global_i128: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i128(name, v))
            },
            get_global_i128: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i128(name)
            },
            get_global_i128_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i128_by_handle(h)
            },

            set_global_isize: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_isize(name, v))
            },
            get_global_isize: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_isize(name)
            },
            get_global_isize_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_isize_by_handle(h)
            },

            set_global_bool: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_bool(name, v))
            },
            get_global_bool: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_bool(name)
            },
            get_global_bool_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_bool_by_handle(h)
            },
        });

        KaytonVm { host, api }
    }

    pub fn context(&mut self) -> KaytonContext {
        KaytonContext {
            abi_version: 1,
            host_data: &mut *self.host as *mut HostState as *mut c_void,
            api: &*self.api as *const KaytonApi,
        }
    }

    pub fn api(&self) -> &KaytonApi {
        &*self.api
    }
}
