#![no_std]
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

pub mod macros;
pub mod manifest;
pub mod rust_abi;

pub use manifest::{FunctionEntry, Manifest, Signature, TypeEntry, TypeKind};
pub use rust_abi::{RegisterFn, manifest_to_static_json};

/// Current expected Kayton Plugin ABI version. Bump on breaking changes.
pub const KAYTON_PLUGIN_ABI_VERSION: u32 = 1;

/// Helper to leak a JSON manifest as a &'static [u8]
pub fn leak_manifest_json_bytes(bytes: Vec<u8>) -> &'static [u8] {
    let boxed: Box<[u8]> = bytes.into_boxed_slice();
    let slice: &'static [u8] = Box::leak(boxed);
    slice
}
