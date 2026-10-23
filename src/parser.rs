use miette::{Diagnostic, NamedSource, Report, Result, SourceOffset, SourceSpan};
use thiserror::Error;

use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

impl Expr {
    fn new(kind: ExprKind, span: SourceSpan) -> Self {
        Expr { kind, span }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    AtomLit(String),
    NumberLit(i32),
    StringLit(String),
    Ident(String),
    List(Vec<Expr>),
}

pub struct Parser {
    pub lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) -> Result<Vec<Expr>> {
        let mut exprs = vec![];
        loop {
            match self.parse_expr() {
                Some(Ok(expr)) => exprs.push(expr),
                Some(Err(e)) => return Err(e),
                None => return Ok(exprs),
            }
        }
    }

    fn parse_expr(&mut self) -> Option<Result<Expr>> {
        match self.lexer.next()? {
            Ok(tk) => match tk.kind {
                TokenKind::OpenParen => {
                    let start = tk.span.offset();
                    let mut exprs = vec![];
                    while let Ok(Some(tk)) = self.lexer.peek()
                        && tk.kind != TokenKind::CloseParen
                    {
                        exprs.push(match self.expect_expr() {
                            Ok(expr) => expr,
                            Err(e) => return Some(Err(e)),
                        })
                    }
                    if let Some(Err(e)) = self.expect(TokenKind::CloseParen) {
                        return Some(Err(e));
                    }
                    Some(Ok(self.expr(ExprKind::List(exprs), start)))
                }
                TokenKind::CloseParen => Some(Err(self.token_error(tk))),
                TokenKind::Ident => Some(Ok(Expr::new(ExprKind::Ident(tk.lexeme), tk.span))),
                TokenKind::Number => Some(Ok(Expr::new(
                    ExprKind::NumberLit(tk.lexeme.parse().unwrap()),
                    tk.span,
                ))),
                TokenKind::String => Some(Ok(Expr::new(ExprKind::StringLit(tk.lexeme), tk.span))),
                TokenKind::Atom => Some(Ok(Expr::new(ExprKind::AtomLit(tk.lexeme), tk.span))),
                TokenKind::Eof => None,
            },
            Err(e) => Some(Err(e)),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Result<()>> {
        Some(match self.lexer.next() {
            Some(Ok(tk)) if tk.kind == kind => Ok(()),
            Some(Ok(tk)) => Err(self.token_error(tk)),
            Some(Err(e)) => Err(e),
            None => Err(self.eof_error()),
        })
    }

    fn expect_expr(&mut self) -> Result<Expr> {
        match self.parse_expr() {
            Some(e) => e,
            None => Err(self.eof_error()),
        }
    }

    fn eof_error(&self) -> Report {
        ParserError {
            src: NamedSource::new(self.lexer.source_path.clone(), self.lexer.source.clone()),
            span: SourceSpan::new(SourceOffset::from(self.lexer.index - 1), 1),
            kind: ParserErrorKind::UnexpectedEof,
        }
        .into()
    }

    fn token_error(&self, tk: Token) -> Report {
        ParserError {
            src: NamedSource::new(self.lexer.source_path.clone(), self.lexer.source.clone()),
            span: tk.span,
            kind: ParserErrorKind::UnexpectedToken(tk.kind),
        }
        .into()
    }

    fn expr(&self, kind: ExprKind, start: usize) -> Expr {
        Expr {
            kind,
            span: SourceSpan::new(SourceOffset::from(start), self.lexer.index - start),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("{kind}")]
#[diagnostic(code(interpreter::parser))]
pub struct ParserError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("error occurred here")]
    pub span: SourceSpan,

    pub kind: ParserErrorKind,
}

#[derive(Debug, Error, PartialEq)]
pub enum ParserErrorKind {
    #[error("unexpected token `{:?}`", .0)]
    UnexpectedToken(TokenKind),
    #[error("unexpected eof")]
    UnexpectedEof,
}
