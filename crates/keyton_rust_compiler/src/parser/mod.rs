use crate::lexer::{FStringPart, Lexer, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // rimport reqwest
    RImportModule {
        module: String,
    },
    // from reqwest rimport Client, StatusCode
    RImportItems {
        module: String,
        items: Vec<String>,
    },
    Assign {
        name: String,
        expr: Expr,
    },
    ForRange {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    ExprStmt(Expr),
    FuncDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Str(String),
    Ident(String),
    Bool(bool),
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
        // rimport statements
        if matches!(self.peek(), Token::RimportKw) {
            self.advance(); // 'rimport'
            let module = match self.advance() {
                Token::Ident(s) => s,
                other => panic!("expected module name after rimport, found {:?}", other),
            };
            return Some(Stmt::RImportModule { module });
        }
        // from X rimport A, B, ...
        if matches!(self.peek(), Token::FromKw) {
            self.advance(); // 'from'
            let module = match self.advance() {
                Token::Ident(s) => s,
                other => panic!("expected module name after from, found {:?}", other),
            };
            self.expect(Token::RimportKw);
            let mut items = Vec::new();
            loop {
                let name = match self.advance() {
                    Token::Ident(s) => s,
                    other => panic!("expected identifier in rimport list, found {:?}", other),
                };
                items.push(name);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
            return Some(Stmt::RImportItems { module, items });
        }
        // Let declaration (desugars to assignment)
        if matches!(self.peek(), Token::LetKw) {
            self.advance(); // 'let'
            let name = match self.advance() {
                Token::Ident(s) => s,
                other => panic!("expected identifier after let, found {:?}", other),
            };
            self.expect(Token::Colon);
            // Skip type tokens until '='
            while !matches!(self.peek(), Token::Equal) {
                self.advance();
            }
            self.expect(Token::Equal);
            let expr = self.parse_expr();
            return Some(Stmt::Assign { name, expr });
        }
        // For loop
        if matches!(self.peek(), Token::ForKw) {
            return Some(self.parse_for_range());
        }
        // If statement
        if matches!(self.peek(), Token::IfKw) {
            return Some(self.parse_if());
        }
        // Function definition
        if matches!(self.peek(), Token::FnKw) {
            return Some(self.parse_func_def());
        }
        // Return statement
        if matches!(self.peek(), Token::ReturnKw) {
            self.advance();
            let expr = self.parse_expr();
            return Some(Stmt::Return(expr));
        }
        if let Token::Ident(name) = self.peek().clone() {
            if self.peek_next_is(Token::Colon) {
                // Typed assignment without let: name: Type = expr
                self.advance(); // ident
                self.expect(Token::Colon);
                while !matches!(self.peek(), Token::Equal) {
                    self.advance();
                }
                self.expect(Token::Equal);
                let expr = self.parse_expr();
                return Some(Stmt::Assign { name, expr });
            } else if self.peek_next_is(Token::Equal) {
                self.advance(); // ident
                self.advance(); // '='
                let expr = self.parse_expr();
                return Some(Stmt::Assign { name, expr });
            } else if self.peek_next_is(Token::PlusEqual) {
                // Desugar: x += y  =>  x = x + y
                self.advance(); // ident
                self.advance(); // '+='
                let rhs = self.parse_expr();
                let lhs = Expr::Ident(name.clone());
                let expr = Expr::Binary {
                    left: Box::new(lhs),
                    op: BinOp::Add,
                    right: Box::new(rhs),
                };
                return Some(Stmt::Assign { name, expr });
            }
        }
        let expr = self.parse_expr();
        Some(Stmt::ExprStmt(expr))
    }

    fn parse_func_def(&mut self) -> Stmt {
        self.expect(Token::FnKw);
        let name = match self.advance() {
            Token::Ident(s) => s,
            other => panic!("expected function name, found {:?}", other),
        };
        self.expect(Token::LParen);
        let mut params = Vec::new();
        if !matches!(self.peek(), Token::RParen) {
            loop {
                match self.advance() {
                    Token::Ident(p) => params.push(p),
                    other => panic!("expected parameter name, found {:?}", other),
                }
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect(Token::RParen);
        self.expect(Token::Colon);
        self.expect(Token::Newline);
        self.expect(Token::Indent);
        let mut body = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), Token::Dedent | Token::EOF) {
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            }
            self.skip_newlines();
        }
        self.expect(Token::Dedent);

        // If the last line is an expression statement, treat it as an implicit return
        if let Some(last) = body.pop() {
            match last {
                Stmt::ExprStmt(expr) => body.push(Stmt::Return(expr)),
                other => body.push(other),
            }
        }
        Stmt::FuncDef { name, params, body }
    }

    fn parse_for_range(&mut self) -> Stmt {
        self.expect(Token::ForKw);
        let var = match self.advance() {
            Token::Ident(s) => s,
            other => panic!("expected loop variable name, found {:?}", other),
        };
        self.expect(Token::InKw);
        let start = self.parse_expr();
        self.expect(Token::DotDot);
        let end = self.parse_expr();
        self.expect(Token::Colon);
        self.expect(Token::Newline);
        self.expect(Token::Indent);
        let mut body = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), Token::Dedent | Token::EOF) {
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            }
            self.skip_newlines();
        }
        self.expect(Token::Dedent);
        Stmt::ForRange {
            var,
            start,
            end,
            body,
        }
    }

    fn parse_if(&mut self) -> Stmt {
        self.expect(Token::IfKw);
        let cond = self.parse_expr();
        self.expect(Token::Colon);
        self.expect(Token::Newline);
        self.expect(Token::Indent);
        let mut then_branch = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), Token::Dedent | Token::EOF) {
            if let Some(stmt) = self.parse_stmt() {
                then_branch.push(stmt);
            }
            self.skip_newlines();
        }
        self.expect(Token::Dedent);
        self.skip_newlines();

        let mut else_branch = Vec::new();
        if matches!(self.peek(), Token::ElseKw) {
            self.expect(Token::ElseKw);
            self.expect(Token::Colon);
            self.expect(Token::Newline);
            self.expect(Token::Indent);
            self.skip_newlines();
            while !matches!(self.peek(), Token::Dedent | Token::EOF) {
                if let Some(stmt) = self.parse_stmt() {
                    else_branch.push(stmt);
                }
                self.skip_newlines();
            }
            self.expect(Token::Dedent);
        }

        Stmt::If {
            cond,
            then_branch,
            else_branch,
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.advance() {
            Token::Int(n) => Expr::Int(n),
            Token::Str(s) => Expr::Str(s),
            Token::TrueKw => Expr::Bool(true),
            Token::FalseKw => Expr::Bool(false),
            Token::Ident(s) => {
                let expr = Expr::Ident(s);
                self.parse_postfix(expr)
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
                self.parse_postfix(expr)
            }
            Token::LBracket => {
                let mut elems = Vec::new();
                if !matches!(self.peek(), Token::RBracket) {
                    elems.push(self.parse_expr());
                    while matches!(self.peek(), Token::Comma) {
                        self.advance();
                        elems.push(self.parse_expr());
                    }
                }
                self.expect(Token::RBracket);
                let expr = Expr::Call {
                    func: Box::new(Expr::Ident("vec".to_string())),
                    args: elems,
                };
                self.parse_postfix(expr)
            }
            other => panic!("Unexpected token {:?}", other),
        }
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Expr {
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
                Token::Dot => {
                    self.advance(); // consume '.'
                    let method = match self.advance() {
                        Token::Ident(s) => s,
                        other => panic!("expected method name after '.', found {:?}", other),
                    };
                    self.expect(Token::LParen);
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        args.push(self.parse_expr());
                        while matches!(self.peek(), Token::Comma) {
                            self.advance();
                            args.push(self.parse_expr());
                        }
                    }
                    self.expect(Token::RParen);
                    let mut call_args = Vec::new();
                    call_args.push(expr);
                    call_args.extend(args);
                    expr = Expr::Call {
                        func: Box::new(Expr::Ident(method)),
                        args: call_args,
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
