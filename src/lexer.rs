use crate::source::Span;
use crate::source::SrcPos;
use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub pos: Span,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)?;
        write!(f, "(\"")?;
        write!(f, "{}", self.lexeme)?;
        write!(f, "\")")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    OpenParen,  // (
    CloseParen, // )

    Ident,

    Number,
    String,

    Eof,
}

#[derive(Clone)]
pub struct Lexer {
    source: String,
    pub source_path: String,
    index: usize,
    pub pos: SrcPos,
    peeked: Option<Result<Token, LexerError>>,
}

impl Lexer {
    pub fn new(source_path: String, source: String) -> Self {
        Self {
            source,
            source_path,
            index: 0,
            pos: SrcPos::new(),
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<Result<Token, LexerError>> {
        let tk = self.next_token();
        self.peeked = Some(tk.clone());
        if let Ok(ref tk) = tk
            && tk.kind == TokenKind::Eof
        {
            None
        } else {
            Some(tk)
        }
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        match self.source.chars().nth(self.index) {
            Some('(') => {
                self.index += 1;
                self.pos.col += 1;
                Ok(self.single_char_token(TokenKind::OpenParen, '('))
            }
            Some(')') => {
                self.index += 1;
                self.pos.col += 1;
                Ok(self.single_char_token(TokenKind::CloseParen, ')'))
            }
            Some('\n') => {
                self.index += 1;
                self.pos.new_line();
                self.next_token()
            }
            Some('"') => {
                self.index += 1;
                self.pos.col += 1;
                let mut buffer = String::new();
                let start = self.pos;

                while let Some(c) = self.source.chars().nth(self.index) {
                    self.index += 1;
                    self.pos.col += 1;
                    if matches!(c, '\n') {
                        return Err(self.error(LexerErrorKind::UnexpectedCharacter(c), self.pos));
                    }

                    self.pos.col += 1;
                    if matches!(c, '"') {
                        break;
                    }

                    buffer.push(c);
                }

                Ok(self.token(TokenKind::String, buffer, start))
            }
            Some(c) if c.is_whitespace() => {
                self.index += 1;
                self.pos.new_line();
                self.next_token()
            }
            Some(c) => {
                self.index += 1;
                self.pos.col += 1;
                let mut buffer = String::from(c);
                let start = self.pos;

                while let Some(c) = self.source.chars().nth(self.index) {
                    if c == ')' || c == '(' {
                        break;
                    }

                    self.index += 1;
                    if c.is_whitespace() {
                        break;
                    }
                    if c == '\n' {
                        self.pos.new_line();
                        break;
                    }

                    self.pos.col += 1;

                    if c == '"' {
                        return Err(self.error(LexerErrorKind::UnexpectedCharacter('"'), self.pos));
                    }

                    buffer.push(c);
                }

                if buffer.clone().parse::<i32>().is_ok() {
                    Ok(self.token(TokenKind::Number, buffer, start))
                } else {
                    Ok(self.token(TokenKind::Ident, buffer, start))
                }
            }
            None => {
                self.index += 1;
                self.pos.col += 1;
                Ok(self.single_char_token(TokenKind::Eof, 0 as char))
            }
        }
    }

    fn token(&self, kind: TokenKind, lexeme: String, start: SrcPos) -> Token {
        Token {
            kind,
            lexeme,
            pos: Span {
                start,
                end: self.pos,
            },
        }
    }

    fn single_char_token(&self, kind: TokenKind, lexeme: char) -> Token {
        self.token(kind, lexeme.to_string(), self.pos)
    }

    fn error(&self, kind: LexerErrorKind, start: SrcPos) -> LexerError {
        LexerError {
            kind,
            source_path: self.source_path.clone(),
            pos: Span {
                start,
                end: self.pos,
            },
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tk) = self.peeked.take() {
            return Some(tk);
        }
        let tk = self.next_token();
        if let Ok(ref tk) = tk
            && tk.kind == TokenKind::Eof
        {
            None
        } else {
            Some(tk)
        }
    }
}

#[derive(Error, Debug, Clone)]
#[error("{}:{}: {}",.source_path, .kind, .pos.start)]
pub struct LexerError {
    kind: LexerErrorKind,
    source_path: String,
    pos: Span,
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum LexerErrorKind {
    #[error("unexpected char `{}`", .0)]
    UnexpectedCharacter(char),
}
