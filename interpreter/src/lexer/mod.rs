mod error;
pub use error::*;

pub mod token;
pub use token::*;

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars(),
            line: 1,
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        loop {
            let Some(first_char) = self.bump() else {
                return Ok(Token::new(TokenKind::Eof, None, self.line));
            };

            if first_char.is_whitespace() {
                continue;
            }

            if first_char == '/' && self.eat('/') {
                self.eat_until('\n');
                continue;
            }

            let (token_kind, lexeme) = match first_char {
                '(' => (TokenKind::LParen, None),
                ')' => (TokenKind::RParen, None),
                '{' => (TokenKind::LBrace, None),
                '}' => (TokenKind::RBrace, None),
                ',' => (TokenKind::Comma, None),
                '.' => (TokenKind::Dot, None),
                '-' => (TokenKind::Minus, None),
                '+' => (TokenKind::Plus, None),
                ';' => (TokenKind::Semicolon, None),
                '/' => (TokenKind::Slash, None),
                '*' => (TokenKind::Star, None),

                '!' => (self.two_char('=', TokenKind::Neq, TokenKind::Bang), None),
                '=' => (self.two_char('=', TokenKind::Eq, TokenKind::Assign), None),
                '<' => (self.two_char('=', TokenKind::Lte, TokenKind::Lt), None),
                '>' => (self.two_char('=', TokenKind::Gte, TokenKind::Gt), None),

                '"' => {
                    let string = self.string().ok_or(LexError::UnterminatedString)?;

                    (TokenKind::String, Some(string))
                }
                c if c.is_ascii_digit() => (TokenKind::Number, Some(self.number(c))),
                c if Self::is_ident_start(c) => Self::keyword(self.identifier(c)),

                c => {
                    self.bump();
                    return Err(LexError::InvalidCharacter(c));
                }
            };

            return Ok(Token::new(token_kind, lexeme, self.line));
        }
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        if c == '\n' {
            self.line += 1;
        }

        Some(c)
    }

    fn two_char(&mut self, expected: char, yes: TokenKind, no: TokenKind) -> TokenKind {
        if self.eat(expected) { yes } else { no }
    }

    fn eat(&mut self, target: char) -> bool {
        if self.first() == Some(target) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn first(&self) -> Option<char> {
        self.chars.clone().next()
    }

    fn second(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next()
    }

    fn eat_until(&mut self, target: char) {
        while let Some(c) = self.bump() {
            if c == target {
                break;
            }
        }
    }

    fn string(&mut self) -> Option<String> {
        let mut s = String::new();

        while let Some(c) = self.bump() {
            if c == '"' {
                return Some(s);
            }
            s.push(c);
        }

        None
    }

    fn number(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);

        while let Some(c) = self.first() {
            if c.is_ascii_digit() {
                self.bump();
                s.push(c);
            } else {
                break;
            }
        }

        match (self.first(), self.second()) {
            (Some('.'), Some(c)) if c.is_ascii_digit() => {
                self.bump();
                s.push('.');
                while let Some(c) = self.first() {
                    if c.is_ascii_digit() {
                        self.bump();
                        s.push(c);
                    } else {
                        break;
                    }
                }
            }
            _ => (),
        }

        s
    }

    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn identifier(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);

        while let Some(c) = self.first() {
            if Self::is_ident_continue(c) {
                self.bump();
                s.push(c);
            } else {
                break;
            }
        }

        s
    }

    fn keyword(identifier: String) -> (TokenKind, Option<String>) {
        match identifier.as_str() {
            "and" => (TokenKind::And, None),
            "class" => (TokenKind::Class, None),
            "else" => (TokenKind::Else, None),
            "false" => (TokenKind::False, None),
            "fun" => (TokenKind::Fun, None),
            "for" => (TokenKind::For, None),
            "if" => (TokenKind::If, None),
            "nil" => (TokenKind::Nil, None),
            "or" => (TokenKind::Or, None),
            "print" => (TokenKind::Print, None),
            "return" => (TokenKind::Return, None),
            "super" => (TokenKind::Super, None),
            "this" => (TokenKind::This, None),
            "true" => (TokenKind::True, None),
            "var" => (TokenKind::Var, None),
            "while" => (TokenKind::While, None),
            _ => (TokenKind::Identifier, Some(identifier)),
        }
    }
}
