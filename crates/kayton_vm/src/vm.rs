use std::boxed::Box;
use std::ffi::c_void;
use std::path::Path;

use kayton_api::api::KaytonApi;
use kayton_api::types::{GlobalStrBuf, KaytonContext, KaytonError, RawFnPtr, TypeMeta};

use crate::host::HostState;

// ---------------- Kayton VM wrapper ----------------

pub struct KaytonVm {
    host: Box<HostState>,
    api: Box<KaytonApi>,
    plugins: Vec<libloading::Library>,
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
            drop_global_dyn_ptr: |ctx, h| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.drop_dyn_by_handle(h)
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
            drop_global_str_buf: |ctx, h| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.drop_str_buf_by_handle(h)
            },

            // ---- KVec ----
            set_global_kvec: |ctx, name, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                Ok(s.set_kvec(name, v))
            },
            get_global_kvec: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_kvec(name)
            },
            get_global_kvec_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_kvec_by_handle(h)
            },
            drop_global_kvec: |ctx, h| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.drop_kvec_by_handle(h)
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

            // ---- Interners ----
            intern_u64: |ctx, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.intern_u64(v)
            },
            intern_u8: |ctx, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.intern_u8(v)
            },
            intern_f64: |ctx, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.intern_f64(v)
            },
            intern_f32: |ctx, v| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.intern_f32(v)
            },
            intern_static_str: |ctx, sstr| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.intern_static_str(sstr)
            },
            intern_str_buf: |ctx, sref| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.intern_str_buf(sref)
            },
            intern_dyn_ptr: |ctx, kind, ptr| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.intern_dyn_ptr(kind, ptr)
            },

            // ---- Tuples ----
            set_global_tuple_from_handles: |ctx, name, items, len| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.set_tuple_from_handles(name, items, len)
            },
            get_global_tuple_len: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_tuple_len_by_name(name)
            },
            get_tuple_len_by_handle: |ctx, h| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_tuple_len_by_handle(h)
            },
            get_global_tuple_item: |ctx, name, index| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_tuple_item_by_name(name, index)
            },
            get_global_tuple_item_by_handle: |ctx, h, index| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_tuple_item_by_index(h, index)
            },
            read_tuple_into_slice_by_handle: |ctx, h, out, cap| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.read_tuple_into_slice_by_handle(h, out, cap)
            },

            // ---- Registries ----
            register_function: |ctx, name, raw_ptr, sig_id| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.register_function(name, raw_ptr, sig_id)
            },
            get_function: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_function(name)
            },
            register_type: |ctx, name, meta| {
                let s = unsafe { &mut *(ctx.host_data as *mut HostState) };
                s.register_type(name, meta)
            },
            get_type: |ctx, name| {
                let s = unsafe { &*(ctx.host_data as *mut HostState) };
                s.get_type(name)
            },
        });

        KaytonVm {
            host,
            api,
            plugins: Vec::new(),
        }
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

    /// Snapshot current globals (names and handles) for inspection by hosts/kernels.
    pub fn snapshot_globals(&self) -> Vec<(String, kayton_api::types::HKayRef)> {
        self.host.snapshot_globals()
    }

    /// Resolve a global by name to its handle, if present.
    pub fn resolve_name(&self, name: &str) -> Option<kayton_api::types::HKayRef> {
        self.host.resolve(name)
    }

    /// Format a VM value referenced by handle as a human-readable string.
    pub fn format_value_by_handle(
        &mut self,
        h: kayton_api::types::HKayRef,
    ) -> Result<String, KaytonError> {
        use crate::{
            KIND_BOOL, KIND_F32, KIND_F64, KIND_I8, KIND_I16, KIND_I32, KIND_I64, KIND_I128,
            KIND_ISIZE, KIND_KVEC, KIND_STATICSTR, KIND_STRBUF, KIND_TUPLE, KIND_U8, KIND_U16,
            KIND_U32, KIND_U64, KIND_U128, KIND_USIZE,
        };

        let mut ctx = self.context();
        // Avoid borrowing `ctx` immutably while also passing &mut ctx to API calls
        let api_ptr = ctx.api;
        let api: &KaytonApi = unsafe { &*api_ptr };
        let k = h.kind as u32;
        let out = if k == KIND_U64 {
            (api.get_global_u64_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_U32 {
            (api.get_global_u32_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_U16 {
            (api.get_global_u16_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_U8 {
            (api.get_global_u8_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_U128 {
            (api.get_global_u128_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_USIZE {
            (api.get_global_usize_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_I64 {
            (api.get_global_i64_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_I32 {
            (api.get_global_i32_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_I16 {
            (api.get_global_i16_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_I8 {
            (api.get_global_i8_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_I128 {
            (api.get_global_i128_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_ISIZE {
            (api.get_global_isize_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_BOOL {
            (api.get_global_bool_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_F64 {
            (api.get_global_f64_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_F32 {
            (api.get_global_f32_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_STATICSTR {
            (api.get_global_static_str_by_handle)(&mut ctx, h).map(|v| v.to_string())?
        } else if k == KIND_STRBUF {
            let sb = (api.get_global_str_buf_by_handle)(&mut ctx, h)?;
            if let Some(s) = sb.as_str() {
                s.to_string()
            } else {
                "<invalid-str>".to_string()
            }
        } else if k == KIND_TUPLE {
            // Recursively format tuple items
            let len = (api.get_tuple_len_by_handle)(&mut ctx, h)?;
            let mut items: Vec<String> = Vec::with_capacity(len);
            for i in 0..len {
                let ih = (api.get_global_tuple_item_by_handle)(&mut ctx, h, i)?;
                let s = self.format_value_by_handle(ih)?;
                items.push(s);
            }
            format!("({})", items.join(", "))
        } else if k == KIND_KVEC {
            let kv = (api.get_global_kvec_by_handle)(&mut ctx, h)?;
            format!("<kvec kind={} len_bytes={}>", kv.kind, kv.len)
        } else {
            format!("<kind {} @{}>", h.kind, h.index)
        };
        Ok(out)
    }

    /// Convenience: snapshot and format all globals as strings.
    pub fn read_all_globals_as_strings(&mut self) -> Vec<(String, String)> {
        let snapshot = self.snapshot_globals();
        let mut out: Vec<(String, String)> = Vec::with_capacity(snapshot.len());
        for (name, h) in snapshot {
            match self.format_value_by_handle(h) {
                Ok(s) => out.push((name, s)),
                Err(_) => out.push((name, String::from("<error>"))),
            }
        }
        out
    }

    // ---- Registries convenience ----
    pub fn get_function_ptr(&self, name: &str) -> Option<RawFnPtr> {
        self.host.get_function(name).ok()
    }

    pub fn get_type_meta(&self, name: &str) -> Option<TypeMeta> {
        self.host.get_type(name).ok()
    }

    // ---- Plugin loading ----
    pub fn load_plugin_from_path(&mut self, path: &Path) -> Result<(), KaytonError> {
        let lib = unsafe {
            libloading::Library::new(path).map_err(|e| {
                KaytonError::with_source(
                    kayton_api::types::ErrorKind::Generic,
                    "Failed to load plugin DLL",
                    e,
                )
            })?
        };

        unsafe {
            type AbiVersionFn = extern "Rust" fn() -> u32;
            type ManifestFn = extern "Rust" fn() -> &'static [u8];
            type RegisterFn = extern "Rust" fn(ctx: &mut KaytonContext);

            let abi_sym: libloading::Symbol<AbiVersionFn> =
                lib.get(b"kayton_plugin_abi_version").map_err(|e| {
                    KaytonError::with_source(
                        kayton_api::types::ErrorKind::Generic,
                        "Missing kayton_plugin_abi_version",
                        e,
                    )
                })?;
            if (abi_sym)() != kayton_api::KAYTON_PLUGIN_ABI_VERSION {
                return Err(KaytonError::generic("Plugin ABI mismatch"));
            }

            let _manifest_sym: libloading::Symbol<ManifestFn> =
                lib.get(b"kayton_plugin_manifest_json").map_err(|e| {
                    KaytonError::with_source(
                        kayton_api::types::ErrorKind::Generic,
                        "Missing kayton_plugin_manifest_json",
                        e,
                    )
                })?;

            let register_sym: libloading::Symbol<RegisterFn> =
                lib.get(b"kayton_plugin_register").map_err(|e| {
                    KaytonError::with_source(
                        kayton_api::types::ErrorKind::Generic,
                        "Missing kayton_plugin_register",
                        e,
                    )
                })?;

            let mut ctx = self.context();
            (register_sym)(&mut ctx);
        }

        self.plugins.push(lib);
        Ok(())
    }
}
