use crate::hir::hir_types::{HirExpr, HirId, HirStringPart};

use super::core::Resolver;
use super::errors::ResolveError;
use super::super::sym::{FuncSig, SymKind, SymbolId, Type};
use super::super::types::{SExpr, SStringPart};

impl Resolver {
    pub(super) fn resolve_expr(&mut self, e: &HirExpr) -> SExpr {
        match e {
            HirExpr::Int { hir_id, value } => SExpr::Int {
                hir_id: *hir_id,
                value: *value,
            },
            HirExpr::Str { hir_id, value } => SExpr::Str {
                hir_id: *hir_id,
                value: value.clone(),
            },
            HirExpr::Bool { hir_id, value } => SExpr::Bool {
                hir_id: *hir_id,
                value: *value,
            },
            HirExpr::Ident { hir_id, name } => {
                let sym = self.lookup_name(*hir_id, name);
                SExpr::Name {
                    hir_id: *hir_id,
                    sym,
                }
            }
            HirExpr::Binary { hir_id, left, op, right } => {
                let l = self.resolve_expr(left);
                let r = self.resolve_expr(right);
                SExpr::Binary {
                    hir_id: *hir_id,
                    left: Box::new(l),
                    op: op.clone(),
                    right: Box::new(r),
                }
            }
            HirExpr::Call { hir_id, func, args } => {
                if let HirExpr::Ident { name, .. } = func.as_ref() {
                    let sym = self.lookup_name(*hir_id, name);
                    if let Some(fdef) = self.user_funcs.get(&sym) {
                        if let Some(body_expr) = Self::last_expr_of_body(&fdef.body) {
                            let inlined =
                                Self::substitute_params(&fdef.params, args, &body_expr);
                            return self.resolve_expr(&inlined);
                        }
                    }
                }
                let f = self.resolve_expr(func);
                let a = args.iter().map(|x| self.resolve_expr(x)).collect();
                SExpr::Call {
                    hir_id: *hir_id,
                    func: Box::new(f),
                    args: a,
                }
            }
            HirExpr::InterpolatedString { hir_id, parts } => {
                let parts = parts
                    .iter()
                    .map(|p| match p {
                        HirStringPart::Text { hir_id, text } => SStringPart::Text {
                            hir_id: *hir_id,
                            value: text.clone(),
                        },
                        HirStringPart::Expr { hir_id, expr } => SStringPart::Expr {
                            hir_id: *hir_id,
                            expr: self.resolve_expr(expr),
                        },
                    })
                    .collect();
                SExpr::InterpolatedString {
                    hir_id: *hir_id,
                    parts,
                }
            }
        }
    }

    pub(super) fn lookup_name(&mut self, use_hir: HirId, name: &str) -> SymbolId {
        if let Some(sid) = self.syms.lookup(self.current_scope(), name) {
            return sid;
        }
        if let Some(&sid) = self.builtins.get(name) {
            return sid;
        }
        match name {
            "vec" => {
                let sid = self.add_builtin("vec");
                if let Some(info) = self.syms.infos.get_mut(sid.0 as usize) {
                    info.sig = Some(FuncSig {
                        params: vec![],
                        ret: Type::Any,
                    });
                }
                return sid;
            }
            "append" => {
                let sid = self.add_builtin("append");
                if let Some(info) = self.syms.infos.get_mut(sid.0 as usize) {
                    info.sig = Some(FuncSig {
                        params: vec![Type::Any, Type::Any],
                        ret: Type::Unit,
                    });
                }
                return sid;
            }
            "sum" => {
                let sid = self.add_builtin("sum");
                if let Some(info) = self.syms.infos.get_mut(sid.0 as usize) {
                    info.sig = Some(FuncSig {
                        params: vec![Type::Any],
                        ret: Type::I64,
                    });
                }
                return sid;
            }
            _ => {}
        }
        let sid = self
            .syms
            .define(self.global_scope(), name, SymKind::GlobalVar);
        let span = self.spans.get(&use_hir).cloned().unwrap_or_default();
        self.report.errors.push(ResolveError::UnresolvedName {
            span,
            name: name.to_string(),
        });
        sid
    }
}
