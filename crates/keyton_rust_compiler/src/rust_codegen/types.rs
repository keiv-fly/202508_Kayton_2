use crate::shir::sym::SymbolId;

/// A complete Rust program as source code
#[derive(Debug)]
pub struct RustCode {
    pub source_code: String,
    pub var_names: std::collections::HashMap<SymbolId, String>,
}
