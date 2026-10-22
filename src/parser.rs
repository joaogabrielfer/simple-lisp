use thiserror::Error;

use crate::{
    lexer::{Lexer, TokenKind},
    parser::ParserErrorKind::{UnexpectedEof, UnexpectedToken},
    source::Span,
};

#[derive(Debug, Clone)]
pub enum Expr {
    AtomLit(String),
    NumberLit(i32),
    StringLit(String),
    Ident(String),
    List(Vec<Expr>),
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut exprs = vec![];
        loop {
            match self.parse_expr() {
                Some(Ok(expr)) => exprs.push(expr),
                Some(Err(e)) => return Err(e),
                None => return Ok(exprs),
            }
        }
    }

    fn parse_expr(&mut self) -> Option<Result<Expr, ParserError>> {
        match self.lexer.next()? {
            Ok(tk) => match tk.kind {
                TokenKind::OpenParen => {
                    let mut exprs = vec![];
                    while let Some(Ok(tk)) = self.lexer.peek()
                        && tk.kind != TokenKind::CloseParen
                    {
                        exprs.push(match self.expect_expr() {
                            Ok(expr) => expr,
                            Err(e) => return Some(Err(e)),
                        })
                    }
                    self.expect(TokenKind::CloseParen);
                    Some(Ok(Expr::List(exprs)))
                }
                TokenKind::CloseParen => Some(Err(ParserError {
                    kind: UnexpectedToken(TokenKind::CloseParen),
                    source_path: self.lexer.source_path.clone(),
                    pos: tk.pos,
                })),
                TokenKind::Ident => Some(Ok(Expr::Ident(tk.lexeme))),
                TokenKind::Number => Some(Ok(Expr::NumberLit(tk.lexeme.parse().unwrap()))),
                TokenKind::String => Some(Ok(Expr::StringLit(tk.lexeme))),
                TokenKind::Eof => None,
            },
            Err(e) => todo!(),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Result<(), ParserError>> {
        Some(match self.lexer.next() {
            Some(Ok(tk)) if tk.kind == kind => Ok(()),
            Some(Ok(tk)) => Err(ParserError {
                kind: ParserErrorKind::UnexpectedToken(tk.kind),
                source_path: self.lexer.source_path.clone(),
                pos: tk.pos,
            }),
            Some(Err(_e)) => todo!(),
            None => Err(ParserError {
                kind: ParserErrorKind::UnexpectedEof,
                source_path: self.lexer.source_path.clone(),
                pos: Span {
                    start: self.lexer.pos,
                    end: self.lexer.pos,
                },
            }),
        })
    }

    fn expect_expr(&mut self) -> Result<Expr, ParserError> {
        match self.parse_expr() {
            Some(e) => e,
            None => Err(ParserError {
                kind: UnexpectedEof,
                source_path: self.lexer.source_path.clone(),
                pos: Span {
                    start: self.lexer.pos,
                    end: self.lexer.pos,
                },
            }),
        }
    }
}

#[derive(Error, Debug)]
#[error("{}:{}: {}",.source_path, .kind, .pos.start)]
pub struct ParserError {
    kind: ParserErrorKind,
    source_path: String,
    pos: Span,
}

#[derive(Debug, Error, PartialEq)]
pub enum ParserErrorKind {
    #[error("unexpected token `{:?}`", .0)]
    UnexpectedToken(TokenKind),
    #[error("unexpected eof")]
    UnexpectedEof,
}
