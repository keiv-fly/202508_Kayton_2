use crate::hir::hir_types::HirStmt;

use super::core::Resolver;
use super::super::sym::{FuncSig, SymKind, Type};
use super::user_funcs::UserFuncDef;

impl Resolver {
    pub fn collect_defs(&mut self, hir: &[HirStmt]) {
        let scope = self.current_scope();
        for stmt in hir {
            match stmt {
                HirStmt::RImportModule { .. } => {}
                HirStmt::RImportItems { .. } => {}
                HirStmt::Assign { name, .. } => {
                    let kind = if scope == self.global_scope() {
                        SymKind::GlobalVar
                    } else {
                        SymKind::LocalVar
                    };
                    self.syms.define(scope, name, kind);
                }
                HirStmt::FuncDef { name, params, body, .. } => {
                    let sid = self.syms.define(scope, name, SymKind::Func);
                    if let Some(info) = self.syms.infos.get_mut(sid.0 as usize) {
                        info.sig = Some(FuncSig {
                            params: vec![Type::Any; params.len()],
                            ret: Type::Any,
                        });
                    }
                    self.user_funcs.insert(
                        sid,
                        UserFuncDef {
                            params: params.clone(),
                            body: body.clone(),
                        },
                    );
                }
                HirStmt::ExprStmt { .. } => {}
                HirStmt::ForRange { var, body, .. } => {
                    self.syms.define(scope, var, SymKind::LocalVar);
                    self.collect_defs(body);
                }
                HirStmt::If { then_branch, else_branch, .. } => {
                    self.collect_defs(then_branch);
                    self.collect_defs(else_branch);
                }
            }
        }
    }
}
