use std::fmt;
use std::result;

use crate::lexer::TokenKind;

pub type Result<T> = result::Result<T, SyntaxError>;

pub enum SyntaxError {
    UnexpectedToken {
        expected: Vec<TokenKind>,
        found: TokenKind,
    },
    InvalidExpression,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SyntaxError::*;

        match self {
            UnexpectedToken { expected, found } => {
                let expected_list = expected
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(
                    f,
                    "unexpected token `{found}`; expected{}: {expected_list}",
                    if expected_list.len() == 1 {
                        ""
                    } else {
                        " one of"
                    }
                )
            }
            InvalidExpression => write!(f, "invalid expression"),
        }
    }
}
