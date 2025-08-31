use crate::hir::hir_types::{HirBinOp, HirId};
use crate::shir::sym::{SymbolId, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum RStmt {
    Assign {
        hir_id: HirId,
        sym: SymbolId,
        expr: RExpr,
    },
    ExprStmt {
        hir_id: HirId,
        expr: RExpr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RExpr {
    Int {
        hir_id: HirId,
        value: i64,
        ty: Type,
    },
    Str {
        hir_id: HirId,
        value: String,
        ty: Type,
    },
    Name {
        hir_id: HirId,
        sym: SymbolId,
        ty: Type,
    },
    Binary {
        hir_id: HirId,
        left: Box<RExpr>,
        op: HirBinOp,
        right: Box<RExpr>,
        ty: Type,
    },
    Call {
        hir_id: HirId,
        func: Box<RExpr>,
        args: Vec<RExpr>,
        ty: Type,
    },
    MacroCall {
        hir_id: HirId,
        macro_name: String,
        args: Vec<RExpr>,
        ty: Type,
    },
    InterpolatedString {
        hir_id: HirId,
        parts: Vec<RStringPart>,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RStringPart {
    Text { hir_id: HirId, value: String },
    Expr { hir_id: HirId, expr: RExpr },
}

#[derive(Debug)]
pub struct RustProgram {
    pub rhir: Vec<RStmt>,
    pub var_types: std::collections::HashMap<SymbolId, Type>,
}

impl RExpr {
    pub fn ty(&self) -> &Type {
        match self {
            RExpr::Int { ty, .. }
            | RExpr::Str { ty, .. }
            | RExpr::Name { ty, .. }
            | RExpr::Binary { ty, .. }
            | RExpr::Call { ty, .. }
            | RExpr::MacroCall { ty, .. }
            | RExpr::InterpolatedString { ty, .. } => ty,
        }
    }
}
