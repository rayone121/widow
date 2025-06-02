// Widow Programming Language
// Error module for custom error types

use std::fmt;
use std::io;
use thiserror::Error;

/// Main error type for the Widow language
#[derive(Error, Debug)]
pub enum WidowError {
    /// Lexical analysis errors
    #[error("Lexer error at line {line}, column {column}: {message}")]
    Lexer {
        line: usize,
        column: usize,
        message: String,
    },

    /// Parser errors
    #[error("Parser error at line {line}, column {column}: {message}")]
    Parser {
        line: usize,
        column: usize,
        message: String,
    },

    /// Type checking errors
    #[error("Type error at line {line}, column {column}: {message}")]
    Type {
        line: usize,
        column: usize,
        message: String,
    },

    /// Semantic analysis errors
    #[error("Semantic error at line {line}, column {column}: {message}")]
    Semantic {
        line: usize,
        column: usize,
        message: String,
    },

    /// Runtime errors during execution
    #[error("Runtime error: {message}")]
    Runtime { message: String },

    /// File I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Generic errors
    #[error("{0}")]
    Generic(String),
}

/// Source code location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// A result type alias for Widow operations
pub type Result<T> = std::result::Result<T, WidowError>;