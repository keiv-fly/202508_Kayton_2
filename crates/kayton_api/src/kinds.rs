// ---------------- Kind IDs ----------------

/// 32-bit kind identifier used in handles
pub type KindId = u32;

pub const KIND_U64: KindId = 1;
pub const KIND_U8: KindId = 2;
pub const KIND_F64: KindId = 3;
pub const KIND_F32: KindId = 4;
pub const KIND_STATICSTR: KindId = 5;
pub const KIND_STRBUF: KindId = 6;
pub const KIND_U32: KindId = 7;
pub const KIND_U16: KindId = 8;
pub const KIND_U128: KindId = 9;
pub const KIND_USIZE: KindId = 10;
pub const KIND_I8: KindId = 11;
pub const KIND_I16: KindId = 12;
pub const KIND_I32: KindId = 13;
pub const KIND_I64: KindId = 14;
pub const KIND_I128: KindId = 15;
pub const KIND_ISIZE: KindId = 16;
pub const KIND_BOOL: KindId = 17;
pub const KIND_TUPLE: KindId = 18;
pub const KIND_KVEC: KindId = 19;
