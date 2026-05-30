use std::{fmt, rc::Rc};

use crate::interpreter::{Interpreter, Result, RuntimeError};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Nil,
    NativeFunction(Rc<NativeFunction>),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
        match self {
            Value::NativeFunction(f) => {
                if args.len() != f.arity {
                    return Err(RuntimeError::ArityMismatch {
                        expected: f.arity,
                        found: args.len(),
                    });
                }

                (f.function)(args)
            }
            _ => Err(RuntimeError::NotCallable),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::NativeFunction(l), Self::NativeFunction(r)) => Rc::ptr_eq(l, r),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s:?}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::NativeFunction(native) => write!(f, "<native fun {}>", native.name),
        }
    }
}

#[derive(Debug)]
pub struct NativeFunction {
    pub name: &'static str,
    pub arity: usize,
    pub function: fn(Vec<Value>) -> Result<Value>,
}

#[derive(Debug)]
pub enum ValueKind {
    Number,
    String,
    Bool,
    Nil,
    NativeFunction,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValueKind::*;

        match self {
            Number => write!(f, "number"),
            String => write!(f, "string"),
            Bool => write!(f, "bool"),
            Nil => write!(f, "nil"),
            NativeFunction => write!(f, "native_function"),
        }
    }
}

impl From<Value> for ValueKind {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(_) => Self::Number,
            Value::String(_) => Self::String,
            Value::Bool(_) => Self::Bool,
            Value::Nil => Self::Nil,
            Value::NativeFunction(_) => Self::NativeFunction,
        }
    }
}
