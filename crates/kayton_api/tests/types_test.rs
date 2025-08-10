use core::mem::{align_of, size_of};

use kayton_api::{HKayGlobal, KaytonStatus};

#[test]
fn kayton_status_discriminants() {
    assert_eq!(KaytonStatus::Ok as u32, 0);
    assert_eq!(KaytonStatus::Error as u32, 1);
}

#[test]
fn hkayglobal_layout() {
    assert_eq!(size_of::<HKayGlobal>(), size_of::<u64>());
    assert_eq!(align_of::<HKayGlobal>(), align_of::<u64>());
}
