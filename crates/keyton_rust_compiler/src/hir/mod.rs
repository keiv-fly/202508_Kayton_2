pub mod hir_types;

use std::collections::HashMap;

use crate::parser::{BinOp, Expr, Stmt, StringPart};
use crate::span::Span;
use hir_types::{HirBinOp, HirExpr, HirId, HirStmt, HirStringPart};

struct LoweringCtx {
    next_id: u32,
    spans: HashMap<HirId, Span>,
}

impl LoweringCtx {
    fn new() -> Self {
        Self {
            next_id: 1,
            spans: HashMap::new(),
        }
    }

    fn new_id(&mut self) -> HirId {
        let id = self.next_id;
        self.next_id += 1;
        let hir_id = HirId(id);
        // Dummy span info for now; real spans not tracked
        let span = Span::new(id as usize, id as usize);
        self.spans.insert(hir_id, span);
        hir_id
    }
}

pub fn lower_program(ast: Vec<Stmt>) -> Vec<HirStmt> {
    lower_program_with_spans(ast).0
}

pub fn lower_program_with_spans(ast: Vec<Stmt>) -> (Vec<HirStmt>, HashMap<HirId, Span>) {
    let mut ctx = LoweringCtx::new();
    let hir = ast.into_iter().map(|s| lower_stmt(&mut ctx, s)).collect();
    (hir, ctx.spans)
}

fn lower_stmt(ctx: &mut LoweringCtx, stmt: Stmt) -> HirStmt {
    match stmt {
        Stmt::Assign { name, expr } => HirStmt::Assign {
            hir_id: ctx.new_id(),
            name,
            expr: lower_expr(ctx, expr),
        },
        Stmt::ForRange { var, start, end, body } => HirStmt::ForRange {
            hir_id: ctx.new_id(),
            var,
            start: lower_expr(ctx, start),
            end: lower_expr(ctx, end),
            body: body.into_iter().map(|s| lower_stmt(ctx, s)).collect(),
        },
        Stmt::If {
            cond,
            then_branch,
            else_branch,
        } => HirStmt::If {
            hir_id: ctx.new_id(),
            cond: lower_expr(ctx, cond),
            then_branch: then_branch
                .into_iter()
                .map(|s| lower_stmt(ctx, s))
                .collect(),
            else_branch: else_branch
                .into_iter()
                .map(|s| lower_stmt(ctx, s))
                .collect(),
        },
        Stmt::ExprStmt(expr) => HirStmt::ExprStmt {
            hir_id: ctx.new_id(),
            expr: lower_expr(ctx, expr),
        },
        Stmt::FuncDef { name, params, body } => HirStmt::FuncDef {
            hir_id: ctx.new_id(),
            name,
            params,
            body: body.into_iter().map(|s| lower_stmt(ctx, s)).collect(),
        },
        Stmt::Return(expr) => HirStmt::ExprStmt {
            // For now, returns are not first-class; treat as expression stmt in HIR
            hir_id: ctx.new_id(),
            expr: lower_expr(ctx, expr),
        },
    }
}

fn lower_expr(ctx: &mut LoweringCtx, expr: Expr) -> HirExpr {
    match expr {
        Expr::Int(n) => HirExpr::Int {
            hir_id: ctx.new_id(),
            value: n,
        },
        Expr::Str(s) => HirExpr::Str {
            hir_id: ctx.new_id(),
            value: s,
        },
        Expr::Bool(b) => HirExpr::Bool {
            hir_id: ctx.new_id(),
            value: b,
        },
        Expr::Ident(s) => HirExpr::Ident {
            hir_id: ctx.new_id(),
            name: s,
        },
        Expr::Binary { left, op, right } => HirExpr::Binary {
            hir_id: ctx.new_id(),
            left: Box::new(lower_expr(ctx, *left)),
            op: lower_bin_op(op),
            right: Box::new(lower_expr(ctx, *right)),
        },
        Expr::Call { func, args } => HirExpr::Call {
            hir_id: ctx.new_id(),
            func: Box::new(lower_expr(ctx, *func)),
            args: args.into_iter().map(|a| lower_expr(ctx, a)).collect(),
        },
        Expr::InterpolatedString(parts) => HirExpr::InterpolatedString {
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
        StringPart::Text(t) => HirStringPart::Text {
            hir_id: ctx.new_id(),
            text: t,
        },
        StringPart::Expr(e) => HirStringPart::Expr {
            hir_id: ctx.new_id(),
            expr: Box::new(lower_expr(ctx, *e)),
        },
    }
}

#[cfg(test)]
mod tests;
