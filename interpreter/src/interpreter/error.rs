use std::result;

use crate::{
    interpreter::Value,
    parser::{BinaryOp, UnaryOp},
};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnaryTypeMismatch {
        op: UnaryOp,
        actual_type: Value,
    },
    BinaryTypeMismatch {
        left_type: Value,
        op: BinaryOp,
        right_type: Value,
    },
    VariableNotFound {
        name: String,
    },
}
