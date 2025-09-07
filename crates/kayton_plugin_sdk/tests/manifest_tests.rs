extern crate alloc;
use kayton_plugin_sdk::{
    self, KAYTON_PLUGIN_ABI_VERSION, TypeKind, kayton_manifest, leak_manifest_json_bytes,
    manifest_to_static_json,
};

#[test]
fn abi_version_matches() {
    assert_eq!(KAYTON_PLUGIN_ABI_VERSION, 1);
}

#[test]
fn leak_manifest_returns_same_bytes() {
    let data = vec![1, 2, 3, 4];
    let leaked = leak_manifest_json_bytes(data);
    assert_eq!(leaked, &[1, 2, 3, 4]);
}

#[test]
fn manifest_macro_and_json_roundtrip() {
    let manifest = kayton_manifest!(
        crate_name = "test_crate",
        crate_version = "0.1.0",
        functions = [
            { stable: "add", symbol: "add_fn", params: [I64, I64], ret: I64 },
        ],
        types = [
            { name: "MyType", kind: Dynamic, size: 0, align: 0 },
        ]
    );

    assert_eq!(manifest.abi_version, KAYTON_PLUGIN_ABI_VERSION);
    assert_eq!(manifest.crate_name, "test_crate");
    assert_eq!(manifest.crate_version, "0.1.0");
    assert_eq!(manifest.functions.len(), 1);
    assert_eq!(manifest.functions[0].stable_name, "add");
    assert_eq!(manifest.functions[0].symbol, "add_fn");
    assert_eq!(
        manifest.functions[0].sig.params,
        vec![TypeKind::I64, TypeKind::I64]
    );
    assert_eq!(manifest.functions[0].sig.ret, TypeKind::I64);
    assert_eq!(manifest.types.len(), 1);
    assert_eq!(manifest.types[0].name, "MyType");
    assert_eq!(manifest.types[0].kind, TypeKind::Dynamic);
    assert_eq!(manifest.types[0].size, 0);
    assert_eq!(manifest.types[0].align, 0);

    let json = manifest_to_static_json(&manifest);
    let parsed: kayton_plugin_sdk::Manifest = serde_json::from_slice(json).unwrap();
    assert_eq!(parsed, manifest);
}
