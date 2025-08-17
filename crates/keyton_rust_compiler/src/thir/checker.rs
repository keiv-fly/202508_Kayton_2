use std::collections::HashMap;

use crate::hir::hir_types::{HirBinOp, HirId};
use crate::shir::resolver::ResolvedProgram;
use crate::shir::sym::{ScopeId, SymKind, SymbolId, SymbolTable, Type};
use crate::shir::types::{RExpr, RStmt, RStringPart};

use super::types::{TExpr, TStmt, TStringPart, TypeError, TypeReport, TypedProgram};

pub fn typecheck_program(resolved: &mut ResolvedProgram) -> TypedProgram {
    let mut c = Checker::new(&mut resolved.symbols);
    let thir = resolved.rhir.iter().map(|s| c.check_stmt(s)).collect();
    TypedProgram {
        thir,
        var_types: c.var_types,
        report: TypeReport { errors: c.errors },
    }
}

struct Checker<'a> {
    symbols: &'a mut SymbolTable,
    var_types: HashMap<SymbolId, Type>,
    errors: Vec<TypeError>,
    current_scope: ScopeId,
}

impl<'a> Checker<'a> {
    fn new(symbols: &'a mut SymbolTable) -> Self {
        Self {
            symbols,
            var_types: HashMap::new(),
            errors: Vec::new(),
            current_scope: ScopeId(0),
        }
    }

    fn check_stmt(&mut self, s: &RStmt) -> TStmt {
        match s {
            RStmt::Assign { hir_id, sym, expr } => {
                let texpr = self.check_expr(expr);
                let expr_ty = texpr.ty().clone();

                // Check if this symbol already has a type
                let existing_ty = self.var_types.get(sym).cloned();

                let final_sym = match existing_ty {
                    None => {
                        // First assignment - use the original symbol
                        self.var_types.insert(*sym, expr_ty);
                        *sym
                    }
                    Some(existing_ty) => {
                        if existing_ty == expr_ty {
                            // Same type - reuse the symbol
                            *sym
                        } else {
                            // Different type - create a new symbol for shadowing
                            let var_name = self.symbols.infos[sym.0 as usize].name.clone();
                            let kind = self.symbols.infos[sym.0 as usize].kind;

                            // Create new symbol with same name but different ID
                            let new_sym =
                                self.symbols.define_new(self.current_scope, &var_name, kind);
                            self.var_types.insert(new_sym, expr_ty);
                            new_sym
                        }
                    }
                };

                TStmt::Assign {
                    hir_id: *hir_id,
                    sym: final_sym,
                    expr: texpr,
                }
            }
            RStmt::ExprStmt { hir_id, expr } => {
                let texpr = self.check_expr(expr);
                TStmt::ExprStmt {
                    hir_id: *hir_id,
                    expr: texpr,
                }
            }
        }
    }

    fn check_expr(&mut self, e: &RExpr) -> TExpr {
        match e {
            RExpr::Int { hir_id, value } => TExpr::Int {
                hir_id: *hir_id,
                value: *value,
                ty: Type::Int,
            },
            RExpr::Str { hir_id, value } => TExpr::Str {
                hir_id: *hir_id,
                value: value.clone(),
                ty: Type::Str,
            },
            RExpr::Name { hir_id, sym } => {
                let var_name = &self.symbols.infos[sym.0 as usize].name;
                let kind = self.symbols.infos[sym.0 as usize].kind;

                // Find the most recent shadowed symbol with the same name
                let mut final_sym = *sym;
                for i in (0..self.symbols.infos.len()).rev() {
                    let info = &self.symbols.infos[i];
                    if info.name == *var_name && info.kind == kind {
                        if self.var_types.contains_key(&SymbolId(i as u32)) {
                            final_sym = SymbolId(i as u32);
                            break;
                        }
                    }
                }

                let ty = self.lookup_var_type(*hir_id, final_sym);
                TExpr::Name {
                    hir_id: *hir_id,
                    sym: final_sym,
                    ty,
                }
            }
            RExpr::Binary {
                hir_id,
                left,
                op,
                right,
            } => {
                let l = self.check_expr(left);
                let r = self.check_expr(right);
                let (lhs_ty, rhs_ty) = (l.ty().clone(), r.ty().clone());
                let out_ty = match op {
                    HirBinOp::Add => {
                        // Keep it simple: only Int + Int => Int
                        self.require(*hir_id, Type::Int, lhs_ty.clone());
                        self.require(*hir_id, Type::Int, rhs_ty.clone());
                        Type::Int
                    }
                };
                TExpr::Binary {
                    hir_id: *hir_id,
                    left: Box::new(l),
                    op: op.clone(),
                    right: Box::new(r),
                    ty: out_ty,
                }
            }
            RExpr::Call { hir_id, func, args } => {
                // Extract all needed information before any mutable borrows
                let func_info = Self::extract_func_info(&self.symbols, func);

                // Type subexpressions
                let tf = self.check_expr(func);
                let targs: Vec<_> = args.iter().map(|a| self.check_expr(a)).collect();

                // Check arity and parameters
                if let Some(sig) = &func_info.sig {
                    if sig.params.len() != targs.len() {
                        self.errors.push(TypeError::ArityMismatch {
                            hir_id: *hir_id,
                            expected: sig.params.len(),
                            found: targs.len(),
                        });
                    }
                    for (i, targ) in targs.iter().enumerate() {
                        if let Some(exp) = sig.params.get(i) {
                            self.require(*hir_id, exp.clone(), targ.ty().clone());
                        }
                    }
                } else {
                    // Not a known callable symbol
                    self.errors.push(TypeError::NotCallable {
                        hir_id: *hir_id,
                        callee: func_info.name,
                    });
                }

                TExpr::Call {
                    hir_id: *hir_id,
                    func: Box::new(tf),
                    args: targs,
                    ty: func_info.ret_ty,
                }
            }
            RExpr::InterpolatedString { hir_id, parts } => {
                let parts = parts
                    .iter()
                    .map(|p| match p {
                        RStringPart::Text { hir_id, value } => TStringPart::Text {
                            hir_id: *hir_id,
                            value: value.clone(),
                        },
                        RStringPart::Expr { hir_id, expr } => {
                            let te = self.check_expr(expr);
                            // Allow any type inside interpolation (implicitly to-string).
                            TStringPart::Expr {
                                hir_id: *hir_id,
                                expr: te,
                            }
                        }
                    })
                    .collect();
                TExpr::InterpolatedString {
                    hir_id: *hir_id,
                    parts,
                    ty: Type::Str,
                }
            }
        }
    }

    fn extract_func_info(symbols: &SymbolTable, func: &RExpr) -> FuncInfo {
        match func {
            RExpr::Name { sym, .. } => {
                let info = &symbols.infos[sym.0 as usize];
                let name = info.name.clone();
                let sig = info.sig.clone();
                let ret_ty = sig.as_ref().map(|s| s.ret.clone()).unwrap_or(Type::Any);
                FuncInfo { name, sig, ret_ty }
            }
            _ => FuncInfo {
                name: "<expr>".to_string(),
                sig: None,
                ret_ty: Type::Any,
            },
        }
    }

    fn lookup_var_type(&mut self, hir_id: HirId, sym: SymbolId) -> Type {
        // First check if we have a type for this exact symbol
        if let Some(ty) = self.var_types.get(&sym) {
            return ty.clone();
        }

        // If not found, look for the most recent shadowed symbol with the same name
        let var_name = &self.symbols.infos[sym.0 as usize].name;
        let kind = self.symbols.infos[sym.0 as usize].kind;

        // Look for the most recent symbol with the same name
        for i in (0..self.symbols.infos.len()).rev() {
            let info = &self.symbols.infos[i];
            if info.name == *var_name && info.kind == kind {
                if let Some(ty) = self.var_types.get(&SymbolId(i as u32)) {
                    return ty.clone();
                }
            }
        }

        // Extract symbol info before any mutable operations
        match kind {
            SymKind::BuiltinFunc | SymKind::Func => {
                // As a value, treat functions as Any for now (no first-class function type).
                Type::Any
            }
            _ => {
                // Unknown var type (used before any assignment) ⇒ error but fall back to Any.
                self.errors.push(TypeError::UnknownVarType { hir_id, sym });
                Type::Any
            }
        }
    }

    fn require(&mut self, hir_id: HirId, expected: Type, found: Type) {
        if !self.is_compatible(&expected, &found) {
            self.errors.push(TypeError::TypeMismatch {
                hir_id,
                expected,
                found,
            });
        }
    }

    fn is_compatible(&self, expected: &Type, found: &Type) -> bool {
        expected == found || *expected == Type::Any || *found == Type::Any
    }
}

struct FuncInfo {
    name: String,
    sig: Option<crate::shir::sym::FuncSig>,
    ret_ty: Type,
}

impl TExpr {
    pub fn ty(&self) -> &Type {
        match self {
            TExpr::Int { ty, .. }
            | TExpr::Str { ty, .. }
            | TExpr::Name { ty, .. }
            | TExpr::Binary { ty, .. }
            | TExpr::Call { ty, .. }
            | TExpr::InterpolatedString { ty, .. } => ty,
        }
    }
}
