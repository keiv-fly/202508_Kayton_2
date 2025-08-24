extern crate alloc;

use alloc::{borrow::Cow, boxed::Box};
use core::ffi::c_void;
use core::fmt;

/// Error kinds for Kayton API operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive] // lets you add variants later without breaking users
pub enum ErrorKind {
    NotFound,
    Generic,
}

/// Kayton API error type
#[derive(Debug)]
pub struct KaytonError {
    kind: ErrorKind,
    msg: Cow<'static, str>,
    source: Option<Box<dyn core::error::Error + Send + Sync + 'static>>,
}

impl KaytonError {
    pub fn new(kind: ErrorKind, msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind,
            msg: msg.into(),
            source: None,
        }
    }

    pub fn with_source(
        kind: ErrorKind,
        msg: impl Into<Cow<'static, str>>,
        source: impl core::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            msg: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    pub fn message(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for KaytonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show both the enum (machine) and the string (human)
        write!(f, "{:?}: {}", self.kind, self.msg)
    }
}

impl core::error::Error for KaytonError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn core::error::Error + 'static))
    }
}

// Helpful From mappings so `?` works ergonomically
impl From<core::fmt::Error> for KaytonError {
    fn from(e: core::fmt::Error) -> Self {
        KaytonError::with_source(ErrorKind::Generic, "Format error", e)
    }
}

// Convenience constructors
impl KaytonError {
    pub fn not_found(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::NotFound, msg)
    }

    pub fn generic(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::Generic, msg)
    }
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
