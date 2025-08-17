use crate::hir::hir_types::{HirBinOp, HirId};
use crate::rhir::sym::{SymbolId, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum TStmt {
    Assign {
        hir_id: HirId,
        sym: SymbolId,
        expr: TExpr,
    },
    ExprStmt {
        hir_id: HirId,
        expr: TExpr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TExpr {
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
        left: Box<TExpr>,
        op: HirBinOp,
        right: Box<TExpr>,
        ty: Type,
    },
    Call {
        hir_id: HirId,
        func: Box<TExpr>,
        args: Vec<TExpr>,
        ty: Type,
    },
    InterpolatedString {
        hir_id: HirId,
        parts: Vec<TStringPart>,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TStringPart {
    Text { hir_id: HirId, value: String },
    Expr { hir_id: HirId, expr: TExpr },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    TypeMismatch {
        hir_id: HirId,
        expected: Type,
        found: Type,
    },
    NotCallable {
        hir_id: HirId,
        callee: String,
    },
    ArityMismatch {
        hir_id: HirId,
        expected: usize,
        found: usize,
    },
    UnknownVarType {
        hir_id: HirId,
        sym: SymbolId,
    },
}

#[derive(Debug, Default)]
pub struct TypeReport {
    pub errors: Vec<TypeError>,
}

#[derive(Debug)]
pub struct TypedProgram {
    pub shir: Vec<TStmt>,
    // Snapshot of inferred var types (for debugging/consumers); expressions carry their own types.
    pub var_types: std::collections::HashMap<SymbolId, Type>,
    pub report: TypeReport,
}
