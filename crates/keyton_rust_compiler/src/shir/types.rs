use super::sym::SymbolId;
use crate::hir::hir_types::{HirBinOp, HirId};

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
    },
    Str {
        hir_id: HirId,
        value: String,
    },
    Name {
        hir_id: HirId,
        sym: SymbolId,
    },
    Binary {
        hir_id: HirId,
        left: Box<RExpr>,
        op: HirBinOp,
        right: Box<RExpr>,
    },
    Call {
        hir_id: HirId,
        func: Box<RExpr>,
        args: Vec<RExpr>,
    },
    InterpolatedString {
        hir_id: HirId,
        parts: Vec<RStringPart>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RStringPart {
    Text { hir_id: HirId, value: String },
    Expr { hir_id: HirId, expr: RExpr },
}
