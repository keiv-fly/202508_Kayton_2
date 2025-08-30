extern crate alloc;
use crate::kinds::{
    KIND_BOOL, KIND_F32, KIND_F64, KIND_I8, KIND_I16, KIND_I32, KIND_I64, KIND_I128, KIND_ISIZE,
    KIND_U8, KIND_U16, KIND_U32, KIND_U64, KIND_U128, KIND_USIZE, KindId,
};
use alloc::vec::Vec;

/// Generic vector buffer with element kind, pointer, length, and capacity
#[repr(C)]
pub struct KVec {
    /// Pointer to the vector's storage (as bytes)
    pub ptr: *const u8,
    /// Length in bytes
    pub len: usize,
    /// Capacity in bytes
    pub capacity: usize,
    /// KindId of the element type contained in this vector
    pub kind: KindId,
    /// Drop function to call when this buffer is dropped
    pub drop_fn: Option<fn(*const u8, usize, usize, KindId)>,
}

impl KVec {
    /// Construct from raw components. Caller guarantees pointer validity/lifetime.
    pub fn from_raw(ptr: *const u8, len: usize, capacity: usize, kind: KindId) -> Self {
        Self {
            ptr,
            len,
            capacity,
            kind,
            drop_fn: None,
        }
    }

    /// Create from an owned Vec<u8>. Memory is reclaimed in Drop by reconstructing the Vec.
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

    /// Convert the buffer into a Vec<u8> copy if the pointer is valid.
    pub fn as_vec_u8(&self) -> Option<Vec<u8>> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let slice = core::slice::from_raw_parts(self.ptr, self.len);
            Some(slice.to_vec())
        }
    }

    // ===== bool =====
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

    // ===== Unsigned integers =====
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

    // ===== Signed integers =====
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

    // ===== Floats =====
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

impl Drop for KVec {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            drop_fn(self.ptr, self.len, self.capacity, self.kind);
        }
    }
}
