use crate::{Api, VmGlobalStrBuf, VmKaytonContext};

pub type ReportIntFn = extern "C" fn(name_ptr: *const u8, name_len: usize, value: i64);
pub type ReportStrFn =
    extern "C" fn(name_ptr: *const u8, name_len: usize, str_ptr: *const u8, str_len: usize);

static mut HOST_PTRS: Option<(usize, usize)> = None; // (host_data, api_ptr)

#[inline]
pub fn set_report_host_from_ctx(ctx: &mut VmKaytonContext) {
    unsafe {
        HOST_PTRS = Some((ctx.host_data as usize, ctx.api as usize));
    }
}

pub extern "C" fn host_report_int(name_ptr: *const u8, name_len: usize, value: i64) {
    unsafe {
        let name_slice = core::slice::from_raw_parts(name_ptr, name_len);
        if let Ok(name) = core::str::from_utf8(name_slice) {
            if let Some((host_data, api_ptr)) = HOST_PTRS {
                let mut ctx = VmKaytonContext {
                    abi_version: 1,
                    host_data: host_data as *mut core::ffi::c_void,
                    api: api_ptr as *const Api,
                };
                let api: &Api = ctx.api();
                let _ = (api.set_global_u64)(&mut ctx, name, value as u64);
            }
        }
    }
}

pub extern "C" fn host_report_str(
    name_ptr: *const u8,
    name_len: usize,
    str_ptr: *const u8,
    str_len: usize,
) {
    unsafe {
        let name_slice = core::slice::from_raw_parts(name_ptr, name_len);
        let str_slice = core::slice::from_raw_parts(str_ptr, str_len);
        if let (Ok(name), Ok(val)) = (
            core::str::from_utf8(name_slice),
            core::str::from_utf8(str_slice),
        ) {
            if let Some((host_data, api_ptr)) = HOST_PTRS {
                let mut ctx = VmKaytonContext {
                    abi_version: 1,
                    host_data: host_data as *mut core::ffi::c_void,
                    api: api_ptr as *const Api,
                };
                let api: &Api = ctx.api();
                let buf = VmGlobalStrBuf::new(val.to_string());
                let _ = (api.set_global_str_buf)(&mut ctx, name, buf);
            }
        }
    }
}
