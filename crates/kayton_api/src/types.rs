use core::ffi::c_void;

/// Status codes: 0 = Ok, 1 = Error
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KaytonStatus {
    Ok = 0,
    Error = 1,
}

/// Opaque global handle (u64 inside)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct HKayGlobal(pub u64);

/// Opaque VM context surface passed to plugins
#[repr(C)]
pub struct KaytonContext {
    /// ABI version (bump on breaking changes)
    pub abi_version: u64,

    /// Host-owned, plugin-opaque pointer (your VM can stash anything here).
    pub host_data: *mut c_void,

    /// Root API vtable (flat, HPy-style)
    pub api: *const crate::api::KaytonApi,
}
