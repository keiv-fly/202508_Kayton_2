use super::*;

#[test]
fn program1_tokens() {
    let input = r#"x = 12
x = x + 1
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    assert_eq!(
        tokens,
        vec![
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Int(12),
            Token::Newline,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Int(1),
            Token::Newline,
            Token::Ident("print".to_string()),
            Token::LParen,
            Token::Ident("x".to_string()),
            Token::RParen,
            Token::Newline,
            Token::EOF,
        ]
    );
}

#[test]
fn rimport_tokens() {
    let input = "rimport math";
    let tokens = Lexer::new(input).tokenize();
    assert_eq!(tokens, vec![Token::RimportKw, Token::Ident("math".to_string()), Token::EOF]);
}

#[test]
fn from_rimport_tokens() {
    let input = "from math rimport add, sub";
    let tokens = Lexer::new(input).tokenize();
    assert_eq!(
        tokens,
        vec![
            Token::FromKw,
            Token::Ident("math".to_string()),
            Token::RimportKw,
            Token::Ident("add".to_string()),
            Token::Comma,
            Token::Ident("sub".to_string()),
            Token::EOF,
        ]
    );
}

#[test]
fn program4_tokens() {
    let input = r#"x = 12
x = "Hello"
print(x)
"#;
    let tokens = Lexer::new(input).tokenize();
    assert_eq!(
        tokens,
        vec![
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Int(12),
            Token::Newline,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Str("Hello".to_string()),
            Token::Newline,
            Token::Ident("print".to_string()),
            Token::LParen,
            Token::Ident("x".to_string()),
            Token::RParen,
            Token::Newline,
            Token::EOF,
        ]
    );
}

#[test]
fn program2_tokens() {
    let input = r#"print("Hello, World")"#;
    let tokens = Lexer::new(input).tokenize();
    assert_eq!(
        tokens,
        vec![
            Token::Ident("print".to_string()),
            Token::LParen,
            Token::Str("Hello, World".to_string()),
            Token::RParen,
            Token::EOF,
        ]
    );
}

#[test]
fn program3_tokens() {
    let input = r#"x = 12
print(f"{x}")
"#;
    let tokens = Lexer::new(input).tokenize();
    assert_eq!(
        tokens,
        vec![
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Int(12),
            Token::Newline,
            Token::Ident("print".to_string()),
            Token::LParen,
            Token::InterpolatedString(vec![
                FStringPart::Text("".to_string()),
                FStringPart::Expr("x".to_string()),
                FStringPart::Text("".to_string()),
            ]),
            Token::RParen,
            Token::Newline,
            Token::EOF,
        ]
    );
}
