#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Assign { name: String, expr: HirExpr },
    ExprStmt(HirExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    Int(i64),
    Str(String),
    Ident(String),
    Binary {
        left: Box<HirExpr>,
        op: HirBinOp,
        right: Box<HirExpr>,
    },
    Call {
        func: Box<HirExpr>,
        args: Vec<HirExpr>,
    },
    InterpolatedString(Vec<HirStringPart>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirBinOp {
    Add,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStringPart {
    Text(String),
    Expr(Box<HirExpr>),
}
