use std::mem;

use crate::lexer::{Token, TokenKind};

mod grammar;
pub use grammar::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                self.synchronize();
            }
        }

        Some(Program::new(statements))
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let token = self.first()?;

        match token.kind {
            TokenKind::Var => {
                self.bump();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Option<Stmt> {
        let token = self.first()?;
        let name = match &token.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.bump();
                name
            }
            _ => return None,
        };

        let initializer = if self.consume(TokenKind::Assign).is_some() {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon)?;

        Some(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Option<Stmt> {
        let token = self.first()?;

        match &token.kind {
            TokenKind::Print => {
                self.bump();
                self.print_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;

        self.consume(TokenKind::Semicolon)?;

        Some(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;

        self.consume(TokenKind::Semicolon)?;

        Some(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Option<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expr> {
        let left = self.equality()?;

        if let Some(Token {
            kind: TokenKind::Assign,
            ..
        }) = self.first()
        {
            self.bump();

            let value = self.assignment()?;

            match left {
                Expr::Variable(name) => Some(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => None,
            }
        } else {
            Some(left)
        }
    }

    fn equality(&mut self) -> Option<Expr> {
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

    fn comparison(&mut self) -> Option<Expr> {
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

    fn term(&mut self) -> Option<Expr> {
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

    fn factor(&mut self) -> Option<Expr> {
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

    fn unary(&mut self) -> Option<Expr> {
        let Some(token) = self.first() else {
            return self.primary();
        };

        let op = match token.kind {
            TokenKind::Bang => UnaryOp::Not,
            TokenKind::Minus => UnaryOp::Negate,
            _ => return self.primary(),
        };
        self.bump();

        let right = self.unary()?;

        Some(Expr::Unary {
            op,
            right: Box::new(right),
        })
    }

    fn primary(&mut self) -> Option<Expr> {
        let Some(token) = self.first() else {
            return None;
        };

        let literal = match &token.kind {
            TokenKind::False => Expr::Literal(Literal::Bool(false)),
            TokenKind::True => Expr::Literal(Literal::Bool(true)),
            TokenKind::Nil => Expr::Literal(Literal::Nil),
            TokenKind::String(s) => Expr::Literal(Literal::String(s.into())),
            TokenKind::Number(n) => Expr::Literal(Literal::Number(*n)),
            TokenKind::Identifier(name) => Expr::Variable(name.clone()),
            TokenKind::LParen => {
                let expr = self.expression()?;
                self.consume(TokenKind::RParen)?;
                return Some(Expr::Grouping {
                    expr: Box::new(expr),
                });
            }
            _ => return None,
        };
        self.bump();

        Some(literal)
    }

    fn binary_left_associative(
        &mut self,
        mut lhs: Expr,
        next_precedence: fn(&TokenKind) -> Option<BinaryOp>,
        next_parse: fn(&mut Self) -> Option<Expr>,
    ) -> Option<Expr> {
        while let Some(token) = self.first() {
            let op = match next_precedence(&token.kind) {
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

        Some(lhs)
    }

    fn bump(&mut self) {
        self.current += 1;
    }

    fn first(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn second(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn consume(&mut self, kind: TokenKind) -> Option<()> {
        let token = self.first()?;

        if mem::discriminant(&token.kind) == mem::discriminant(&kind) {
            self.bump();
            Some(())
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn synchronize(&mut self) {
        while let Some(token) = self.first() {
            match token.kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
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
