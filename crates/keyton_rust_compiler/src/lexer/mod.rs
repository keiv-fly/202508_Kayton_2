use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Str(String),
    Ident(String),
    LetKw,
    FnKw,
    ReturnKw,
    Plus,
    Equal,
    LParen,
    RParen,
    Comma,
    Colon,
    Indent,
    Dedent,
    Newline,
    EOF,
    InterpolatedString(Vec<FStringPart>),
    ForKw,
    InKw,
    DotDot,
    PlusEqual,
    Dot,
    LBracket,
    RBracket,
    LAngle,
    RAngle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Text(String),
    Expr(String),
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    at_line_start: bool,
    indent_stack: Vec<usize>,
    pending: VecDeque<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            at_line_start: true,
            indent_stack: vec![0],
            pending: VecDeque::new(),
        }
    }

    /// Tokenize the input. The resulting token stream will always end with `Token::EOF`.
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let end = tok == Token::EOF;
            tokens.push(tok);
            if end {
                break;
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        if let Some(tok) = self.pending.pop_front() {
            return tok;
        }

        // Handle indentation at start of line
        if self.at_line_start {
            // Count spaces; tabs are not allowed anywhere
            let mut spaces = 0usize;
            loop {
                match self.chars.peek().copied() {
                    Some(' ') => {
                        self.chars.next();
                        spaces += 1;
                    }
                    Some('\t') => panic!("tabs are not allowed"),
                    _ => break,
                }
            }

            // Blank line handling
            if let Some('\n') = self.chars.peek().copied() {
                self.chars.next();
                // Stay at start of line for the next token
                self.at_line_start = true;
                return Token::Newline;
            }

            let current = *self.indent_stack.last().unwrap();
            if spaces == current {
                // No change in indent
                self.at_line_start = false;
            } else if spaces > current {
                // Enforce exactly +4 spaces
                if spaces != current + 4 {
                    panic!("indentation must increase by exactly 4 spaces");
                }
                self.indent_stack.push(spaces);
                self.at_line_start = false;
                return Token::Indent;
            } else {
                // Dedent(s) to a previous level
                while let Some(&top) = self.indent_stack.last() {
                    if top > spaces {
                        self.indent_stack.pop();
                        self.pending.push_back(Token::Dedent);
                    } else {
                        break;
                    }
                }
                // After popping, the top must match exactly
                if *self.indent_stack.last().unwrap() != spaces {
                    panic!("invalid dedent level; must match a previous indentation");
                }
                self.at_line_start = false;
                if let Some(tok) = self.pending.pop_front() {
                    return tok;
                }
            }
        }

        self.skip_inline_spaces();
        let ch = match self.chars.peek().copied() {
            Some(c) => c,
            None => {
                // At EOF, emit any remaining dedents
                if self.indent_stack.len() > 1 {
                    while self.indent_stack.len() > 1 {
                        self.indent_stack.pop();
                        self.pending.push_back(Token::Dedent);
                    }
                    return self.pending.pop_front().unwrap();
                }
                return Token::EOF;
            }
        };

        match ch {
            '\n' => {
                self.chars.next();
                self.at_line_start = true;
                Token::Newline
            }
            '=' => {
                self.chars.next();
                Token::Equal
            }
            '+' => {
                self.chars.next();
                if let Some('=') = self.chars.peek().copied() {
                    self.chars.next();
                    Token::PlusEqual
                } else {
                    Token::Plus
                }
            }
            '(' => {
                self.chars.next();
                Token::LParen
            }
            ')' => {
                self.chars.next();
                Token::RParen
            }
            ',' => {
                self.chars.next();
                Token::Comma
            }
            ':' => {
                self.chars.next();
                Token::Colon
            }
            '.' => {
                // Possibly Dot or DotDot
                self.chars.next();
                if let Some('.') = self.chars.peek().copied() {
                    self.chars.next();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }
            '[' => {
                self.chars.next();
                Token::LBracket
            }
            ']' => {
                self.chars.next();
                Token::RBracket
            }
            '<' => {
                self.chars.next();
                Token::LAngle
            }
            '>' => {
                self.chars.next();
                Token::RAngle
            }
            '0'..='9' => self.lex_number(ch),
            'a'..='z' | 'A'..='Z' | '_' => {
                if ch == 'f' {
                    if let Some('"') = self.peek_next() {
                        return self.lex_fstring();
                    }
                }
                self.lex_ident(ch)
            }
            '"' => self.lex_string(),
            '\t' => panic!("tabs are not allowed"),
            _ => {
                // Unknown character, skip and continue
                self.chars.next();
                self.next_token()
            }
        }
    }

    fn lex_number(&mut self, first: char) -> Token {
        let mut num = first.to_string();
        self.chars.next();
        while let Some(c) = self.chars.peek() {
            if c.is_ascii_digit() {
                num.push(*c);
                self.chars.next();
            } else {
                break;
            }
        }
        Token::Int(num.parse().unwrap())
    }

    fn lex_ident(&mut self, first: char) -> Token {
        let mut ident = first.to_string();
        self.chars.next();
        while let Some(c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || *c == '_' {
                ident.push(*c);
                self.chars.next();
            } else {
                break;
            }
        }
        match ident.as_str() {
            "let" => Token::LetKw,
            "fn" => Token::FnKw,
            "return" => Token::ReturnKw,
            "for" => Token::ForKw,
            "in" => Token::InKw,
            _ => Token::Ident(ident),
        }
    }

    fn lex_string(&mut self) -> Token {
        self.chars.next(); // skip opening quote
        let mut s = String::new();
        while let Some(c) = self.chars.next() {
            if c == '"' {
                break;
            } else {
                s.push(c);
            }
        }
        Token::Str(s)
    }

    fn lex_fstring(&mut self) -> Token {
        self.chars.next(); // consume 'f'
        self.chars.next(); // consume opening quote
        let mut parts = vec![FStringPart::Text(String::new())];
        let mut current_index = 0; // index of current text part
        while let Some(c) = self.chars.next() {
            match c {
                '"' => break,
                '{' => {
                    let mut expr_src = String::new();
                    while let Some(ch) = self.chars.next() {
                        if ch == '}' {
                            break;
                        } else {
                            expr_src.push(ch);
                        }
                    }
                    parts.push(FStringPart::Expr(expr_src));
                    parts.push(FStringPart::Text(String::new()));
                    current_index = parts.len() - 1;
                }
                _ => {
                    if let FStringPart::Text(ref mut t) = parts[current_index] {
                        t.push(c);
                    }
                }
            }
        }
        Token::InterpolatedString(parts)
    }

    fn skip_inline_spaces(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\r' {
                self.chars.next();
            } else if c == '\t' {
                panic!("tabs are not allowed");
            } else {
                break;
            }
        }
    }

    fn peek_next(&mut self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.peek().copied()
    }
}

#[cfg(test)]
mod tests;
