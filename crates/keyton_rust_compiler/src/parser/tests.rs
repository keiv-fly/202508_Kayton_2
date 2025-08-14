use super::*;
use crate::defs::collect_definitions;
use crate::lexer::Lexer;

#[test]
fn program1_ast() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![
            Stmt::Assign {
                node_id: NodeId(1),
                name: "x".to_string(),
                expr: Expr::Int {
                    node_id: NodeId(2),
                    value: 12
                },
            },
            Stmt::Assign {
                node_id: NodeId(3),
                name: "x".to_string(),
                expr: Expr::Binary {
                    node_id: NodeId(6),
                    left: Box::new(Expr::Ident {
                        node_id: NodeId(4),
                        name: "x".to_string()
                    }),
                    op: BinOp::Add,
                    right: Box::new(Expr::Int {
                        node_id: NodeId(5),
                        value: 1
                    }),
                },
            },
            Stmt::ExprStmt {
                node_id: NodeId(7),
                expr: Expr::Call {
                    node_id: NodeId(10),
                    func: Box::new(Expr::Ident {
                        node_id: NodeId(8),
                        name: "print".to_string()
                    }),
                    args: vec![Expr::Ident {
                        node_id: NodeId(9),
                        name: "x".to_string()
                    }],
                },
            },
        ]
    );

    let defs = collect_definitions(&ast);
    // program1 has two assignments to x, verify both are present
    let x_defs = defs.by_name.get("x").unwrap();
    assert_eq!(x_defs.len(), 2);
    assert_eq!(x_defs[0].node_id, NodeId(1));
    assert_eq!(x_defs[1].node_id, NodeId(3));
    assert!(defs.by_node.contains_key(&NodeId(1)));
    assert!(defs.by_node.contains_key(&NodeId(3)));
}

#[test]
fn program2_ast() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![Stmt::ExprStmt {
            node_id: NodeId(1),
            expr: Expr::Call {
                node_id: NodeId(4),
                func: Box::new(Expr::Ident {
                    node_id: NodeId(2),
                    name: "print".to_string()
                }),
                args: vec![Expr::Str {
                    node_id: NodeId(3),
                    value: "Hello, World".to_string()
                }],
            },
        }]
    );

    let defs = collect_definitions(&ast);
    assert!(defs.all.is_empty());
}

#[test]
fn program3_ast() {
    let input = r#"x = 12
print(f"{x}")
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![
            Stmt::Assign {
                node_id: NodeId(1),
                name: "x".to_string(),
                expr: Expr::Int {
                    node_id: NodeId(2),
                    value: 12
                },
            },
            Stmt::ExprStmt {
                node_id: NodeId(3),
                expr: Expr::Call {
                    node_id: NodeId(10),
                    func: Box::new(Expr::Ident {
                        node_id: NodeId(4),
                        name: "print".to_string()
                    }),
                    args: vec![Expr::InterpolatedString {
                        node_id: NodeId(9),
                        parts: vec![
                            StringPart::Text {
                                node_id: NodeId(5),
                                text: "".to_string()
                            },
                            StringPart::Expr {
                                node_id: NodeId(7),
                                expr: Box::new(Expr::Ident {
                                    node_id: NodeId(6),
                                    name: "x".to_string()
                                })
                            },
                            StringPart::Text {
                                node_id: NodeId(8),
                                text: "".to_string()
                            },
                        ],
                    }],
                },
            },
        ]
    );

    let defs = collect_definitions(&ast);
    let x_defs = defs.by_name.get("x").unwrap();
    assert_eq!(x_defs.len(), 1);
    assert_eq!(x_defs[0].node_id, NodeId(1));
    assert!(defs.by_node.contains_key(&NodeId(1)));
}
