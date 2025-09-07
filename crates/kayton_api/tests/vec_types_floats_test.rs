use core::mem::size_of;
use kayton_api::kinds::{KIND_F32, KIND_F64};
use kayton_api::vec_types::KVec;

#[test]
fn from_vec_f32_roundtrip() {
    let expected = vec![1.0f32, 2.5, 3.0];
    let kv = KVec::from_vec_f32(expected.clone());
    assert_eq!(kv.len, expected.len() * size_of::<f32>());
    assert_eq!(kv.capacity, expected.capacity() * size_of::<f32>());
    assert_eq!(kv.kind, KIND_F32);
    let back = kv.as_vec_f32().unwrap();
    assert_eq!(back, expected);
}

#[test]
fn from_vec_f64_roundtrip() {
    let expected = vec![1.0f64, 2.5, 3.0];
    let kv = KVec::from_vec_f64(expected.clone());
    assert_eq!(kv.len, expected.len() * size_of::<f64>());
    assert_eq!(kv.capacity, expected.capacity() * size_of::<f64>());
    assert_eq!(kv.kind, KIND_F64);
    let back = kv.as_vec_f64().unwrap();
    assert_eq!(back, expected);
}
