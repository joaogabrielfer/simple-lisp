use std::fmt::Display;

use miette::{Diagnostic, NamedSource, Report, Result, SourceOffset, SourceSpan};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
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
    Atom,

    Eof,
}

#[derive(Clone)]
pub struct Lexer {
    pub source: String,
    pub source_path: String,
    pub index: usize,
    peeked: Option<Token>,
}

impl Lexer {
    pub fn new(source_path: String, source: String) -> Self {
        Self {
            source,
            source_path,
            index: 0,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Result<Option<Token>> {
        let tk = self.next_token()?;
        self.peeked = Some(tk.clone());
        if tk.kind == TokenKind::Eof {
            Ok(None)
        } else {
            Ok(Some(tk))
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        match self.source.chars().nth(self.index) {
            Some('(') => {
                self.index += 1;
                Ok(self.single_char_token(TokenKind::OpenParen, '('))
            }
            Some(')') => {
                self.index += 1;
                Ok(self.single_char_token(TokenKind::CloseParen, ')'))
            }
            Some('\n') => {
                self.index += 1;
                self.next_token()
            }
            Some('"') => {
                let start = self.index;
                self.index += 1;
                let mut buffer = String::new();

                while let Some(c) = self.source.chars().nth(self.index) {
                    self.index += 1;
                    if matches!(c, '\n') {
                        return Err(self.error(LexerErrorKind::UnexpectedCharacter(c), self.index));
                    }

                    if matches!(c, '"') {
                        break;
                    }

                    buffer.push(c);
                }

                Ok(self.token(TokenKind::String, buffer, start))
            }
            Some(':') => {
                let start = self.index;
                self.index += 1;
                let mut buffer = String::new();

                while let Some(c) = self.source.chars().nth(self.index) {
                    if c == ')' || c == '(' {
                        break;
                    }

                    if c.is_whitespace() {
                        break;
                    }
                    if c == '\n' {
                        break;
                    }

                    if c == '"' {
                        return Err(
                            self.error(LexerErrorKind::UnexpectedCharacter('"'), self.index)
                        );
                    }

                    buffer.push(c);
                    self.index += 1;
                }

                Ok(self.token(TokenKind::Atom, buffer, start))
            }
            Some(';') => {
                while self
                    .source
                    .chars()
                    .nth(self.index)
                    .is_none_or(|c| !matches!(c, '\n'))
                {
                    self.index += 1;
                }
                self.next_token()
            }
            Some(c) if c.is_whitespace() => {
                self.index += 1;
                self.next_token()
            }
            Some(c) => {
                let start = self.index;
                self.index += 1;
                let mut buffer = String::from(c);

                while let Some(c) = self.source.chars().nth(self.index) {
                    if c == ')' || c == '(' || c.is_whitespace() || c == '\n' {
                        break;
                    }

                    if c == '"' {
                        return Err(
                            self.error(LexerErrorKind::UnexpectedCharacter('"'), self.index)
                        );
                    }

                    self.index += 1;
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
                Ok(self.single_char_token(TokenKind::Eof, 0 as char))
            }
        }
    }

    fn token(&self, kind: TokenKind, lexeme: String, start: usize) -> Token {
        Token {
            kind,
            lexeme,
            span: SourceSpan::new(SourceOffset::from(start), self.index - start),
        }
    }

    fn single_char_token(&self, kind: TokenKind, lexeme: char) -> Token {
        self.token(kind, lexeme.to_string(), self.index - 1)
    }

    fn error(&self, kind: LexerErrorKind, start: usize) -> Report {
        LexerError {
            kind,
            src: NamedSource::new(self.source_path.clone(), self.source.clone()),
            span: SourceSpan::new(SourceOffset::from(start), self.index - start),
        }
        .into()
    }
}

impl Iterator for Lexer {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tk) = self.peeked.take() {
            return Some(Ok(tk));
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

#[derive(Debug, Error, Diagnostic, Clone)]
#[error("{kind}")]
#[diagnostic(code(interpreter::lexer))]
pub struct LexerError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("error occurred here")]
    pub span: SourceSpan,

    pub kind: LexerErrorKind,
}

#[derive(Debug, Error, PartialEq, Clone)]
pub enum LexerErrorKind {
    #[error("unexpected char '{0}'")]
    UnexpectedCharacter(char),
}
