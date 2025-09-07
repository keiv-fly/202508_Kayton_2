extern crate alloc;

use kayton_api::KaytonContext;

use crate::{Manifest, leak_manifest_json_bytes};

/// Exported function pointer type for `kayton_plugin_register` (defined by plugins).
pub type RegisterFn = fn(ctx: &mut KaytonContext);

/// Helper to expose a manifest as leaked JSON bytes.
/// Plugins should call this with their `Manifest` and return the slice from their own
/// `#[no_mangle] extern "Rust" fn kayton_plugin_manifest_json()` symbol.
pub fn manifest_to_static_json(manifest: &Manifest) -> &'static [u8] {
    leak_manifest_json_bytes(manifest.to_json_bytes())
}
