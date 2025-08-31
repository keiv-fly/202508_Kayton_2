use std::collections::HashMap;

use crate::hir::hir_types::{HirExpr, HirId, HirStmt, HirStringPart};
use crate::span::Span;

use super::sym::{FuncSig, ScopeId, SymKind, SymbolId, SymbolTable, Type};
use super::types::{SExpr, SStmt, SStringPart};

#[derive(Debug, Clone)]
pub enum ResolveError {
    UnresolvedName { span: Span, name: String },
}

#[derive(Debug, Default)]
pub struct ResolveReport {
    pub errors: Vec<ResolveError>,
}

pub struct Resolver {
    pub syms: SymbolTable,
    pub report: ResolveReport,
    scope_stack: Vec<ScopeId>,
    builtins: HashMap<String, SymbolId>,
    spans: HashMap<HirId, Span>,
    user_funcs: HashMap<SymbolId, UserFuncDef>,
}

impl Resolver {
    pub fn new(spans: HashMap<HirId, Span>) -> Self {
        let (syms, global) = SymbolTable::new();
        Self {
            syms,
            report: ResolveReport::default(),
            scope_stack: vec![global],
            builtins: HashMap::new(),
            spans,
            user_funcs: HashMap::new(),
        }
    }

    pub fn add_builtin(&mut self, name: &str) -> SymbolId {
        let g = self.global_scope();
        let sid = self.syms.define(g, name, SymKind::BuiltinFunc);
        // Seed a simple signature for builtins like print(name: Any) -> Unit
        if let Some(info) = self.syms.infos.get_mut(sid.0 as usize) {
            info.sig = Some(FuncSig {
                params: vec![Type::Any],
                ret: Type::Unit,
            });
        }
        self.builtins.insert(name.to_string(), sid);
        sid
    }

    fn current_scope(&self) -> ScopeId {
        *self.scope_stack.last().unwrap()
    }
    fn global_scope(&self) -> ScopeId {
        ScopeId(0)
    }

    #[allow(dead_code)]
    fn enter_scope(&mut self) -> ScopeId {
        let parent = self.current_scope();
        let s = self.syms.new_scope(parent);
        self.scope_stack.push(s);
        s
    }

    #[allow(dead_code)]
    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn collect_defs(&mut self, hir: &[HirStmt]) {
        let scope = self.current_scope();
        for stmt in hir {
            match stmt {
                HirStmt::Assign { name, .. } => {
                    let kind = if scope == self.global_scope() {
                        SymKind::GlobalVar
                    } else {
                        SymKind::LocalVar
                    };
                    self.syms.define(scope, name, kind);
                }
                HirStmt::FuncDef {
                    name, params, body, ..
                } => {
                    // Define function symbol in current scope
                    let sid = self.syms.define(scope, name, SymKind::Func);
                    // Seed unknown signature for now (Any params/ret inferred at call sites)
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
            }
        }
    }

    pub fn resolve_program(&mut self, hir: &[HirStmt]) -> Vec<SStmt> {
        self.collect_defs(hir);
        let mut out = Vec::with_capacity(hir.len());
        for stmt in hir {
            out.push(self.resolve_stmt(stmt));
        }
        out
    }

    fn resolve_stmt(&mut self, s: &HirStmt) -> SStmt {
        match s {
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
            HirStmt::FuncDef { .. } => {
                // Function bodies are not yet modeled in SHIR; skip to a no-op expr stmt Unit
                SStmt::ExprStmt {
                    hir_id: HirId(0),
                    expr: SExpr::Int {
                        hir_id: HirId(0),
                        value: 0,
                    },
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

    fn resolve_expr(&mut self, e: &HirExpr) -> SExpr {
        match e {
            HirExpr::Int { hir_id, value } => SExpr::Int {
                hir_id: *hir_id,
                value: *value,
            },
            HirExpr::Str { hir_id, value } => SExpr::Str {
                hir_id: *hir_id,
                value: value.clone(),
            },
            HirExpr::Ident { hir_id, name } => {
                let sym = self.lookup_name(*hir_id, name);
                SExpr::Name {
                    hir_id: *hir_id,
                    sym,
                }
            }
            HirExpr::Binary {
                hir_id,
                left,
                op,
                right,
            } => {
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
                            let inlined = Self::substitute_params(&fdef.params, args, &body_expr);
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

    fn lookup_name(&mut self, use_hir: HirId, name: &str) -> SymbolId {
        if let Some(sid) = self.syms.lookup(self.current_scope(), name) {
            return sid;
        }
        if let Some(&sid) = self.builtins.get(name) {
            return sid;
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

pub struct ResolvedProgram {
    pub shir: Vec<SStmt>,
    pub symbols: SymbolTable,
}

pub fn resolve_program(hir: &[HirStmt]) -> ResolvedProgram {
    resolve_program_with_spans(hir, HashMap::new())
}

pub fn resolve_program_with_spans(hir: &[HirStmt], spans: HashMap<HirId, Span>) -> ResolvedProgram {
    let mut resolver = Resolver::new(spans);
    resolver.add_builtin("print");
    let shir = resolver.resolve_program(hir);
    ResolvedProgram {
        shir,
        symbols: resolver.syms,
    }
}

#[derive(Clone)]
struct UserFuncDef {
    params: Vec<String>,
    body: Vec<crate::hir::hir_types::HirStmt>,
}

impl Resolver {
    fn last_expr_of_body(
        body: &[crate::hir::hir_types::HirStmt],
    ) -> Option<crate::hir::hir_types::HirExpr> {
        use crate::hir::hir_types::HirStmt as HS;
        body.iter().rev().find_map(|s| match s {
            HS::ExprStmt { expr, .. } => Some(expr.clone()),
            _ => None,
        })
    }

    fn substitute_params(
        params: &[String],
        args: &[crate::hir::hir_types::HirExpr],
        expr: &crate::hir::hir_types::HirExpr,
    ) -> crate::hir::hir_types::HirExpr {
        use crate::hir::hir_types::{HirExpr as HE, HirStringPart};
        let mut mapping: std::collections::HashMap<&str, &crate::hir::hir_types::HirExpr> =
            std::collections::HashMap::new();
        for (i, p) in params.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                mapping.insert(p.as_str(), arg);
            }
        }
        fn subst<'a>(e: &HE, map: &std::collections::HashMap<&'a str, &'a HE>) -> HE {
            match e {
                HE::Int { hir_id, value } => HE::Int {
                    hir_id: *hir_id,
                    value: *value,
                },
                HE::Str { hir_id, value } => HE::Str {
                    hir_id: *hir_id,
                    value: value.clone(),
                },
                HE::Ident { hir_id, name } => {
                    if let Some(repl) = map.get(name.as_str()) {
                        (*repl).clone()
                    } else {
                        HE::Ident {
                            hir_id: *hir_id,
                            name: name.clone(),
                        }
                    }
                }
                HE::Binary {
                    hir_id,
                    left,
                    op,
                    right,
                } => HE::Binary {
                    hir_id: *hir_id,
                    left: Box::new(subst(left, map)),
                    op: op.clone(),
                    right: Box::new(subst(right, map)),
                },
                HE::Call { hir_id, func, args } => HE::Call {
                    hir_id: *hir_id,
                    func: Box::new(subst(func, map)),
                    args: args.iter().map(|a| subst(a, map)).collect(),
                },
                HE::InterpolatedString { hir_id, parts } => HE::InterpolatedString {
                    hir_id: *hir_id,
                    parts: parts
                        .iter()
                        .map(|p| match p {
                            HirStringPart::Text { hir_id, text } => HirStringPart::Text {
                                hir_id: *hir_id,
                                text: text.clone(),
                            },
                            HirStringPart::Expr { hir_id, expr } => HirStringPart::Expr {
                                hir_id: *hir_id,
                                expr: Box::new(subst(expr, map)),
                            },
                        })
                        .collect(),
                },
            }
        }
        subst(expr, &mapping)
    }
}
