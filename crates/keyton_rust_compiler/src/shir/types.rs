use super::sym::SymbolId;
use crate::hir::hir_types::{HirBinOp, HirId};

#[derive(Debug, Clone, PartialEq)]
pub enum SStmt {
    Assign {
        hir_id: HirId,
        sym: SymbolId,
        expr: SExpr,
    },
    ExprStmt {
        hir_id: HirId,
        expr: SExpr,
    },
    ForRange {
        hir_id: HirId,
        sym: SymbolId,
        start: SExpr,
        end: SExpr,
        body: Vec<SStmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
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
        left: Box<SExpr>,
        op: HirBinOp,
        right: Box<SExpr>,
    },
    Call {
        hir_id: HirId,
        func: Box<SExpr>,
        args: Vec<SExpr>,
    },
    InterpolatedString {
        hir_id: HirId,
        parts: Vec<SStringPart>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SStringPart {
    Text { hir_id: HirId, value: String },
    Expr { hir_id: HirId, expr: SExpr },
}
