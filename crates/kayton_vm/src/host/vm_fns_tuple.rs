use kayton_api::types::{HKayRef, KaytonError};

use super::HostState;
use crate::kinds::{pack_handle, unpack_handle};
use kayton_api::kinds::KIND_TUPLE;

impl HostState {
    pub fn set_tuple_from_handles(
        &mut self,
        name: &str,
        items: *const HKayRef,
        len: usize,
    ) -> Result<HKayRef, KaytonError> {
        if len > 0 && items.is_null() {
            return Err(KaytonError::generic("null items with nonzero len"));
        }
        let start = self.tuple_items.len() as u32;
        // Safety: caller promises `len` items at `items`
        let slice = unsafe { core::slice::from_raw_parts(items, len) };
        self.tuple_items.extend_from_slice(slice);
        let tuple_idx = self.tuples.len() as u32;
        self.tuples.push((start, len as u32));
        let h = pack_handle(KIND_TUPLE, tuple_idx);
        self.bind_name(name, h);
        Ok(h)
    }

    pub fn get_tuple_len_by_name(&self, name: &str) -> Result<usize, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        self.get_tuple_len_by_handle(h)
    }

    pub fn get_tuple_len_by_handle(&self, h: HKayRef) -> Result<usize, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_TUPLE {
            return Err(KaytonError::generic("wrong kind"));
        }
        self.tuples
            .get(idx as usize)
            .map(|&(_, len)| len as usize)
            .ok_or_else(|| KaytonError::generic("index out of range"))
    }

    pub fn get_tuple_item_by_name(&self, name: &str, index: usize) -> Result<HKayRef, KaytonError> {
        let h = self
            .resolve(name)
            .ok_or_else(|| KaytonError::not_found("no global"))?;
        self.get_tuple_item_by_index(h, index)
    }

    pub fn get_tuple_item_by_index(
        &self,
        h: HKayRef,
        index: usize,
    ) -> Result<HKayRef, KaytonError> {
        let (k, idx) = unpack_handle(h);
        if k != KIND_TUPLE {
            return Err(KaytonError::generic("wrong kind"));
        }
        let (start, len) = *self
            .tuples
            .get(idx as usize)
            .ok_or_else(|| KaytonError::generic("index out of range"))?;
        if index >= len as usize {
            return Err(KaytonError::generic("tuple index out of range"));
        }
        let off = (start as usize) + index;
        self.tuple_items
            .get(off)
            .copied()
            .ok_or_else(|| KaytonError::generic("tuple storage out of range"))
    }

    pub fn read_tuple_into_slice_by_handle(
        &self,
        h: HKayRef,
        out: *mut HKayRef,
        cap: usize,
    ) -> Result<usize, KaytonError> {
        if cap > 0 && out.is_null() {
            return Err(KaytonError::generic("null out with nonzero cap"));
        }
        let (k, idx) = unpack_handle(h);
        if k != KIND_TUPLE {
            return Err(KaytonError::generic("wrong kind"));
        }
        let (start, len) = *self
            .tuples
            .get(idx as usize)
            .ok_or_else(|| KaytonError::generic("index out of range"))?;
        let n = core::cmp::min(cap, len as usize);
        let src = &self.tuple_items[start as usize..(start as usize + len as usize)];
        // Safety: caller provides valid `out` with at least `cap` capacity
        unsafe {
            core::ptr::copy_nonoverlapping(src.as_ptr(), out, n);
        }
        Ok(n)
    }
}
