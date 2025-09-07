use crate::hir::hir_types::{HirExpr, HirStmt};

use super::core::Resolver;

#[derive(Clone)]
pub(super) struct UserFuncDef {
    pub params: Vec<String>,
    pub body: Vec<HirStmt>,
}

impl Resolver {
    pub(crate) fn last_expr_of_body(body: &[HirStmt]) -> Option<HirExpr> {
        use crate::hir::hir_types::HirStmt as HS;
        body.iter().rev().find_map(|s| match s {
            HS::ExprStmt { expr, .. } => Some(expr.clone()),
            _ => None,
        })
    }

    pub(crate) fn substitute_params(
        params: &[String],
        args: &[HirExpr],
        expr: &HirExpr,
    ) -> HirExpr {
        use crate::hir::hir_types::{HirExpr as HE, HirStringPart};
        let mut mapping: std::collections::HashMap<&str, &HirExpr> =
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
                HE::Bool { hir_id, value } => HE::Bool {
                    hir_id: *hir_id,
                    value: *value,
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
