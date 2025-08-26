use std::boxed::Box;
use std::ffi::c_void;

use kayton_api::api::KaytonApi;
use kayton_api::types::{GlobalStrBuf, KaytonContext};

use crate::host::HostState;

// ---------------- Kayton VM wrapper ----------------

pub struct KaytonVm {
    host: Box<HostState>,
    api: Box<KaytonApi>,
}

impl KaytonVm {
    pub fn new() -> Self {
        let host = Box::new(HostState::new());

        // Instantiate API vtable
        let api = Box::new(KaytonApi {
            size: core::mem::size_of::<KaytonApi>() as u64,

            set_global_u64: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u64(name, v))
            },
            get_global_u64: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u64(name)
            },

            set_global_u8: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u8(name, v))
            },
            get_global_u8: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u8(name)
            },

            set_global_f64: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_f64(name, v))
            },
            get_global_f64: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f64(name)
            },

            set_global_f32: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_f32(name, v))
            },
            get_global_f32: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f32(name)
            },

            set_global_static_str: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_static_str(name, v))
            },
            get_global_static_str: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_static_str(name)
            },

            set_global_str_buf: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_str_buf(name, v))
            },
            get_global_str_buf: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                // Rebuild a by-value copy without drop_fn to avoid double-drop
                let sb = s.get_str_buf(name)?;
                Ok(GlobalStrBuf::from_raw(sb.ptr, sb.len, sb.capacity))
            },

            register_dynamic_kind: |ctx, name, drop_fn| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.register_dynamic_kind(name, drop_fn)
            },
            set_global_dyn_ptr: |ctx, kind, name, value| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.set_dyn_by_name(kind, name, value)
            },
            get_global_dyn_ptr: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_dyn_by_name(name)
            },
            get_global_dyn_ptr_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_dyn_by_handle(h)
            },

            get_global_u64_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u64_by_handle(h)
            },
            get_global_u8_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u8_by_handle(h)
            },

            get_global_f64_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f64_by_handle(h)
            },
            get_global_f32_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_f32_by_handle(h)
            },

            get_global_static_str_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_static_str_by_handle(h)
            },
            get_global_str_buf_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_str_buf_by_handle(h)
            },

            // ---- New integer/bool functions ----
            set_global_u32: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u32(name, v))
            },
            get_global_u32: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u32(name)
            },
            get_global_u32_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u32_by_handle(h)
            },

            set_global_u16: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u16(name, v))
            },
            get_global_u16: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u16(name)
            },
            get_global_u16_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u16_by_handle(h)
            },

            set_global_u128: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_u128(name, v))
            },
            get_global_u128: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u128(name)
            },
            get_global_u128_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_u128_by_handle(h)
            },

            set_global_usize: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_usize(name, v))
            },
            get_global_usize: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_usize(name)
            },
            get_global_usize_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_usize_by_handle(h)
            },

            set_global_i8: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i8(name, v))
            },
            get_global_i8: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i8(name)
            },
            get_global_i8_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i8_by_handle(h)
            },

            set_global_i16: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i16(name, v))
            },
            get_global_i16: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i16(name)
            },
            get_global_i16_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i16_by_handle(h)
            },

            set_global_i32: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i32(name, v))
            },
            get_global_i32: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i32(name)
            },
            get_global_i32_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i32_by_handle(h)
            },

            set_global_i64: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i64(name, v))
            },
            get_global_i64: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i64(name)
            },
            get_global_i64_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i64_by_handle(h)
            },

            set_global_i128: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_i128(name, v))
            },
            get_global_i128: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i128(name)
            },
            get_global_i128_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_i128_by_handle(h)
            },

            set_global_isize: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_isize(name, v))
            },
            get_global_isize: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_isize(name)
            },
            get_global_isize_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_isize_by_handle(h)
            },

            set_global_bool: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_bool(name, v))
            },
            get_global_bool: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_bool(name)
            },
            get_global_bool_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_bool_by_handle(h)
            },
        });

        KaytonVm { host, api }
    }

    pub fn context(&mut self) -> KaytonContext {
        KaytonContext {
            abi_version: 1,
            host_data: &mut *self.host as *mut HostState as *mut c_void,
            api: &*self.api as *const KaytonApi,
        }
    }

    pub fn api(&self) -> &KaytonApi {
        &*self.api
    }
}
