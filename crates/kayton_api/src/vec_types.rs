extern crate alloc;
use crate::kinds::KindId;
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
    pub fn from_vec_u8(vec: Vec<u8>, kind: KindId) -> Self {
        let mut v = vec;
        let ptr = v.as_mut_ptr();
        let len = v.len();
        let capacity = v.capacity();
        core::mem::forget(v);
        Self {
            ptr,
            len,
            capacity,
            kind,
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
}

impl Drop for KVec {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            drop_fn(self.ptr, self.len, self.capacity, self.kind);
        }
    }
}
