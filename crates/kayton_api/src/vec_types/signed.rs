extern crate alloc;
use alloc::vec::Vec;

use crate::kinds::{KIND_I128, KIND_I16, KIND_I32, KIND_I64, KIND_I8, KIND_ISIZE};
use super::KVec;

impl KVec {
    pub fn from_vec_i8(vec: Vec<i8>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len = v.len();
        let capacity = v.capacity();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_I8,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let _ = Vec::from_raw_parts(ptr as *mut i8, len, capacity);
            }),
        }
    }

    pub fn as_vec_i8(&self) -> Option<Vec<i8>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let slice = core::slice::from_raw_parts(self.ptr as *const i8, self.len);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_i16(vec: Vec<i16>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<i16>();
        let capacity = cap_elems * core::mem::size_of::<i16>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_I16,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<i16>();
                let cap_elems = capacity / core::mem::size_of::<i16>();
                let _ = Vec::from_raw_parts(ptr as *mut i16, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_i16(&self) -> Option<Vec<i16>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<i16>();
            let slice = core::slice::from_raw_parts(self.ptr as *const i16, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_i32(vec: Vec<i32>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<i32>();
        let capacity = cap_elems * core::mem::size_of::<i32>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_I32,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<i32>();
                let cap_elems = capacity / core::mem::size_of::<i32>();
                let _ = Vec::from_raw_parts(ptr as *mut i32, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_i32(&self) -> Option<Vec<i32>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<i32>();
            let slice = core::slice::from_raw_parts(self.ptr as *const i32, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_i64(vec: Vec<i64>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<i64>();
        let capacity = cap_elems * core::mem::size_of::<i64>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_I64,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<i64>();
                let cap_elems = capacity / core::mem::size_of::<i64>();
                let _ = Vec::from_raw_parts(ptr as *mut i64, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_i64(&self) -> Option<Vec<i64>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<i64>();
            let slice = core::slice::from_raw_parts(self.ptr as *const i64, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_i128(vec: Vec<i128>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<i128>();
        let capacity = cap_elems * core::mem::size_of::<i128>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_I128,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<i128>();
                let cap_elems = capacity / core::mem::size_of::<i128>();
                let _ = Vec::from_raw_parts(ptr as *mut i128, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_i128(&self) -> Option<Vec<i128>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<i128>();
            let slice = core::slice::from_raw_parts(self.ptr as *const i128, len_elems);
            Some(slice.to_vec())
        }
    }

    pub fn from_vec_isize(vec: Vec<isize>) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len_elems = v.len();
        let cap_elems = v.capacity();
        let len = len_elems * core::mem::size_of::<isize>();
        let capacity = cap_elems * core::mem::size_of::<isize>();
        core::mem::forget(v);
        Self {
            ptr: ptr as *const u8,
            len,
            capacity,
            kind: KIND_ISIZE,
            drop_fn: Some(|ptr, len, capacity, _kind| unsafe {
                let len_elems = len / core::mem::size_of::<isize>();
                let cap_elems = capacity / core::mem::size_of::<isize>();
                let _ = Vec::from_raw_parts(ptr as *mut isize, len_elems, cap_elems);
            }),
        }
    }

    pub fn as_vec_isize(&self) -> Option<Vec<isize>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let len_elems = self.len / core::mem::size_of::<isize>();
            let slice = core::slice::from_raw_parts(self.ptr as *const isize, len_elems);
            Some(slice.to_vec())
        }
    }
}
