use kayton_api::kinds::KindId;
use kayton_api::types::HKayRef;

// ---------------- Handle packing ----------------

#[inline]
pub(crate) fn pack_handle(kind: KindId, idx: u32) -> HKayRef {
    HKayRef { kind, index: idx }
}

#[inline]
pub(crate) fn unpack_handle(h: HKayRef) -> (KindId, u32) {
    (h.kind as u32, h.index as u32)
}
