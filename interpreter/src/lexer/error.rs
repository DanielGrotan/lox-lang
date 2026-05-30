use std::{fmt, result};

pub type Result<T> = result::Result<T, LexError>;

pub enum LexError {
    InvalidCharacter(char),
    UnterminatedString,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LexError::*;

        match self {
            InvalidCharacter(c) => write!(f, "found invalid character `{c}`"),
            UnterminatedString => write!(f, "found unterminated string"),
        }
    }
}
