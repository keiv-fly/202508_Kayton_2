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
                hir_id: HirId(1),
                name: "x".to_string(),
                expr: HirExpr::Int {
                    hir_id: HirId(2),
                    value: 12
                },
            },
            HirStmt::Assign {
                hir_id: HirId(3),
                name: "x".to_string(),
                expr: HirExpr::Binary {
                    hir_id: HirId(4),
                    left: Box::new(HirExpr::Ident {
                        hir_id: HirId(5),
                        name: "x".to_string()
                    }),
                    op: HirBinOp::Add,
                    right: Box::new(HirExpr::Int {
                        hir_id: HirId(6),
                        value: 1
                    }),
                },
            },
            HirStmt::ExprStmt {
                hir_id: HirId(7),
                expr: HirExpr::Call {
                    hir_id: HirId(8),
                    func: Box::new(HirExpr::Ident {
                        hir_id: HirId(9),
                        name: "print".to_string()
                    }),
                    args: vec![HirExpr::Ident {
                        hir_id: HirId(10),
                        name: "x".to_string()
                    }],
                },
            },
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
        vec![HirStmt::ExprStmt {
            hir_id: HirId(1),
            expr: HirExpr::Call {
                hir_id: HirId(2),
                func: Box::new(HirExpr::Ident {
                    hir_id: HirId(3),
                    name: "print".to_string()
                }),
                args: vec![HirExpr::Str {
                    hir_id: HirId(4),
                    value: "Hello, World".to_string()
                }],
            },
        }]
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
                hir_id: HirId(1),
                name: "x".to_string(),
                expr: HirExpr::Int {
                    hir_id: HirId(2),
                    value: 12
                },
            },
            HirStmt::ExprStmt {
                hir_id: HirId(3),
                expr: HirExpr::Call {
                    hir_id: HirId(4),
                    func: Box::new(HirExpr::Ident {
                        hir_id: HirId(5),
                        name: "print".to_string()
                    }),
                    args: vec![HirExpr::InterpolatedString {
                        hir_id: HirId(6),
                        parts: vec![
                            HirStringPart::Text {
                                hir_id: HirId(7),
                                text: "".to_string()
                            },
                            HirStringPart::Expr {
                                hir_id: HirId(8),
                                expr: Box::new(HirExpr::Ident {
                                    hir_id: HirId(9),
                                    name: "x".to_string(),
                                }),
                            },
                            HirStringPart::Text {
                                hir_id: HirId(10),
                                text: "".to_string()
                            },
                        ],
                    }],
                },
            },
        ]
    );
}
