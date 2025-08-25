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
    name: &'static str,
    elems: Vec<*mut c_void>,
    drop_fn: DynDropFn,
}

impl DynKindStore {
    fn new(name: &'static str, drop_fn: DynDropFn) -> Self {
        Self {
            name,
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

            _reserved0: core::ptr::null(),
            _reserved1: core::ptr::null(),
            _reserved2: core::ptr::null(),
            _reserved3: core::ptr::null(),
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
