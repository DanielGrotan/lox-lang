use std::mem;

use crate::lexer::{Token, TokenKind};

mod grammar;
pub use grammar::*;

mod error;
pub use error::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> (Program, Vec<SyntaxError>) {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        (Program::new(statements), errors)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        match self.peek_kind() {
            TokenKind::Var => {
                self.bump();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = match self.peek_kind() {
            TokenKind::Identifier => {
                let name = self.peek().lexeme.clone().unwrap();
                self.bump();
                name
            }
            kind => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: vec![TokenKind::Identifier],
                    found: kind.clone(),
                });
            }
        };

        let initializer = if self.consume(TokenKind::Assign).is_ok() {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon)?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        match self.peek_kind() {
            TokenKind::Print => {
                self.bump();
                self.print_statement()
            }
            TokenKind::LBrace => {
                self.bump();
                self.block()
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;

        self.consume(TokenKind::Semicolon)?;

        Ok(Stmt::Print(expr))
    }

    fn block(&mut self) -> Result<Stmt> {
        let mut statements = Vec::new();

        loop {
            if matches!(self.peek_kind(), TokenKind::RBrace) {
                break;
            }
            statements.push(self.declaration()?)
        }

        self.consume(TokenKind::RBrace)?;

        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;

        self.consume(TokenKind::Semicolon)?;

        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let left = self.equality()?;

        if matches!(self.peek_kind(), TokenKind::Assign) {
            self.bump();

            let value = self.assignment()?;

            match left {
                Expr::Variable(name) => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(SyntaxError::InvalidExpression),
            }
        } else {
            Ok(left)
        }
    }

    fn equality(&mut self) -> Result<Expr> {
        let lhs = self.comparison()?;

        self.binary_left_associative(
            lhs,
            |k| match k {
                TokenKind::Eq => Some(BinaryOp::Eq),
                TokenKind::Neq => Some(BinaryOp::Neq),
                _ => None,
            },
            Self::comparison,
        )
    }

    fn comparison(&mut self) -> Result<Expr> {
        let lhs = self.term()?;

        self.binary_left_associative(
            lhs,
            |k| match k {
                TokenKind::Gt => Some(BinaryOp::Gt),
                TokenKind::Gte => Some(BinaryOp::Gte),
                TokenKind::Lt => Some(BinaryOp::Lt),
                TokenKind::Lte => Some(BinaryOp::Lte),
                _ => None,
            },
            Self::term,
        )
    }

    fn term(&mut self) -> Result<Expr> {
        let lhs = self.factor()?;

        self.binary_left_associative(
            lhs,
            |k| match k {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Sub),
                _ => None,
            },
            Self::factor,
        )
    }

    fn factor(&mut self) -> Result<Expr> {
        let lhs = self.unary()?;

        self.binary_left_associative(
            lhs,
            |k| match k {
                TokenKind::Slash => Some(BinaryOp::Div),
                TokenKind::Star => Some(BinaryOp::Mul),
                _ => None,
            },
            Self::unary,
        )
    }

    fn unary(&mut self) -> Result<Expr> {
        let op = match self.peek_kind() {
            TokenKind::Bang => UnaryOp::Not,
            TokenKind::Minus => UnaryOp::Negate,
            _ => return self.primary(),
        };
        self.bump();

        let right = self.unary()?;

        Ok(Expr::Unary {
            op,
            right: Box::new(right),
        })
    }

    fn primary(&mut self) -> Result<Expr> {
        let literal = match self.peek_kind() {
            TokenKind::False => Expr::Literal(Literal::Bool(false)),
            TokenKind::True => Expr::Literal(Literal::Bool(true)),
            TokenKind::Nil => Expr::Literal(Literal::Nil),
            TokenKind::String => {
                Expr::Literal(Literal::String(self.peek().lexeme.clone().unwrap()))
            }
            TokenKind::Number => Expr::Literal(Literal::Number(
                self.peek()
                    .lexeme
                    .clone()
                    .unwrap()
                    .parse()
                    .expect("valid decimal number according to lexer"),
            )),
            TokenKind::Identifier => Expr::Variable(self.peek().lexeme.clone().unwrap()),
            TokenKind::LParen => {
                let expr = self.expression()?;
                self.consume(TokenKind::RParen)?;
                return Ok(Expr::Grouping {
                    expr: Box::new(expr),
                });
            }
            kind => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: vec![
                        TokenKind::False,
                        TokenKind::True,
                        TokenKind::Nil,
                        TokenKind::String,
                        TokenKind::Number,
                        TokenKind::Identifier,
                        TokenKind::LParen,
                    ],
                    found: kind.clone(),
                });
            }
        };
        self.bump();

        Ok(literal)
    }

    fn binary_left_associative(
        &mut self,
        mut lhs: Expr,
        next_precedence: fn(&TokenKind) -> Option<BinaryOp>,
        next_parse: fn(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        loop {
            let op = match next_precedence(self.peek_kind()) {
                Some(op) => op,
                None => break,
            };

            self.bump();

            let rhs = next_parse(self)?;

            lhs = Expr::Binary {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            }
        }

        Ok(lhs)
    }

    fn bump(&mut self) {
        self.current += 1;

        debug_assert!(self.current < self.tokens.len());
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.current].kind
    }

    fn consume(&mut self, expected: TokenKind) -> Result<()> {
        let found = self.peek_kind();
        if mem::discriminant(found) == mem::discriminant(&expected) {
            self.bump();
            Ok(())
        } else {
            Err(SyntaxError::UnexpectedToken {
                expected: vec![expected],
                found: found.clone(),
            })
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn synchronize(&mut self) {
        loop {
            match self.peek_kind() {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return
                | TokenKind::Eof => return,
                TokenKind::Semicolon => {
                    self.bump();
                    return;
                }
                _ => (),
            }

            self.bump();
        }
    }
}
