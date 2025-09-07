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
        // Convert into a boxed str first so we can capture the final, stable pointer
        let boxed: Box<str> = string.into_boxed_str();
        let ptr = boxed.as_ptr();
        let len = boxed.len();
        // Box<str> capacity equals length; we record len for capacity to keep invariants simple
        let capacity = len;

        // Leak the box; our drop_fn will reconstruct it and free later
        let _raw: *mut str = Box::into_raw(boxed);

        Self {
            ptr,
            len,
            capacity,
            drop_fn: Some(|ptr, len, _capacity| {
                // Safety: We know this pointer was created from a leaked Box<str>
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

// ---------------- Registry-related core types ----------------

/// Opaque raw function pointer used for registry lookups. Cast by the caller to the desired Rust ABI function type.
pub type RawFnPtr = *const c_void;

/// Function pointer type for dropping a value of a registered type in-place.
/// Receives a pointer to the value's storage; implementors must respect `TypeMeta::size` and `align`.
pub type DropValueFn = unsafe fn(ptr: *mut u8);

/// Optional clone function to duplicate a value from `src` into `dst` (both properly aligned and non-overlapping).
pub type CloneValueFn = unsafe fn(dst: *mut u8, src: *const u8);

/// Metadata describing a value type crossing the plugin boundary.
/// All functions use the Rust ABI and operate on raw memory locations.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TypeMeta {
    /// Size in bytes of the value type
    pub size: usize,
    /// Alignment in bytes of the value type
    pub align: usize,
    /// Optional opaque tag describing layout category for fast-path checks (0 = unspecified)
    pub layout_tag: u64,
    /// Optional drop function to clean up resources for this value when dropped
    pub drop_value: Option<DropValueFn>,
    /// Optional clone function to copy from src to dst
    pub clone_value: Option<CloneValueFn>,
}

impl TypeMeta {
    /// Construct a POD (plain-old-data) meta (no drop/clone).
    pub const fn pod(size: usize, align: usize) -> Self {
        Self {
            size,
            align,
            layout_tag: 0,
            drop_value: None,
            clone_value: None,
        }
    }
}
