pub mod hir_types;

use crate::parser::{BinOp, Expr, Stmt, StringPart};
use hir_types::{HirBinOp, HirExpr, HirId, HirStmt, HirStringPart};

struct LoweringCtx {
    next_id: u32,
}

impl LoweringCtx {
    fn new() -> Self {
        Self { next_id: 1 }
    }

    fn new_id(&mut self) -> HirId {
        let id = self.next_id;
        self.next_id += 1;
        HirId(id)
    }
}

pub fn lower_program(ast: Vec<Stmt>) -> Vec<HirStmt> {
    let mut ctx = LoweringCtx::new();
    ast.into_iter().map(|s| lower_stmt(&mut ctx, s)).collect()
}

fn lower_stmt(ctx: &mut LoweringCtx, stmt: Stmt) -> HirStmt {
    match stmt {
        Stmt::Assign {
            node_id: _,
            name,
            expr,
        } => HirStmt::Assign {
            hir_id: ctx.new_id(),
            name,
            expr: lower_expr(ctx, expr),
        },
        Stmt::ExprStmt { node_id: _, expr } => HirStmt::ExprStmt {
            hir_id: ctx.new_id(),
            expr: lower_expr(ctx, expr),
        },
    }
}

fn lower_expr(ctx: &mut LoweringCtx, expr: Expr) -> HirExpr {
    match expr {
        Expr::Int {
            node_id: _,
            value: n,
        } => HirExpr::Int {
            hir_id: ctx.new_id(),
            value: n,
        },
        Expr::Str {
            node_id: _,
            value: s,
        } => HirExpr::Str {
            hir_id: ctx.new_id(),
            value: s,
        },
        Expr::Ident {
            node_id: _,
            name: s,
        } => HirExpr::Ident {
            hir_id: ctx.new_id(),
            name: s,
        },
        Expr::Binary {
            node_id: _,
            left,
            op,
            right,
        } => HirExpr::Binary {
            hir_id: ctx.new_id(),
            left: Box::new(lower_expr(ctx, *left)),
            op: lower_bin_op(op),
            right: Box::new(lower_expr(ctx, *right)),
        },
        Expr::Call {
            node_id: _,
            func,
            args,
        } => HirExpr::Call {
            hir_id: ctx.new_id(),
            func: Box::new(lower_expr(ctx, *func)),
            args: args.into_iter().map(|a| lower_expr(ctx, a)).collect(),
        },
        Expr::InterpolatedString { node_id: _, parts } => HirExpr::InterpolatedString {
            hir_id: ctx.new_id(),
            parts: parts
                .into_iter()
                .map(|p| lower_string_part(ctx, p))
                .collect(),
        },
    }
}

fn lower_bin_op(op: BinOp) -> HirBinOp {
    match op {
        BinOp::Add => HirBinOp::Add,
    }
}

fn lower_string_part(ctx: &mut LoweringCtx, part: StringPart) -> HirStringPart {
    match part {
        StringPart::Text {
            node_id: _,
            text: t,
        } => HirStringPart::Text {
            hir_id: ctx.new_id(),
            text: t,
        },
        StringPart::Expr {
            node_id: _,
            expr: e,
        } => HirStringPart::Expr {
            hir_id: ctx.new_id(),
            expr: Box::new(lower_expr(ctx, *e)),
        },
    }
}

#[cfg(test)]
mod tests;
