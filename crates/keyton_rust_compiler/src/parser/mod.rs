use crate::lexer::{FStringPart, Lexer, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign { name: String, expr: Expr },
    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Str(String),
    Ident(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    InterpolatedString(Vec<StringPart>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Expr(Box<Expr>),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            self.skip_newlines();
        }
        stmts
    }

    pub fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_primary();
        while matches!(self.peek(), Token::Plus) {
            self.advance();
            let right = self.parse_primary();
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Add,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        if self.is_at_end() {
            return None;
        }
        if let Token::Ident(name) = self.peek().clone() {
            if self.peek_next_is(Token::Equal) {
                self.advance(); // ident
                self.advance(); // '='
                let expr = self.parse_expr();
                return Some(Stmt::Assign { name, expr });
            }
        }
        let expr = self.parse_expr();
        Some(Stmt::ExprStmt(expr))
    }

    fn parse_primary(&mut self) -> Expr {
        match self.advance() {
            Token::Int(n) => Expr::Int(n),
            Token::Str(s) => Expr::Str(s),
            Token::Ident(s) => {
                let expr = Expr::Ident(s);
                self.parse_call(expr)
            }
            Token::InterpolatedString(parts) => {
                let mut ast_parts = Vec::new();
                for part in parts {
                    match part {
                        FStringPart::Text(t) => ast_parts.push(StringPart::Text(t)),
                        FStringPart::Expr(src) => {
                            let expr = parse_embedded_expr(&src);
                            ast_parts.push(StringPart::Expr(Box::new(expr)));
                        }
                    }
                }
                Expr::InterpolatedString(ast_parts)
            }
            Token::LParen => {
                let expr = self.parse_expr();
                self.expect(Token::RParen);
                self.parse_call(expr)
            }
            other => panic!("Unexpected token {:?}", other),
        }
    }

    fn parse_call(&mut self, mut expr: Expr) -> Expr {
        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        args.push(self.parse_expr());
                        while matches!(self.peek(), Token::Comma) {
                            self.advance();
                            args.push(self.parse_expr());
                        }
                    }
                    self.expect(Token::RParen);
                    expr = Expr::Call {
                        func: Box::new(expr),
                        args,
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    fn expect(&mut self, expected: Token) {
        let tok = self.advance();
        if tok != expected {
            panic!("expected {:?}, found {:?}", expected, tok);
        }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF)
    }

    fn peek_next_is(&self, expected: Token) -> bool {
        self.tokens
            .get(self.pos + 1)
            .cloned()
            .map_or(false, |t| t == expected)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek();
        if !self.is_at_end() {
            self.pos += 1;
        }
        tok
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }
}

fn parse_embedded_expr(src: &str) -> Expr {
    let tokens = Lexer::new(src).tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_expr()
}

#[cfg(test)]
mod tests;
