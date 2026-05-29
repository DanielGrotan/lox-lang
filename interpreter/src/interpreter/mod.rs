use crate::parser::{BinaryOp, Expr, Literal, Program, Stmt, UnaryOp};

pub mod error;
pub use error::*;

pub mod value;
pub use value::*;

pub mod environment;
pub use environment::*;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<()> {
        for stmt in &program.statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expr(expr) => self.execute_expr(expr),
            Stmt::Print(expr) => self.execute_print(expr),
            Stmt::Var { name, initializer } => self.execute_var(name, initializer.as_ref()),
        }
    }

    fn execute_expr(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr)?;

        Ok(())
    }

    fn execute_print(&self, expr: &Expr) -> Result<()> {
        let value = self.evaluate(expr)?;
        println!("{value}");

        Ok(())
    }

    fn execute_var(&mut self, name: &str, initializer: Option<&Expr>) -> Result<()> {
        let value = match initializer {
            Some(expr) => self.evaluate(expr)?,
            None => Value::Nil,
        };

        self.environment.define(name.to_string(), value);

        Ok(())
    }

    fn evaluate(&self, expr: &Expr) -> Result<Value> {
        Ok(match expr {
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Unary { op, right } => self.evaluate_unary(*op, right)?,
            Expr::Binary { left, op, right } => self.evaluate_binary(left, *op, right)?,
            Expr::Grouping { expr } => self.evaluate(expr)?,
            Expr::Variable(name) => self.evaluate_variable(name)?,
        })
    }

    fn evaluate_literal(&self, literal: &Literal) -> Value {
        match literal {
            Literal::String(s) => Value::String(s.into()),
            Literal::Number(n) => Value::Number(*n),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Nil => Value::Nil,
        }
    }

    fn evaluate_unary(&self, op: UnaryOp, right: &Expr) -> Result<Value> {
        let value = self.evaluate(right)?;

        Ok(match (op, value) {
            (UnaryOp::Negate, Value::Number(n)) => Value::Number(-n),
            (UnaryOp::Not, l) => Value::Bool(!l.is_truthy()),
            (op, v) => return Err(Error::UnaryTypeMismatch { op, actual_type: v }),
        })
    }

    fn evaluate_binary(&self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<Value> {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;

        Ok(match (left_value, op, right_value) {
            (Value::Number(l), BinaryOp::Add, Value::Number(r)) => Value::Number(l + r),
            (Value::Number(l), BinaryOp::Sub, Value::Number(r)) => Value::Number(l - r),
            (Value::Number(l), BinaryOp::Mul, Value::Number(r)) => Value::Number(l * r),
            (Value::Number(l), BinaryOp::Div, Value::Number(r)) => Value::Number(l / r),
            (Value::String(l), BinaryOp::Add, Value::String(r)) => Value::String(l + &r),
            (Value::Number(l), BinaryOp::Gt, Value::Number(r)) => Value::Bool(l > r),
            (Value::Number(l), BinaryOp::Gte, Value::Number(r)) => Value::Bool(l >= r),
            (Value::Number(l), BinaryOp::Lt, Value::Number(r)) => Value::Bool(l < r),
            (Value::Number(l), BinaryOp::Lte, Value::Number(r)) => Value::Bool(l <= r),
            (l, BinaryOp::Eq, r) => Value::Bool(l == r),
            (l, BinaryOp::Neq, r) => Value::Bool(l != r),
            (l, op, r) => {
                return Err(Error::BinaryTypeMismatch {
                    left_type: l,
                    op,
                    right_type: r,
                });
            }
        })
    }

    fn evaluate_variable(&self, name: &str) -> Result<Value> {
        self.environment
            .get(name)
            .cloned()
            .ok_or_else(|| Error::VariableNotFound {
                name: name.to_string(),
            })
    }
}
