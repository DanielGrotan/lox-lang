use std::fmt;
use std::result;

use crate::{
    interpreter::ValueKind,
    parser::{BinaryOp, UnaryOp},
};

pub type Result<T> = result::Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedUnaryOp {
        op: UnaryOp,
        operand: ValueKind,
    },
    UndefinedBinaryOp {
        op: BinaryOp,
        left: ValueKind,
        right: ValueKind,
    },
    VariableNotFound(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuntimeError::*;

        match self {
            UndefinedUnaryOp { op, operand } => {
                write!(f, "undefined unary operator `{op}` for {operand}")
            }
            UndefinedBinaryOp { op, left, right } => {
                write!(f, "undefined binary operator `{op}` for {left} and {right}")
            }
            VariableNotFound(name) => write!(f, "undefined variable `{name}`"),
        }
    }
}
