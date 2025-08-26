extern crate alloc;

use alloc::{borrow::Cow, boxed::Box, string::String};
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

/// Universal handle to any VM value (kind + per-kind index)
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct HKayRef {
    pub kind: u32,
    pub index: u32,
}

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

/// String buffer structure with pointer, length, and capacity
#[repr(C)]
pub struct GlobalStrBuf {
    /// Pointer to the string data
    pub ptr: *const u8,
    /// Length of the string in bytes
    pub len: usize,
    /// Capacity of the buffer in bytes
    pub capacity: usize,
    /// Drop function to call when this buffer is dropped
    pub drop_fn: Option<fn(*const u8, usize, usize)>,
}

impl GlobalStrBuf {
    /// Create a new GlobalStrBuf from a String
    pub fn new(string: String) -> Self {
        let ptr = string.as_ptr();
        let len = string.len();
        let capacity = string.capacity();

        // Leak the string to prevent it from being dropped automatically
        // The drop function will handle cleanup
        let _ = Box::leak(string.into_boxed_str());

        Self {
            ptr,
            len,
            capacity,
            drop_fn: Some(|ptr, len, _capacity| {
                // Safety: We know this pointer was created from a leaked String
                unsafe {
                    let slice = core::slice::from_raw_parts_mut(ptr as *mut u8, len);
                    let _ = Box::from_raw(slice as *mut [u8] as *mut str);
                }
            }),
        }
    }

    /// Create a new GlobalStrBuf from raw components
    pub fn from_raw(ptr: *const u8, len: usize, capacity: usize) -> Self {
        Self {
            ptr,
            len,
            capacity,
            drop_fn: None,
        }
    }

    /// Convert to a string slice if the pointer is valid
    pub fn as_str(&self) -> Option<&str> {
        if self.ptr.is_null() {
            return None;
        }
        unsafe {
            let slice = core::slice::from_raw_parts(self.ptr, self.len);
            core::str::from_utf8(slice).ok()
        }
    }
}

impl Drop for GlobalStrBuf {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            drop_fn(self.ptr, self.len, self.capacity);
        }
    }
}
