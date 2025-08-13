use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Str(String),
    Ident(String),
    Plus,
    Equal,
    LParen,
    RParen,
    Comma,
    Newline,
    EOF,
    InterpolatedString(Vec<FStringPart>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Text(String),
    Expr(String),
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
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
        self.skip_whitespace();
        let ch = match self.chars.peek().copied() {
            Some(c) => c,
            None => return Token::EOF,
        };

        match ch {
            '\n' => {
                self.chars.next();
                Token::Newline
            }
            '=' => {
                self.chars.next();
                Token::Equal
            }
            '+' => {
                self.chars.next();
                Token::Plus
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
            _ => {
                // Unknown character, skip
                self.chars.next();
                Token::EOF
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
        Token::Ident(ident)
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

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.chars.next();
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
