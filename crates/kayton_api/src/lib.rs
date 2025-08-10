#![no_std]
extern crate core;

pub mod api;
pub mod fns_float;
pub mod fns_int;
pub mod types;

// Explicit re-exports (no globs); function typedefs are NOT re-exported.
pub use types::{HKayGlobal, KaytonContext, KaytonStatus};

pub use api::KaytonApi;

// Re-export function pointer typedefs at crate root so cbindgen can include them
pub use fns_float::*;
pub use fns_int::*;
