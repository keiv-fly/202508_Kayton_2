use std::collections::HashMap;
use core::ffi::c_void;

use kayton_api::fns_registry::{sig64_mix, sig64_finish};
use kayton_api::types::{KaytonContext, KaytonError, RawFnPtr, TypeMeta};

#[derive(Default)]
struct Registry {
    functions: HashMap<String, (RawFnPtr, u64)>,
    types: HashMap<String, TypeMeta>,
}

fn register_function(
    ctx: &mut KaytonContext,
    name: &str,
    raw_ptr: RawFnPtr,
    sig_id: u64,
) -> Result<(), KaytonError> {
    unsafe {
        let reg = &mut *(ctx.host_data as *mut Registry);
        reg.functions.insert(name.to_string(), (raw_ptr, sig_id));
    }
    Ok(())
}

fn get_function(ctx: &mut KaytonContext, name: &str) -> Result<RawFnPtr, KaytonError> {
    unsafe {
        let reg = &mut *(ctx.host_data as *mut Registry);
        reg
            .functions
            .get(name)
            .map(|(ptr, _)| *ptr)
            .ok_or_else(|| KaytonError::not_found("function not found"))
    }
}

fn register_type(
    ctx: &mut KaytonContext,
    name: &str,
    meta: TypeMeta,
) -> Result<(), KaytonError> {
    unsafe {
        let reg = &mut *(ctx.host_data as *mut Registry);
        reg.types.insert(name.to_string(), meta);
    }
    Ok(())
}

fn get_type(ctx: &mut KaytonContext, name: &str) -> Result<TypeMeta, KaytonError> {
    unsafe {
        let reg = &mut *(ctx.host_data as *mut Registry);
        reg
            .types
            .get(name)
            .copied()
            .ok_or_else(|| KaytonError::not_found("type not found"))
    }
}

#[test]
fn function_registry_roundtrip() {
    let mut registry = Registry::default();
    let mut ctx = KaytonContext {
        abi_version: 1,
        host_data: &mut registry as *mut _ as *mut c_void,
        api: core::ptr::null(),
    };

    fn sample() -> u32 { 123 }
    let ptr = sample as RawFnPtr;
    register_function(&mut ctx, "sample", ptr, 0).unwrap();
    let fetched = get_function(&mut ctx, "sample").unwrap();
    assert_eq!(fetched, ptr);
}

#[test]
fn type_registry_roundtrip() {
    let mut registry = Registry::default();
    let mut ctx = KaytonContext {
        abi_version: 1,
        host_data: &mut registry as *mut _ as *mut c_void,
        api: core::ptr::null(),
    };

    let meta = TypeMeta::pod(core::mem::size_of::<u64>(), core::mem::align_of::<u64>());
    register_type(&mut ctx, "u64", meta).unwrap();
    let fetched = get_type(&mut ctx, "u64").unwrap();
    assert_eq!(fetched.size, core::mem::size_of::<u64>());
    assert_eq!(fetched.align, core::mem::align_of::<u64>());
    assert!(fetched.drop_value.is_none());
    assert!(fetched.clone_value.is_none());
}

#[test]
fn sig_helpers_produce_expected_value() {
    const EXPECTED_MIX: u64 = 0xbcecd7a58395922a;
    const EXPECTED_FINISH: u64 = 0xe0ab3c8afe3f2c88;
    assert_eq!(sig64_mix(0, 1), EXPECTED_MIX);
    assert_eq!(sig64_finish(EXPECTED_MIX), EXPECTED_FINISH);
}

#[test]
fn type_meta_pod_has_no_drop_or_clone() {
    let meta = TypeMeta::pod(4, 4);
    assert_eq!(meta.size, 4);
    assert_eq!(meta.align, 4);
    assert!(meta.drop_value.is_none());
    assert!(meta.clone_value.is_none());
}
