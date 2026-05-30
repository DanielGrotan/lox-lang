use std::fmt;

#[derive(Debug, Clone)]
pub enum TokenKind {
    // Single-character tokens.
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    Neq,
    Assign,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenKind::*;

        let s = match self {
            LParen => "(",
            RParen => ")",
            LBrace => "{",
            RBrace => "}",
            Comma => ",",
            Dot => ".",
            Minus => "-",
            Plus => "+",
            Semicolon => ";",
            Slash => "/",
            Star => "*",
            Bang => "!",
            Neq => "!=",
            Assign => "=",
            Eq => "==",
            Gt => ">",
            Gte => ">=",
            Lt => "<",
            Lte => "<=",
            Identifier => "identifer",
            String => "string",
            Number => "number",
            And => "and",
            Class => "class",
            Else => "else",
            False => "false",
            Fun => "fun",
            For => "for",
            If => "if",
            Nil => "nil",
            Or => "or",
            Print => "print",
            Return => "return",
            Super => "super",
            This => "this",
            True => "true",
            Var => "var",
            While => "while",
            Eof => "eof",
        };

        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: Option<String>,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: Option<String>, line: usize) -> Self {
        Self { kind, lexeme, line }
    }
}
