use core::mem::size_of;

use kayton_api::fns_float::{GetGlobalF32Fn, GetGlobalF64Fn, SetGlobalF32Fn, SetGlobalF64Fn};
use kayton_api::fns_int::{GetGlobalU8Fn, GetGlobalU64Fn, SetGlobalU8Fn, SetGlobalU64Fn};

#[test]
fn function_pointer_sizes_match_plain_pointer() {
    let ptr_size = size_of::<*const ()>();

    assert_eq!(size_of::<SetGlobalU64Fn>(), ptr_size);
    assert_eq!(size_of::<GetGlobalU64Fn>(), ptr_size);
    assert_eq!(size_of::<SetGlobalU8Fn>(), ptr_size);
    assert_eq!(size_of::<GetGlobalU8Fn>(), ptr_size);

    assert_eq!(size_of::<SetGlobalF64Fn>(), ptr_size);
    assert_eq!(size_of::<GetGlobalF64Fn>(), ptr_size);
    assert_eq!(size_of::<SetGlobalF32Fn>(), ptr_size);
    assert_eq!(size_of::<GetGlobalF32Fn>(), ptr_size);
}
