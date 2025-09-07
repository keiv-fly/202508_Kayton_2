extern crate alloc;

use kayton_api::{KaytonContext, types::{RawFnPtr, TypeMeta}};
use kayton_plugin_sdk::{kayton_manifest, KAYTON_PLUGIN_ABI_VERSION};

#[repr(C)]
pub struct MyType {
    pub value: i64,
}

#[unsafe(no_mangle)]
pub extern "Rust" fn add(left: i64, right: i64) -> i64 {
    left + right
}

fn register(ctx: &mut KaytonContext) {
    (ctx.api().register_function)(
        ctx,
        "add",
        add as *const () as RawFnPtr,
        0,
    ).unwrap();
    (ctx.api().register_type)(
        ctx,
        "MyType",
        TypeMeta::pod(core::mem::size_of::<MyType>(), core::mem::align_of::<MyType>()),
    ).unwrap();
}

#[unsafe(no_mangle)]
pub extern "Rust" fn kayton_plugin_abi_version() -> u32 {
    KAYTON_PLUGIN_ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "Rust" fn kayton_plugin_manifest_json() -> &'static [u8] {
    let manifest = kayton_manifest!(
        crate_name = "plugin_hello",
        crate_version = "0.1.0",
        functions = [
            { stable: "add", symbol: "add", params: [I64, I64], ret: I64 }
        ],
        types = [
            { name: "MyType", kind: Dynamic, size: 8, align: 8 }
        ]
    );
    kayton_plugin_sdk::rust_abi::manifest_to_static_json(&manifest)
}

#[unsafe(no_mangle)]
pub extern "Rust" fn kayton_plugin_register(ctx: &mut KaytonContext) {
    register(ctx);
}
