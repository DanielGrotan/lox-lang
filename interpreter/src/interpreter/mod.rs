use std::{cell::RefCell, rc::Rc};

use crate::parser::{BinaryOp, Expr, Literal, Program, Stmt, UnaryOp};

pub mod error;
pub use error::*;

pub mod value;
pub use value::*;

pub mod environment;
pub use environment::*;

pub struct Interpreter {
    environment: EnvRef,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
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
            Stmt::Block(stmts) => self.execute_block(stmts),
        }
    }

    fn execute_expr(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr)?;

        Ok(())
    }

    fn execute_print(&mut self, expr: &Expr) -> Result<()> {
        let value = self.evaluate(expr)?;
        println!("{value}");

        Ok(())
    }

    fn execute_var(&mut self, name: &str, initializer: Option<&Expr>) -> Result<()> {
        let value = match initializer {
            Some(expr) => self.evaluate(expr)?,
            None => Value::Nil,
        };

        self.environment
            .borrow_mut()
            .define(name.to_string(), value);

        Ok(())
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(Environment::child(previous.clone())));

        for stmt in statements {
            self.execute(stmt)?;
        }

        self.environment = previous;

        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        Ok(match expr {
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Unary { op, right } => self.evaluate_unary(*op, right)?,
            Expr::Binary { left, op, right } => self.evaluate_binary(left, *op, right)?,
            Expr::Grouping { expr } => self.evaluate(expr)?,
            Expr::Variable(name) => self.evaluate_variable(name)?,
            Expr::Assign { name, value } => self.evaluate_assign(name, value)?,
        })
    }

    fn evaluate_literal(&self, literal: &Literal) -> Value {
        match literal {
            Literal::String(s) => Value::String(Rc::new(s.to_string())),
            Literal::Number(n) => Value::Number(*n),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Nil => Value::Nil,
        }
    }

    fn evaluate_unary(&mut self, op: UnaryOp, right: &Expr) -> Result<Value> {
        let value = self.evaluate(right)?;

        Ok(match (op, value) {
            (UnaryOp::Negate, Value::Number(n)) => Value::Number(-n),
            (UnaryOp::Not, l) => Value::Bool(!l.is_truthy()),
            (op, v) => {
                return Err(RuntimeError::UndefinedUnaryOp {
                    op,
                    operand: v.into(),
                });
            }
        })
    }

    fn evaluate_binary(&mut self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<Value> {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;

        Ok(match (left_value, op, right_value) {
            (Value::Number(l), BinaryOp::Add, Value::Number(r)) => Value::Number(l + r),
            (Value::Number(l), BinaryOp::Sub, Value::Number(r)) => Value::Number(l - r),
            (Value::Number(l), BinaryOp::Mul, Value::Number(r)) => Value::Number(l * r),
            (Value::Number(l), BinaryOp::Div, Value::Number(r)) => Value::Number(l / r),
            (Value::String(l), BinaryOp::Add, Value::String(r)) => {
                Value::String(Rc::new((*l).clone() + &r))
            }
            (Value::Number(l), BinaryOp::Gt, Value::Number(r)) => Value::Bool(l > r),
            (Value::Number(l), BinaryOp::Gte, Value::Number(r)) => Value::Bool(l >= r),
            (Value::Number(l), BinaryOp::Lt, Value::Number(r)) => Value::Bool(l < r),
            (Value::Number(l), BinaryOp::Lte, Value::Number(r)) => Value::Bool(l <= r),
            (l, BinaryOp::Eq, r) => Value::Bool(l == r),
            (l, BinaryOp::Neq, r) => Value::Bool(l != r),
            (l, op, r) => {
                return Err(RuntimeError::UndefinedBinaryOp {
                    op,
                    left: l.into(),
                    right: r.into(),
                });
            }
        })
    }

    fn evaluate_variable(&self, name: &str) -> Result<Value> {
        self.environment
            .borrow()
            .get(name)
            .ok_or_else(|| RuntimeError::VariableNotFound(name.to_string()))
    }

    fn evaluate_assign(&mut self, name: &str, expr: &Expr) -> Result<Value> {
        let value = self.evaluate(expr)?;
        self.environment
            .borrow_mut()
            .assign(name, value.clone())
            .ok_or_else(|| RuntimeError::VariableNotFound(name.to_string()))?;
        Ok(value)
    }
}
