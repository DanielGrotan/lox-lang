use std::mem::discriminant;

use crate::lexer::{Token, TokenKind};

mod expr;
pub use expr::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
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
            TokenKind::LParen => {
                self.bump();
                let expr = self.expression()?;
                self.consume(TokenKind::RParen)?;
                Expr::Grouping {
                    expr: Box::new(expr),
                }
            }
            _ => panic!("unexpected token: {token:?}"),
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

    fn consume(&mut self, expected: TokenKind) -> Option<()> {
        let token = self.first()?;

        if discriminant(&token.kind) == discriminant(&expected) {
            self.bump();
            Some(())
        } else {
            None
        }
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
