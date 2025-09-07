use core::mem::size_of;
use kayton_api::kinds::{KIND_I128, KIND_I16, KIND_I32, KIND_I64, KIND_I8, KIND_ISIZE};
use kayton_api::vec_types::KVec;

macro_rules! test_signed {
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

test_signed!(from_vec_i8_roundtrip, i8, from_vec_i8, as_vec_i8, KIND_I8);
test_signed!(from_vec_i16_roundtrip, i16, from_vec_i16, as_vec_i16, KIND_I16);
test_signed!(from_vec_i32_roundtrip, i32, from_vec_i32, as_vec_i32, KIND_I32);
test_signed!(from_vec_i64_roundtrip, i64, from_vec_i64, as_vec_i64, KIND_I64);
test_signed!(from_vec_i128_roundtrip, i128, from_vec_i128, as_vec_i128, KIND_I128);
test_signed!(from_vec_isize_roundtrip, isize, from_vec_isize, as_vec_isize, KIND_ISIZE);
