use core::mem::{align_of, size_of};

use kayton_api::{ErrorKind, HKayRef, KaytonError};

#[test]
fn kayton_error_kinds() {
    let not_found = KaytonError::not_found("Resource not found");
    assert_eq!(not_found.kind(), ErrorKind::NotFound);
    assert_eq!(not_found.message(), "Resource not found");

    let generic = KaytonError::generic("Something went wrong");
    assert_eq!(generic.kind(), ErrorKind::Generic);
    assert_eq!(generic.message(), "Something went wrong");
}

#[test]
fn kayton_error_display() {
    let error = KaytonError::not_found("Test error message");
    let display = format!("{}", error);
    assert!(display.contains("NotFound"));
    assert!(display.contains("Test error message"));
}

#[test]
fn hkayref_layout() {
    // HKayRef is a two-field u32 struct
    assert_eq!(size_of::<HKayRef>(), 8);
    assert_eq!(align_of::<HKayRef>(), align_of::<u32>());
}
