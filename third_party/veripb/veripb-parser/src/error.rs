//! Definitions of parsing errors.

use std::ops::Range;

use thiserror::Error;

use crate::pbp_parser::utils::Position;

/// An error from the VeriPB parsing library.
#[derive(Debug, Error)]
pub enum ParserError {
    /// Error related to reading the file.
    #[error("Failed to open file!")]
    IOError(
        #[from]
        #[source]
        std::io::Error,
    ),

    #[error("Failed to open file! File '{0}' might not exist?")]
    FileError(String),

    #[error("Unexpected token starting at position {}! Expected: {expected}!", .span.start + 1)]
    TokenError {
        span: Range<usize>,
        expected: String,
    },

    #[error("Unexpected token starting at {filename}:{}:{}! Expected {expected}!", .line+1, .span.start + 1, )]
    TokenErrorFile {
        span: Range<usize>,
        expected: String,
        line: usize,
        filename: String,
    },

    #[error{"Constraint ID parsing error!"}]
    ParseIntError(
        #[from]
        #[source]
        std::num::ParseIntError,
    ),

    #[error("The file did not contain a header, which is required for this input file format!")]
    NoHeader,

    #[error("{message} at line {} col {}.\n{code}", .pos.line, .pos.col)]
    ProofFormatError {
        message: String,
        pos: Position,
        code: String,
    },
}

impl ParserError {
    pub fn is_version_error(&self) -> bool {
        matches!(
            self,
            ParserError::ProofFormatError {
                pos: Position {
                    pos: 0,
                    col: 1,
                    line: 0,
                },
                ..
            }
        )
    }

    pub fn token_error(span: Range<usize>, expected: &str) -> Self {
        Self::TokenError {
            span,
            expected: expected.to_string(),
        }
    }

    pub fn token_error_with_file(
        span: Range<usize>,
        expected: &str,
        filename: String,
        line: usize,
    ) -> Self {
        Self::TokenErrorFile {
            span,
            expected: expected.to_string(),
            line,
            filename,
        }
    }

    pub fn add_file_and_line(self, filename: String, line: usize) -> Option<Self> {
        match self {
            Self::TokenError { span, expected } => Some(Self::TokenErrorFile {
                span,
                expected,
                line,
                filename,
            }),
            _ => None,
        }
    }

    pub fn pbp_error(message: String, pos: Position, len: usize, line: String) -> ParserError {
        if line.is_empty() {
            Self::ProofFormatError {
                message,
                pos,
                code: String::new(),
            }
        } else {
            Self::ProofFormatError {
                message,
                pos,
                code: format!(
                    "{}\n{}{}",
                    line,
                    " ".repeat(std::cmp::max(line.len(), 1) - len),
                    "^".repeat(std::cmp::max(len, 1)),
                ),
            }
        }
    }
}
