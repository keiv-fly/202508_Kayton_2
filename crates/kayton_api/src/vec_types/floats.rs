extern crate alloc;
use alloc::vec::Vec;

use crate::kinds::{KIND_F32, KIND_F64};
use super::KVec;

impl KVec {
    pub fn from_vec_f32(vec: Vec<f32>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<f32>();
        let capacity = cap_elems * core::mem::size_of::<f32>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_F32,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<f32>();
                let cap_elems = capacity / core::mem::size_of::<f32>();
                let _ = Vec::from_raw_parts(ptr as *mut f32, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_f32(&self) -> Option<Vec<f32>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<f32>();
            let slice = core::slice::from_raw_parts(self.ptr as *const f32, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_f64(vec: Vec<f64>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<f64>();
        let capacity = cap_elems * core::mem::size_of::<f64>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_F64,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<f64>();
                let cap_elems = capacity / core::mem::size_of::<f64>();
                let _ = Vec::from_raw_parts(ptr as *mut f64, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_f64(&self) -> Option<Vec<f64>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<f64>();
            let slice = core::slice::from_raw_parts(self.ptr as *const f64, len_elems);
            Some(slice.to_vec())
        }
    }
}
