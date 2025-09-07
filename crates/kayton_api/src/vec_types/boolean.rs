extern crate alloc;
use alloc::vec::Vec;

use crate::kinds::KIND_BOOL;
use super::KVec;

impl KVec {
    pub fn from_vec_bool(vec: Vec<bool>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        // Note: Vec<bool> is bit-packed; len/cap are in elements, not bytes.
        // We store element counts in len/cap using size_of::<bool>() == 1 for round-tripping.
        let len = len_elems * core::mem::size_of::<bool>();
        let capacity = cap_elems * core::mem::size_of::<bool>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_BOOL,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<bool>();
                let cap_elems = capacity / core::mem::size_of::<bool>();
                let _ = Vec::from_raw_parts(ptr as *mut bool, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_bool(&self) -> Option<Vec<bool>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<bool>();
            let slice = core::slice::from_raw_parts(self.ptr as *const bool, len_elems);
            Some(slice.to_vec())
        }
    }
}
