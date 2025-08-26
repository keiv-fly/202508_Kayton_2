use kayton_api::fns_dynamic::KindId;
use kayton_api::types::HKayGlobal;

// ---------------- Kind IDs and handle packing ----------------

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

const KIND_SHIFT: u64 = 32;
const IDX_MASK: u64 = (1u64 << 32) - 1;

#[inline]
pub(crate) fn pack_handle(kind: KindId, idx: u32) -> HKayGlobal {
    HKayGlobal(((kind as u64) << KIND_SHIFT) | (idx as u64 & IDX_MASK))
}

#[inline]
pub(crate) fn unpack_handle(h: HKayGlobal) -> (KindId, u32) {
    let raw = h.0;
    (((raw >> KIND_SHIFT) as u32), (raw & IDX_MASK) as u32)
}
