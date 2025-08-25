// std-enabled VM crate

mod vm_host;
pub use vm_host::{KIND_F32, KIND_F64, KIND_STATICSTR, KIND_STRBUF, KIND_U8, KIND_U64, KaytonVm};

// Re-export common API types for convenience of VM users
pub use kayton_api::{
    ErrorKind, GlobalStrBuf as VmGlobalStrBuf, HKayGlobal as VmHKayGlobal, KaytonApi as Api,
    KaytonContext as VmKaytonContext, KaytonError as VmKaytonError,
};
