use std::fmt;
use std::result;

use crate::interpreter::Value;
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
    NotCallable,
    ArityMismatch {
        expected: usize,
        found: usize,
    },
    Return(Value),
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
            NotCallable => write!(f, "can only call functions and classes"),
            ArityMismatch { expected, found } => write!(
                f,
                "argument count must match function arity; expected `{expected}`, received `{found}`"
            ),
            Return(value) => todo!(),
        }
    }
}
