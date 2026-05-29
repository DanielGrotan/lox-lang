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

    pub fn next_token(&mut self) -> Token {
        loop {
            let Some(first_char) = self.bump() else {
                return Token::new(TokenKind::Eof, self.line);
            };

            if first_char.is_whitespace() {
                continue;
            }

            if first_char == '/' && self.eat('/') {
                self.eat_until('\n');
                continue;
            }

            let token_kind = match first_char {
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                '-' => TokenKind::Minus,
                '+' => TokenKind::Plus,
                ';' => TokenKind::Semicolon,
                '/' => TokenKind::Slash,
                '*' => TokenKind::Star,

                '!' => self.two_char('=', TokenKind::Neq, TokenKind::Bang),
                '=' => self.two_char('=', TokenKind::Eq, TokenKind::Assign),
                '<' => self.two_char('=', TokenKind::Lte, TokenKind::Lt),
                '>' => self.two_char('=', TokenKind::Gte, TokenKind::Gt),

                '"' => match self.string() {
                    Some(s) => TokenKind::String(s),
                    None => panic!("unterminated string"),
                },
                c if c.is_ascii_digit() => TokenKind::Number(self.number(c)),
                c if Self::is_ident_start(c) => Self::keyword(self.identifier(c)),

                _ => todo!(),
            };

            return Token::new(token_kind, self.line);
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

    fn number(&mut self, first: char) -> f64 {
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

        s.parse().unwrap()
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

    fn keyword(identifier: String) -> TokenKind {
        match identifier.as_str() {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "fun" => TokenKind::Fun,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier(identifier),
        }
    }
}
