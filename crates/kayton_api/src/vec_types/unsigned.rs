extern crate alloc;
use alloc::vec::Vec;

use crate::kinds::{KIND_U128, KIND_U16, KIND_U32, KIND_U64, KIND_U8, KIND_USIZE};
use super::KVec;

impl KVec {
    pub fn from_vec_u8(vec: Vec<u8>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len = v.len();
        let capacity = v.capacity();
        core::mem::forget(v);
        Self {
            ptr,
            len,
            capacity,
            kind: KIND_U8,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let _ = Vec::from_raw_parts(ptr as *mut u8, len, capacity);
            }),
        }
    }

    pub fn as_vec_u8(&self) -> Option<Vec<u8>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let slice = core::slice::from_raw_parts(self.ptr, self.len);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_u16(vec: Vec<u16>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<u16>();
        let capacity = cap_elems * core::mem::size_of::<u16>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_U16,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<u16>();
                let cap_elems = capacity / core::mem::size_of::<u16>();
                let _ = Vec::from_raw_parts(ptr as *mut u16, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_u16(&self) -> Option<Vec<u16>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<u16>();
            let slice = core::slice::from_raw_parts(self.ptr as *const u16, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_u32(vec: Vec<u32>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<u32>();
        let capacity = cap_elems * core::mem::size_of::<u32>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_U32,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<u32>();
                let cap_elems = capacity / core::mem::size_of::<u32>();
                let _ = Vec::from_raw_parts(ptr as *mut u32, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_u32(&self) -> Option<Vec<u32>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<u32>();
            let slice = core::slice::from_raw_parts(self.ptr as *const u32, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_u64(vec: Vec<u64>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<u64>();
        let capacity = cap_elems * core::mem::size_of::<u64>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_U64,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<u64>();
                let cap_elems = capacity / core::mem::size_of::<u64>();
                let _ = Vec::from_raw_parts(ptr as *mut u64, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_u64(&self) -> Option<Vec<u64>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<u64>();
            let slice = core::slice::from_raw_parts(self.ptr as *const u64, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_u128(vec: Vec<u128>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<u128>();
        let capacity = cap_elems * core::mem::size_of::<u128>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_U128,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<u128>();
                let cap_elems = capacity / core::mem::size_of::<u128>();
                let _ = Vec::from_raw_parts(ptr as *mut u128, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_u128(&self) -> Option<Vec<u128>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<u128>();
            let slice = core::slice::from_raw_parts(self.ptr as *const u128, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_usize(vec: Vec<usize>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<usize>();
        let capacity = cap_elems * core::mem::size_of::<usize>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_USIZE,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<usize>();
                let cap_elems = capacity / core::mem::size_of::<usize>();
                let _ = Vec::from_raw_parts(ptr as *mut usize, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_usize(&self) -> Option<Vec<usize>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<usize>();
            let slice = core::slice::from_raw_parts(self.ptr as *const usize, len_elems);
            Some(slice.to_vec())
        }
    }
}
