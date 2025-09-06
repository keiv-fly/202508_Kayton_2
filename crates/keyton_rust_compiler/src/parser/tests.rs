use super::*;
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
                name: "x".to_string(),
                expr: Expr::Int(12),
            },
            Stmt::Assign {
                name: "x".to_string(),
                expr: Expr::Binary {
                    left: Box::new(Expr::Ident("x".to_string())),
                    op: BinOp::Add,
                    right: Box::new(Expr::Int(1)),
                },
            },
            Stmt::ExprStmt(Expr::Call {
                func: Box::new(Expr::Ident("print".to_string())),
                args: vec![Expr::Ident("x".to_string())],
            }),
        ]
    );
}

#[test]
fn program4_ast() {
    let input = r#"x = 12
x = "Hello"
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![
            Stmt::Assign {
                name: "x".to_string(),
                expr: Expr::Int(12),
            },
            Stmt::Assign {
                name: "x".to_string(),
                expr: Expr::Str("Hello".to_string()),
            },
            Stmt::ExprStmt(Expr::Call {
                func: Box::new(Expr::Ident("print".to_string())),
                args: vec![Expr::Ident("x".to_string())],
            }),
        ]
    );
}

#[test]
fn program2_ast() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![Stmt::ExprStmt(Expr::Call {
            func: Box::new(Expr::Ident("print".to_string())),
            args: vec![Expr::Str("Hello, World".to_string())],
        })]
    );
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
                name: "x".to_string(),
                expr: Expr::Int(12),
            },
            Stmt::ExprStmt(Expr::Call {
                func: Box::new(Expr::Ident("print".to_string())),
                args: vec![Expr::InterpolatedString(vec![
                    StringPart::Text("".to_string()),
                    StringPart::Expr(Box::new(Expr::Ident("x".to_string()))),
                    StringPart::Text("".to_string()),
                ])],
            }),
        ]
    );
}

#[test]
fn function_with_explicit_return() {
    let input = r#"fn my_sum(x, y):
    return x + y
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![Stmt::FuncDef {
            name: "my_sum".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: vec![Stmt::Return(Expr::Binary {
                left: Box::new(Expr::Ident("x".to_string())),
                op: BinOp::Add,
                right: Box::new(Expr::Ident("y".to_string())),
            })],
        }]
    );
}

#[test]
fn function_with_implicit_return() {
    let input = r#"fn my_sum(x, y):
    x + y
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![Stmt::FuncDef {
            name: "my_sum".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: vec![Stmt::Return(Expr::Binary {
                left: Box::new(Expr::Ident("x".to_string())),
                op: BinOp::Add,
                right: Box::new(Expr::Ident("y".to_string())),
            })],
        }]
    );
}

#[test]
fn let_syntax_ast() {
    let input = r#"let x: i64 = 12
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    let ast = Parser::new(tokens).parse_program();
    assert_eq!(
        ast,
        vec![
            Stmt::Assign {
                name: "x".to_string(),
                expr: Expr::Int(12),
            },
            Stmt::ExprStmt(Expr::Call {
                func: Box::new(Expr::Ident("print".to_string())),
                args: vec![Expr::Ident("x".to_string())],
            }),
        ]
    );
}
