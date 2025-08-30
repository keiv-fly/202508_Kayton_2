use std::collections::BTreeMap;
use std::string::String;
use std::vec::Vec;

use kayton_api::kinds::KindId;
use kayton_api::types::{GlobalStrBuf, HKayRef};

mod dyn_store;
mod vm_fns_dynamic;
mod vm_fns_float;
mod vm_fns_intern;
mod vm_fns_sint;
mod vm_fns_string;
mod vm_fns_tuple;
mod vm_fns_uint;

use dyn_store::DynKindStore;

// ---------------- Host state ----------------

pub struct HostState {
    name_to_handle: BTreeMap<String, HKayRef>,
    handle_to_name: BTreeMap<(u32, u32), String>,

    u64s: Vec<u64>,
    u8s: Vec<u8>,
    u32s: Vec<u32>,
    u16s: Vec<u16>,
    u128s: Vec<u128>,
    usizes: Vec<usize>,
    i8s: Vec<i8>,
    i16s: Vec<i16>,
    i32s: Vec<i32>,
    i64s: Vec<i64>,
    i128s: Vec<i128>,
    isizes: Vec<isize>,
    bools: Vec<bool>,
    f64s: Vec<f64>,
    f32s: Vec<f32>,
    static_strs: Vec<&'static str>,
    str_bufs: Vec<Option<GlobalStrBuf>>,

    // Tuple storage: flat items and (start,len) metadata per tuple
    tuple_items: Vec<HKayRef>,
    tuples: Vec<(u32, u32)>,

    next_kind_id: KindId,
    dyn_kinds: BTreeMap<KindId, DynKindStore>,
}

impl HostState {
    pub fn new() -> Self {
        Self {
            name_to_handle: BTreeMap::new(),
            handle_to_name: BTreeMap::new(),
            u64s: Vec::new(),
            u8s: Vec::new(),
            u32s: Vec::new(),
            u16s: Vec::new(),
            u128s: Vec::new(),
            usizes: Vec::new(),
            i8s: Vec::new(),
            i16s: Vec::new(),
            i32s: Vec::new(),
            i64s: Vec::new(),
            i128s: Vec::new(),
            isizes: Vec::new(),
            bools: Vec::new(),
            f64s: Vec::new(),
            f32s: Vec::new(),
            static_strs: Vec::new(),
            str_bufs: Vec::new(),
            tuple_items: Vec::new(),
            tuples: Vec::new(),
            next_kind_id: 1000,
            dyn_kinds: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn bind_name(&mut self, name: &str, h: HKayRef) {
        self.name_to_handle.insert(String::from(name), h);
        self.handle_to_name
            .insert((h.kind as u32, h.index as u32), String::from(name));
    }

    #[inline]
    pub fn resolve(&self, name: &str) -> Option<HKayRef> {
        self.name_to_handle.get(name).copied()
    }
}

impl Drop for HostState {
    fn drop(&mut self) {
        for (_, store) in self.dyn_kinds.iter_mut() {
            store.drop_all();
        }
    }
}
