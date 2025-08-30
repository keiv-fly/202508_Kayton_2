use std::ffi::c_void;
use std::vec::Vec;

use kayton_api::fns_dynamic::DynDropFn;

// ---------------- Dynamic kind store ----------------

pub struct DynKindStore {
    _name: &'static str,
    elems: Vec<Option<*mut c_void>>,
    drop_fn: DynDropFn,
}

impl DynKindStore {
    pub fn new(_name: &'static str, drop_fn: DynDropFn) -> Self {
        Self {
            _name,
            elems: Vec::new(),
            drop_fn,
        }
    }

    pub fn push(&mut self, ptr: *mut c_void) -> u32 {
        let idx = self.elems.len() as u32;
        self.elems.push(Some(ptr));
        idx
    }

    pub fn set(&mut self, idx: u32, ptr: Option<*mut c_void>) {
        let i = idx as usize;
        if let Some(slot) = self.elems.get_mut(i) {
            if let Some(to_drop) = slot.take() {
                if !to_drop.is_null() {
                    unsafe { (self.drop_fn)(to_drop) }
                }
            }
            *slot = ptr;
        }
    }

    pub fn get(&self, idx: u32) -> Option<*mut c_void> {
        self.elems.get(idx as usize).and_then(|p| *p)
    }

    pub fn drop_all(&mut self) {
        for &opt_p in &self.elems {
            if let Some(p) = opt_p {
                if !p.is_null() {
                    unsafe { (self.drop_fn)(p) }
                }
            }
        }
        self.elems.clear();
    }
}
