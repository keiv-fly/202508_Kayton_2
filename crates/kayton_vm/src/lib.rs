// std-enabled VM crate

mod host;
mod kinds;
mod vm;

pub use kinds::{
    KIND_BOOL, KIND_F32, KIND_F64, KIND_I16, KIND_I32, KIND_I64, KIND_I128, KIND_ISIZE,
    KIND_STATICSTR, KIND_STRBUF, KIND_TUPLE, KIND_U8, KIND_U16, KIND_U32, KIND_U64, KIND_U128,
    KIND_USIZE,
};
pub use vm::KaytonVm;

// Re-export common API types for convenience of VM users
pub use kayton_api::{
    ErrorKind, GlobalStrBuf as VmGlobalStrBuf, HKayRef as VmHKayRef, KaytonApi as Api,
    KaytonContext as VmKaytonContext, KaytonError as VmKaytonError,
};
