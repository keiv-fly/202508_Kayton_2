#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct HirId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Assign {
        hir_id: HirId,
        name: String,
        expr: HirExpr,
    },
    ExprStmt {
        hir_id: HirId,
        expr: HirExpr,
    },
    ForRange {
        hir_id: HirId,
        var: String,
        start: HirExpr,
        end: HirExpr,
        body: Vec<HirStmt>,
    },
    FuncDef {
        hir_id: HirId,
        name: String,
        params: Vec<String>,
        body: Vec<HirStmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    Int {
        hir_id: HirId,
        value: i64,
    },
    Str {
        hir_id: HirId,
        value: String,
    },
    Ident {
        hir_id: HirId,
        name: String,
    },
    Binary {
        hir_id: HirId,
        left: Box<HirExpr>,
        op: HirBinOp,
        right: Box<HirExpr>,
    },
    Call {
        hir_id: HirId,
        func: Box<HirExpr>,
        args: Vec<HirExpr>,
    },
    InterpolatedString {
        hir_id: HirId,
        parts: Vec<HirStringPart>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirBinOp {
    Add,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStringPart {
    Text { hir_id: HirId, text: String },
    Expr { hir_id: HirId, expr: Box<HirExpr> },
}
