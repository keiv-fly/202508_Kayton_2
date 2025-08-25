#![no_std]
extern crate core;

pub mod api;
pub mod fns_dynamic;
pub mod fns_float;
pub mod fns_int;
pub mod fns_string;
pub mod types;

// Explicit re-exports (no globs); function typedefs are NOT re-exported.
pub use types::{ErrorKind, GlobalStrBuf, HKayGlobal, KaytonContext, KaytonError};

pub use api::KaytonApi;

// Re-export function pointer typedefs for convenience within Rust-only dynamic linking
pub use fns_dynamic::*;
pub use fns_float::*;
pub use fns_int::*;
pub use fns_string::*;
