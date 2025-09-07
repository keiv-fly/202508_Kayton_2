use crate::hir::hir_types::{HirId, HirStmt};

use super::core::Resolver;
use super::errors::ResolveError;
use super::super::sym::SymKind;
use super::super::types::{SExpr, SStmt};

impl Resolver {
    pub(super) fn resolve_stmt(&mut self, s: &HirStmt) -> SStmt {
        match s {
            HirStmt::RImportModule { hir_id, module } => {
                SStmt::RImportModule {
                    hir_id: *hir_id,
                    module: module.clone(),
                }
            }
            HirStmt::RImportItems { hir_id, module, items } => SStmt::RImportItems {
                hir_id: *hir_id,
                module: module.clone(),
                items: items.clone(),
            },
            HirStmt::Assign { hir_id, name, expr } => {
                let scope = self.current_scope();
                let sym = self
                    .syms
                    .lookup(scope, name)
                    .or_else(|| self.builtins.get(name).copied())
                    .unwrap_or_else(|| {
                        let kind = if scope == self.global_scope() {
                            SymKind::GlobalVar
                        } else {
                            SymKind::LocalVar
                        };
                        let sid = self.syms.define(scope, name, kind);
                        let span = self.spans.get(hir_id).cloned().unwrap_or_default();
                        self.report.errors.push(ResolveError::UnresolvedName {
                            span,
                            name: name.clone(),
                        });
                        sid
                    });
                let rexpr = self.resolve_expr(expr);
                SStmt::Assign {
                    hir_id: *hir_id,
                    sym,
                    expr: rexpr,
                }
            }
            HirStmt::FuncDef { .. } => SStmt::ExprStmt {
                hir_id: HirId(0),
                expr: SExpr::Int {
                    hir_id: HirId(0),
                    value: 0,
                },
            },
            HirStmt::ForRange {
                hir_id,
                var,
                start,
                end,
                body,
            } => {
                let scope = self.current_scope();
                let sym = self
                    .syms
                    .lookup(scope, var)
                    .unwrap_or_else(|| self.syms.define(scope, var, SymKind::LocalVar));
                let s = self.resolve_expr(start);
                let e = self.resolve_expr(end);
                let body_resolved = body.iter().map(|st| self.resolve_stmt(st)).collect();
                SStmt::ForRange {
                    hir_id: *hir_id,
                    sym,
                    start: s,
                    end: e,
                    body: body_resolved,
                }
            }
            HirStmt::If {
                hir_id,
                cond,
                then_branch,
                else_branch,
            } => {
                let c = self.resolve_expr(cond);
                let then_r = then_branch.iter().map(|st| self.resolve_stmt(st)).collect();
                let else_r = else_branch.iter().map(|st| self.resolve_stmt(st)).collect();
                SStmt::If {
                    hir_id: *hir_id,
                    cond: c,
                    then_branch: then_r,
                    else_branch: else_r,
                }
            }
            HirStmt::ExprStmt { hir_id, expr } => {
                let rexpr = self.resolve_expr(expr);
                SStmt::ExprStmt {
                    hir_id: *hir_id,
                    expr: rexpr,
                }
            }
        }
    }
}
