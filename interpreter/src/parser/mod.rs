use std::mem;

mod error;
mod grammar;

use crate::lexer::{Token, TokenKind};

pub use error::*;
pub use grammar::*;

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
            TokenKind::Fun => {
                self.bump();
                self.function()
            }
            TokenKind::Var => {
                self.bump();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn function(&mut self) -> Result<Stmt> {
        let name = self.consume_ident()?;

        self.consume(TokenKind::LParen)?;
        let mut params = Vec::new();

        if !matches!(self.peek_kind(), TokenKind::RParen) {
            loop {
                if params.len() >= 255 {
                    return Err(SyntaxError::TooManyArguments);
                }

                params.push(self.consume_ident()?);

                if !self.consume(TokenKind::Comma).is_ok() {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen)?;

        self.consume(TokenKind::LBrace)?;
        let body = self.block_statements()?;

        Ok(Stmt::Function { name, params, body })
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
            TokenKind::If => {
                self.bump();
                self.if_statement()
            }
            TokenKind::LBrace => {
                self.bump();
                self.block()
            }
            TokenKind::While => {
                self.bump();
                self.r#while()
            }
            TokenKind::For => {
                self.bump();
                self.r#for()
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;

        self.consume(TokenKind::Semicolon)?;

        Ok(Stmt::Print(expr))
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenKind::LParen)?;
        let condition = self.expression()?;
        self.consume(TokenKind::RParen)?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = match self.consume(TokenKind::Else) {
            Ok(_) => Some(Box::new(self.statement()?)),
            _ => None,
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Stmt> {
        Ok(Stmt::Block(self.block_statements()?))
    }

    fn block_statements(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !matches!(self.peek_kind(), TokenKind::RBrace) {
            statements.push(self.declaration()?)
        }

        self.consume(TokenKind::RBrace)?;

        Ok(statements)
    }

    fn r#while(&mut self) -> Result<Stmt> {
        self.consume(TokenKind::LParen)?;
        let condition = self.expression()?;
        self.consume(TokenKind::RParen)?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn r#for(&mut self) -> Result<Stmt> {
        self.consume(TokenKind::LParen)?;

        let initializer = match self.peek_kind() {
            TokenKind::Semicolon => None,
            TokenKind::Var => {
                self.bump();
                Some(self.var_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };

        let condition = match self.peek_kind() {
            TokenKind::Semicolon => Expr::Literal(Literal::Bool(true)),
            _ => self.expression()?,
        };
        self.consume(TokenKind::Semicolon)?;

        let increment = match self.peek_kind() {
            TokenKind::RParen => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenKind::RParen)?;

        let mut body = self.statement()?;

        if let Some(stmt) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(stmt)]);
        }

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(stmt) = initializer {
            body = Stmt::Block(vec![stmt, body]);
        }

        Ok(body)
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
        let left = self.or()?;

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

    fn or(&mut self) -> Result<Expr> {
        let lhs = self.and()?;

        self.logical_left_associative(
            lhs,
            |k| match k {
                TokenKind::Or => Some(LogicalOp::Or),
                _ => None,
            },
            Self::and,
        )
    }

    fn and(&mut self) -> Result<Expr> {
        let lhs = self.equality()?;

        self.logical_left_associative(
            lhs,
            |k| match k {
                TokenKind::And => Some(LogicalOp::And),
                _ => None,
            },
            Self::equality,
        )
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
            _ => return self.call(),
        };
        self.bump();

        let right = self.unary()?;

        Ok(Expr::Unary {
            op,
            right: Box::new(right),
        })
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            match self.peek_kind() {
                TokenKind::LParen => {
                    expr = {
                        self.bump();
                        self.finish_call(expr)?
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, expr: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();

        if !matches!(self.peek_kind(), TokenKind::RParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(SyntaxError::TooManyArguments);
                }
                arguments.push(self.expression()?);

                if !self.consume(TokenKind::Comma).is_ok() {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen)?;

        Ok(Expr::Call {
            callee: Box::new(expr),
            arguments,
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

    fn logical_left_associative(
        &mut self,
        mut lhs: Expr,
        next_precedence: fn(&TokenKind) -> Option<LogicalOp>,
        next_parse: fn(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        loop {
            let op = match next_precedence(self.peek_kind()) {
                Some(op) => op,
                None => break,
            };

            self.bump();

            let rhs = next_parse(self)?;

            lhs = Expr::Logical {
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

    fn consume_ident(&mut self) -> Result<String> {
        match self.peek() {
            Token {
                kind: TokenKind::Identifier,
                lexeme,
                ..
            } => {
                let name = lexeme.clone().unwrap();
                self.bump();
                Ok(name)
            }
            t => Err(SyntaxError::UnexpectedToken {
                expected: vec![TokenKind::Identifier],
                found: t.kind.clone(),
            }),
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
