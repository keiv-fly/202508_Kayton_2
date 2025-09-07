use std::collections::HashMap;

use crate::hir::hir_types::{HirId, HirStmt};
use crate::rimport::env::{load_active_env_registry, load_plugin_manifest};
use crate::span::Span;

use super::core::Resolver;
use super::errors::ResolveError;
use super::super::sym::{FuncSig, SymKind, SymbolTable, Type};
use super::super::types::SStmt;

impl Resolver {
    pub fn resolve_program(&mut self, hir: &[HirStmt]) -> Vec<SStmt> {
        self.collect_defs(hir);
        if let Ok(env_dir) = load_active_env_registry() {
            let _ = env_dir;
        }
        let mut out = Vec::with_capacity(hir.len());
        for stmt in hir {
            if let HirStmt::RImportItems { module, items, .. } = stmt {
                let span = self.spans.get(&HirId(0)).cloned().unwrap_or_default();
                match load_plugin_manifest(module) {
                    Ok(mani) => {
                        self.plugin_manifests.insert(module.clone(), mani.clone());
                        for it in items {
                            if let Some(func) = mani.functions.iter().find(|f| &f.stable_name == it) {
                                let g = self.global_scope();
                                let sid = self.syms.define(g, &it, SymKind::BuiltinFunc);
                                if let Some(info) = self.syms.infos.get_mut(sid.0 as usize) {
                                    let params = func.sig.params.iter().map(map_typekind).collect();
                                    info.sig = Some(FuncSig {
                                        params,
                                        ret: map_typekind(&func.sig.ret),
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.report.errors.push(ResolveError::ImportError {
                            span,
                            message: format!(
                                "ImportError: Library '{}' is not installed or invalid. Run: kik rinstall {}. Details: {}",
                                module, module, e
                            ),
                        });
                    }
                }
            } else if let HirStmt::RImportModule { module, .. } = stmt {
                let span = self.spans.get(&HirId(0)).cloned().unwrap_or_default();
                match load_plugin_manifest(module) {
                    Ok(mani) => {
                        self.plugin_manifests.insert(module.clone(), mani.clone());
                    }
                    Err(e) => {
                        self.report.errors.push(ResolveError::ImportError {
                            span,
                            message: format!(
                                "ImportError: Library '{}' is not installed or invalid. Run: kik rinstall {}. Details: {}",
                                module, module, e
                            ),
                        });
                    }
                }
            }
            out.push(self.resolve_stmt(stmt));
        }
        out
    }
}

fn map_typekind(k: &kayton_plugin_sdk::manifest::TypeKind) -> Type {
    use kayton_plugin_sdk::manifest::TypeKind as TK;
    match k {
        TK::I64 => Type::I64,
        TK::F64 => Type::Any,
        TK::U64 => Type::Any,
        TK::Bool => Type::Any,
        TK::StaticStr | TK::StringBuf => Type::Str,
        TK::VecI64 | TK::VecF64 | TK::Dynamic | TK::Unit => Type::Any,
    }
}

pub struct ResolvedProgram {
    pub shir: Vec<SStmt>,
    pub symbols: SymbolTable,
    pub plugins: HashMap<String, kayton_plugin_sdk::manifest::Manifest>,
}

pub fn resolve_program(hir: &[HirStmt]) -> ResolvedProgram {
    resolve_program_with_spans(hir, HashMap::new())
}

pub fn resolve_program_with_spans(
    hir: &[HirStmt],
    spans: HashMap<HirId, Span>,
) -> ResolvedProgram {
    let mut resolver = Resolver::new(spans);
    resolver.add_builtin("print");
    let shir = resolver.resolve_program(hir);
    ResolvedProgram {
        shir,
        symbols: resolver.syms,
        plugins: resolver.plugin_manifests,
    }
}
