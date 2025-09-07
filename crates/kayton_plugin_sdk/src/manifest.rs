extern crate alloc;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// Supported simple kinds across the plugin boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeKind {
    Unit,
    Bool,
    I64,
    U64,
    F64,
    StaticStr,
    StringBuf,
    VecI64,
    VecF64,
    Dynamic,
}

/// Function signature descriptor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub params: Vec<TypeKind>,
    pub ret: TypeKind,
}

/// Function entry in the plugin manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionEntry {
    /// Stable public name used by compilers/typecheckers.
    pub stable_name: String,
    /// Raw symbol to look up in the DLL.
    pub symbol: String,
    /// Compact signature description for validation and marshaling.
    pub sig: Signature,
}

/// Type entry in the plugin manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeEntry {
    /// Stable type name (e.g., "reqwest::Client").
    pub name: String,
    /// Kind category; opaque types should use Dynamic.
    pub kind: TypeKind,
    /// Size in bytes (0 for opaque).
    pub size: u32,
    /// Alignment in bytes (0 for opaque).
    pub align: u32,
}

/// Top-level manifest returned by plugins.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub abi_version: u32,
    pub crate_name: String,
    pub crate_version: String,
    pub functions: Vec<FunctionEntry>,
    pub types: Vec<TypeEntry>,
}

impl Manifest {
    pub fn to_json_bytes(&self) -> alloc::vec::Vec<u8> {
        // Using serde_json in no_std with alloc enabled
        serde_json::to_vec(self).expect("serialize manifest")
    }
}
