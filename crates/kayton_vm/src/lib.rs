// std-enabled VM crate

mod host;
mod kinds;
mod reporters;
mod vm;

pub use kayton_api::kinds::{
    KIND_BOOL, KIND_F32, KIND_F64, KIND_I8, KIND_I16, KIND_I32, KIND_I64, KIND_I128, KIND_ISIZE,
    KIND_KVEC, KIND_STATICSTR, KIND_STRBUF, KIND_TUPLE, KIND_U8, KIND_U16, KIND_U32, KIND_U64,
    KIND_U128, KIND_USIZE,
};
pub use vm::KaytonVm;

// Re-export common API types for convenience of VM users
pub use kayton_api::{
    ErrorKind, GlobalStrBuf as VmGlobalStrBuf, HKayRef as VmHKayRef, KaytonApi as Api,
    KaytonContext as VmKaytonContext, KaytonError as VmKaytonError,
};

// Reporter helpers used by dynamically compiled code epilogues
pub use reporters::{
    ReportIntFn, ReportStrFn, host_report_int, host_report_str, set_report_host_from_ctx,
};
