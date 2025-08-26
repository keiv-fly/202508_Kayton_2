use std::ffi::c_void;

use kayton_api::fns_dynamic::{DynDropFn, KindId};
use kayton_api::types::{HKayRef, KaytonError};

use super::{DynKindStore, HostState};
use crate::kinds::{pack_handle, unpack_handle};

impl HostState {
    // ---- dynamic kind management ----
    pub fn register_dynamic_kind(&mut self, name: &'static str, drop_fn: DynDropFn) -> KindId {
        let id = self.next_kind_id;
        self.next_kind_id = self.next_kind_id.checked_add(1).unwrap();
        self.dyn_kinds.insert(id, DynKindStore::new(name, drop_fn));
        id
    }

    pub fn set_dyn_by_name(
        &mut self,
        kind: KindId,
        name: &str,
        ptr: *mut c_void,
    ) -> Result<HKayRef, KaytonError> {
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

    pub fn get_dyn_by_name(&self, name: &str) -> Result<(*mut c_void, KindId), KaytonError> {
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

    pub fn get_dyn_by_handle(&self, h: HKayRef) -> Result<*mut c_void, KaytonError> {
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
