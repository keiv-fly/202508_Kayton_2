use std::sync::atomic::{AtomicBool, Ordering};

use kayton_api::kinds::{KindId, KIND_U8};
use kayton_api::vec_types::KVec;

#[test]
fn from_raw_sets_fields() {
    let v = vec![1u8, 2, 3];
    let ptr = v.as_ptr();
    let len = v.len();
    let cap = v.capacity();
    {
        let kv = KVec::from_raw(ptr, len, cap, KIND_U8);
        assert_eq!(kv.ptr, ptr);
        assert_eq!(kv.len, len);
        assert_eq!(kv.capacity, cap);
        assert_eq!(kv.kind, KIND_U8);
        assert!(kv.drop_fn.is_none());
    }
    drop(v);
}

static DROPPED: AtomicBool = AtomicBool::new(false);

fn mark_drop(_ptr: *const u8, _len: usize, _capacity: usize, _kind: KindId) {
    DROPPED.store(true, Ordering::SeqCst);
}

#[test]
fn drop_invokes_drop_fn() {
    {
        let _kv = KVec {
            ptr: core::ptr::null(),
            len: 0,
            capacity: 0,
            kind: KIND_U8,
            drop_fn: Some(mark_drop),
        };
    }
    assert!(DROPPED.load(Ordering::SeqCst));
}
