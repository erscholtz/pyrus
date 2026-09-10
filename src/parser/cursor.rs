use crate::diagnostic::{CompilerDiagnostic, SourceLocation};

pub(super) struct Cursor {
    pub(super) file: String,
    pub(super) src: String,
    pub(super) line: usize,
    pub(super) col: usize,
    pub(super) offset: usize,
}

pub(super) struct Mark {
    pub(super) line: usize,
    pub(super) col: usize,
    pub(super) offset: usize,
}

impl Cursor {
    /// Creates new cursor given a filename
    ///
    /// Results in fatal compiler error if filename does not match file on disk
    pub(super) fn new(file: String, src: String) -> Self {
        Self {
            file,
            src,
            line: 1,
            col: 1,
            offset: 0,
        }
    }

    /// returns the current char the cursor is on
    pub(super) fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.offset).copied()
    }

    /// returns the char in front of the current char the cursor is on
    pub(super) fn peek_next(&self) -> Option<u8> {
        self.src.as_bytes().get(self.offset + 1).copied()
    }

    /// Returns the byte immediately before the current position.
    pub(super) fn peek_previous(&self) -> Option<u8> {
        self.offset
            .checked_sub(1)
            .and_then(|offset| self.src.as_bytes().get(offset).copied())
    }

    pub(super) fn peek_char(&self) -> Option<char> {
        self.src.get(self.offset..)?.chars().next()
    }

    /// advances the cursor to the next char in the file
    pub(super) fn advance(&mut self) -> Result<(), CompilerDiagnostic> {
        let cur = self.peek_char().ok_or_else(|| {
            CompilerDiagnostic::Syntax(
                crate::diagnostic::SyntaxError::UnexpectedEof {
                    location: self.location(),
                    expected: "a source character".to_string(),
                },
            )
        })?;

        self.offset += cur.len_utf8();
        match cur {
            '\r' => {
                if self.peek() == Some(b'\n') {
                    self.offset += 1;
                }
                self.line += 1;
                self.col = 1;
            }
            '\n' => {
                self.line += 1;
                self.col = 1;
            }
            _ => self.col += 1,
        }

        Ok(())
    }

    /// source location of the current location of the cursor
    pub(super) fn location(&self) -> SourceLocation {
        SourceLocation {
            line: self.line,
            column: self.col,
            file: self.file.clone(),
        }
    }

    /// creates a mark for a location, this is useful for marking the start of
    /// a token
    pub(super) fn mark(&self) -> Mark {
        Mark {
            line: self.line,
            col: self.col,
            offset: self.offset,
        }
    }
}
