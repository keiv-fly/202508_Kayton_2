//! Macros to help declare plugin manifests and exports.
#![allow(unused_macros)]

#[macro_export]
macro_rules! kayton_manifest {
    (
        crate_name = $crate_name:expr,
        crate_version = $crate_version:expr,
        functions = [ $( { stable: $stable:expr, symbol: $symbol:expr, params: [ $( $p:ident ),* ], ret: $r:ident } ),* $(,)? ],
        types = [ $( { name: $tname:expr, kind: $tkind:ident, size: $tsize:expr, align: $talign:expr } ),* $(,)? ]
    ) => {{
        use $crate::manifest::{Manifest, FunctionEntry, Signature, TypeEntry, TypeKind};
        let mut fns = alloc::vec::Vec::new();
        $(
            let mut params = alloc::vec::Vec::new();
            $( params.push(TypeKind::$p); )*
            fns.push(FunctionEntry{ stable_name: $stable.to_string(), symbol: $symbol.to_string(), sig: Signature{ params, ret: TypeKind::$r } });
        )*
        let mut tys = alloc::vec::Vec::new();
        $( tys.push(TypeEntry{ name: $tname.to_string(), kind: TypeKind::$tkind, size: $tsize, align: $talign }); )*
        Manifest{ abi_version: $crate::KAYTON_PLUGIN_ABI_VERSION, crate_name: $crate_name.to_string(), crate_version: $crate_version.to_string(), functions: fns, types: tys }
    }};
}

/// Define default exported ABI symbols using the provided manifest value and register function.
///
/// Usage:
/// ```ignore
/// static MY_MANIFEST: Manifest = ...;
/// fn do_register(ctx: &mut KaytonContext) { /* register fns/types with VM */ }
/// kayton_exports!(MY_MANIFEST, do_register);
/// ```
#[macro_export]
macro_rules! kayton_exports {
    ($manifest_static:expr, $register_fn:path) => {
        #[no_mangle]
        pub extern "Rust" fn kayton_plugin_abi_version() -> u32 {
            $crate::KAYTON_PLUGIN_ABI_VERSION
        }

        #[no_mangle]
        pub extern "Rust" fn kayton_plugin_manifest_json() -> &'static [u8] {
            $crate::rust_abi::manifest_to_static_json(&$manifest_static)
        }

        #[no_mangle]
        pub extern "Rust" fn kayton_plugin_register(ctx: &mut kayton_api::KaytonContext) {
            $register_fn(ctx)
        }
    };
}
