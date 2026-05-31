use std::fmt;

pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return(Option<Expr>),
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Logical {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{literal}"),
            Expr::Variable(name) => write!(f, "{name}"),
            Expr::Unary { op, right } => write!(f, "{op}{right}"),
            Expr::Binary { left, op, right } => write!(f, "{left} {op} {right}"),
            Expr::Grouping { expr } => write!(f, "({expr})"),
            Expr::Assign { name, value } => write!(f, "{name} = {value}"),
            Expr::Logical { left, op, right } => write!(f, "{left} {op} {right}"),
            Expr::Call { callee, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{callee}({arguments})")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::String(s) => write!(f, "{s:?}"),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::Bool(b) => write!(f, "{b}"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Negate => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Neq => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Lte => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Gte => write!(f, ">="),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LogicalOp {
    Or,
    And,
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOp::Or => write!(f, "or"),
            LogicalOp::And => write!(f, "and"),
        }
    }
}
