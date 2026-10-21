use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct SrcPos {
    pub line: usize,
    pub col: usize,
}

impl SrcPos {
    pub fn new() -> Self {
        Self { line: 1, col: 1 }
    }

    pub fn new_line(&mut self) {
        self.line += 1;
        self.col = 0;
    }
}

impl Display for SrcPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: SrcPos,
    pub end: SrcPos,
}
