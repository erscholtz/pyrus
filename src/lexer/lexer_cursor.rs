use crate::diagnostic::{CompilerDiagnostic, SourceLocation};

pub struct Cursor {
    pub file: String,
    pub src: String,
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

pub struct Mark {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl Cursor {
    /// Creates new cursor given a filename
    ///
    /// Results in fatal compiler error if filename does not match file on disk
    pub fn new(file: String, src: String) -> Self {
        Self {
            file,
            src,
            line: 1,
            col: 1,
            offset: 0,
        }
    }

    /// returns the current char the cursor is on
    pub fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.offset).copied()
    }

    /// returns the char in front of the current char the cursor is on
    pub fn peek_next(&self) -> Option<u8> {
        self.src.as_bytes().get(self.offset + 1).copied()
    }

    /// advances the cursor to the next char in the file
    pub fn advance(&mut self) -> Result<(), CompilerDiagnostic> {
        let cur = match self.peek() {
            Some(c) => c,
            None => {
                return {
                    Err(CompilerDiagnostic::Syntax(
                        crate::diagnostic::SyntaxError::UnexpectedEof {
                            location: self.location(),
                            expected: "expected next token, found None"
                                .to_string(),
                        },
                    ))
                };
            }
        };

        if cur == b'\n' {
            self.offset += 1;
            self.line += 1;
            self.col = 1;
        } else {
            self.offset += 1;
            self.col += 1;
        }
        Ok(()) // NOTE this is not correct to just return nothing I think
    }

    /// source location of the current location of the cursor
    pub fn location(&self) -> SourceLocation {
        SourceLocation {
            line: self.line,
            column: self.col,
            file: self.file.clone(),
        }
    }
    
    /// creates a mark for a location, this is useful for marking the start of
    /// a token
    pub fn mark(&self) -> Mark {
        Mark {
            line: self.line,
            col: self.col,
            offset: self.offset,
        }
    }
}
