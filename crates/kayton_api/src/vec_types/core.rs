use crate::kinds::KindId;

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
}

impl Drop for KVec {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            drop_fn(self.ptr, self.len, self.capacity, self.kind);
        }
    }
}
