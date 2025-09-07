use core::mem::size_of;
use kayton_api::kinds::{KIND_U128, KIND_U16, KIND_U32, KIND_U64, KIND_U8, KIND_USIZE};
use kayton_api::vec_types::KVec;

macro_rules! test_unsigned {
    ($name:ident, $t:ty, $from:ident, $as:ident, $kind:ident) => {
        #[test]
        fn $name() {
            let expected: Vec<$t> = vec![1 as $t, 2 as $t, 3 as $t];
            let kv = KVec::$from(expected.clone());
            assert_eq!(kv.len, expected.len() * size_of::<$t>());
            assert_eq!(kv.capacity, expected.capacity() * size_of::<$t>());
            assert_eq!(kv.kind, $kind);
            let back = kv.$as().unwrap();
            assert_eq!(back, expected);
        }
    };
}

test_unsigned!(from_vec_u8_roundtrip, u8, from_vec_u8, as_vec_u8, KIND_U8);
test_unsigned!(from_vec_u16_roundtrip, u16, from_vec_u16, as_vec_u16, KIND_U16);
test_unsigned!(from_vec_u32_roundtrip, u32, from_vec_u32, as_vec_u32, KIND_U32);
test_unsigned!(from_vec_u64_roundtrip, u64, from_vec_u64, as_vec_u64, KIND_U64);
test_unsigned!(from_vec_u128_roundtrip, u128, from_vec_u128, as_vec_u128, KIND_U128);
test_unsigned!(from_vec_usize_roundtrip, usize, from_vec_usize, as_vec_usize, KIND_USIZE);
