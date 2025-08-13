pub mod hir_types;

use crate::parser::{BinOp, Expr, Stmt, StringPart};
use hir_types::{HirBinOp, HirExpr, HirStmt, HirStringPart};

pub fn lower_program(ast: Vec<Stmt>) -> Vec<HirStmt> {
    ast.into_iter().map(lower_stmt).collect()
}

pub fn lower_stmt(stmt: Stmt) -> HirStmt {
    match stmt {
        Stmt::Assign { name, expr } => HirStmt::Assign {
            name,
            expr: lower_expr(expr),
        },
        Stmt::ExprStmt(expr) => HirStmt::ExprStmt(lower_expr(expr)),
    }
}

pub fn lower_expr(expr: Expr) -> HirExpr {
    match expr {
        Expr::Int(n) => HirExpr::Int(n),
        Expr::Str(s) => HirExpr::Str(s),
        Expr::Ident(s) => HirExpr::Ident(s),
        Expr::Binary { left, op, right } => HirExpr::Binary {
            left: Box::new(lower_expr(*left)),
            op: lower_bin_op(op),
            right: Box::new(lower_expr(*right)),
        },
        Expr::Call { func, args } => HirExpr::Call {
            func: Box::new(lower_expr(*func)),
            args: args.into_iter().map(lower_expr).collect(),
        },
        Expr::InterpolatedString(parts) => {
            HirExpr::InterpolatedString(parts.into_iter().map(lower_string_part).collect())
        }
    }
}

fn lower_bin_op(op: BinOp) -> HirBinOp {
    match op {
        BinOp::Add => HirBinOp::Add,
    }
}

fn lower_string_part(part: StringPart) -> HirStringPart {
    match part {
        StringPart::Text(t) => HirStringPart::Text(t),
        StringPart::Expr(e) => HirStringPart::Expr(Box::new(lower_expr(*e))),
    }
}

#[cfg(test)]
mod tests;
