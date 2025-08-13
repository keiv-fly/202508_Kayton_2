use super::hir_types::*;
use super::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn program1_hir() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    assert_eq!(
        hir,
        vec![
            HirStmt::Assign {
                name: "x".to_string(),
                expr: HirExpr::Int(12),
            },
            HirStmt::Assign {
                name: "x".to_string(),
                expr: HirExpr::Binary {
                    left: Box::new(HirExpr::Ident("x".to_string())),
                    op: HirBinOp::Add,
                    right: Box::new(HirExpr::Int(1)),
                },
            },
            HirStmt::ExprStmt(HirExpr::Call {
                func: Box::new(HirExpr::Ident("print".to_string())),
                args: vec![HirExpr::Ident("x".to_string())],
            }),
        ]
    );
}

#[test]
fn program2_hir() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    assert_eq!(
        hir,
        vec![HirStmt::ExprStmt(HirExpr::Call {
            func: Box::new(HirExpr::Ident("print".to_string())),
            args: vec![HirExpr::Str("Hello, World".to_string())],
        })]
    );
}

#[test]
fn program3_hir() {
    let input = r#"x = 12
print(f"{x}")
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    let hir = lower_program(ast);
    assert_eq!(
        hir,
        vec![
            HirStmt::Assign {
                name: "x".to_string(),
                expr: HirExpr::Int(12),
            },
            HirStmt::ExprStmt(HirExpr::Call {
                func: Box::new(HirExpr::Ident("print".to_string())),
                args: vec![HirExpr::InterpolatedString(vec![
                    HirStringPart::Text("".to_string()),
                    HirStringPart::Expr(Box::new(HirExpr::Ident("x".to_string()))),
                    HirStringPart::Text("".to_string()),
                ])],
            }),
        ]
    );
}
