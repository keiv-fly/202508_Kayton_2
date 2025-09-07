use kayton_api::kinds::KIND_BOOL;
use kayton_api::vec_types::KVec;

#[test]
fn from_vec_bool_roundtrip() {
    let expected = vec![true, false, true];
    let kv = KVec::from_vec_bool(expected.clone());
    assert_eq!(kv.len, expected.len());
    assert_eq!(kv.capacity, expected.capacity());
    assert_eq!(kv.kind, KIND_BOOL);
    let back = kv.as_vec_bool().unwrap();
    assert_eq!(back, expected);
}
