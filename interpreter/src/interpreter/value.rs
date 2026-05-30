use std::{fmt, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub enum ValueKind {
    Number,
    String,
    Bool,
    Nil,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValueKind::*;

        match self {
            Number => write!(f, "number"),
            String => write!(f, "string"),
            Bool => write!(f, "bool"),
            Nil => write!(f, "nil"),
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
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Nil => false,
            _ => true,
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
        }
    }
}
