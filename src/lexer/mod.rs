// Widow Programming Language
// Lexer module for tokenizing source code

use crate::error::{Result, WidowError, Location};
use logos::Logos;
use std::fmt;

/// Token types for the Widow language
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\f]+")]  // Skip whitespace
#[logos(skip r"#.*")]       // Skip comments
pub enum TokenKind {
    // Keywords
    #[token("func")]
    Func,
    
    #[token("if")]
    If,
    
    #[token("elif")]
    Elif,
    
    #[token("else")]
    Else,
    
    #[token("for")]
    For,
    
    #[token("in")]
    In,
    
    #[token("break")]
    Break,
    
    #[token("continue")]
    Continue,
    
    #[token("ret")]
    Ret,
    
    #[token("struct")]
    Struct,
    
    #[token("impl")]
    Impl,
    
    #[token("switch")]
    Switch,
    
    #[token("case")]
    Case,
    
    #[token("default")]
    Default,
    
    #[token("const")]
    Const,
    
    #[token("nil")]
    Nil,
    
    #[token("true")]
    True,
    
    #[token("false")]
    False,
    
    // Operators
    #[token("+")]
    Plus,
    
    #[token("-")]
    Minus,
    
    #[token("*")]
    Star,
    
    #[token("/")]
    Slash,
    
    #[token("%")]
    Percent,
    
    #[token("=")]
    Assign,
    
    #[token("==")]
    Equal,
    
    #[token("!=")]
    NotEqual,
    
    #[token("<")]
    Less,
    
    #[token("<=")]
    LessEqual,
    
    #[token(">")]
    Greater,
    
    #[token(">=")]
    GreaterEqual,
    
    #[token("&&")]
    And,
    
    #[token("||")]
    Or,
    
    #[token("!")]
    Not,
    
    #[token(".")]
    Dot,
    
    #[token("..")]
    DotDot,
    
    #[token(":")]
    Colon,
    
    // Delimiters
    #[token(",")]
    Comma,
    
    #[token("{")]
    LeftBrace,
    
    #[token("}")]
    RightBrace,
    
    #[token("[")]
    LeftBracket,
    
    #[token("]")]
    RightBracket,
    
    #[token("(")]
    LeftParen,
    
    #[token(")")]
    RightParen,
    
    // Literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLiteral(i64),
    
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let slice = lex.slice();
        // Remove the quotes
        let content = &slice[1..slice.len() - 1];
        // Process escape sequences
        process_string_literal(content)
    })]
    StringLiteral(String),
    
    #[regex(r"'.'|'\\[ntr\\']'", |lex| {
        let slice = lex.slice();
        // Remove the quotes
        let content = &slice[1..slice.len() - 1];
        // Process the character
        if content.starts_with('\\') {
            match &content[1..] {
                "n" => Some('\n'),
                "t" => Some('\t'),
                "r" => Some('\r'),
                "\\" => Some('\\'),
                "'" => Some('\''),
                _ => Some('?'), // Default for invalid escape
            }
        } else {
            Some(content.chars().next().unwrap())
        }
    })]
    CharLiteral(char),
    
    // Whitespace handling
    #[token("\n")]
    Newline,
    
    // Error handling - this will catch any unmatched token
    Error,
}

/// Full token with kind and position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The type of token
    pub kind: TokenKind,
    /// Line number in source (1-based)
    pub line: usize,
    /// Column number in source (1-based)
    pub column: usize,
    /// Offset in source
    pub offset: usize,
    /// Length of the token in bytes
    pub length: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {}:{}", self.kind, self.line, self.column)
    }
}

/// Process string literals and handle escape sequences
fn process_string_literal(s: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = s.chars();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some(c) => result.push(c), // Just include the character
                None => return None,       // Error: string ends with escape
            }
        } else {
            result.push(c);
        }
    }
    
    Some(result)
}

/// Tokenize source code into a vector of tokens
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut lexer = TokenKind::lexer(source);
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut line_start = 0;
    
    while let Some(token_result) = lexer.next() {
        let span = lexer.span();
        let column = span.start - line_start + 1;
        
        match token_result {
            Ok(TokenKind::Newline) => {
                // Track line numbers for better error messages
                line += 1;
                line_start = span.end;
                
                // Add the newline token
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    line,
                    column: 1,
                    offset: span.start,
                    length: span.end - span.start,
                });
            }
            Ok(TokenKind::Error) => {
                // Handle explicit error token
                let error_text = &source[span.start..span.end];
                
                return Err(WidowError::Lexer {
                    line,
                    column,
                    message: format!("Invalid token: '{}'", error_text),
                });
            }
            Ok(kind) => {
                tokens.push(Token {
                    kind,
                    line,
                    column,
                    offset: span.start,
                    length: span.end - span.start,
                });
            }
            Err(_) => {
                // Handle lexer errors
                let error_text = &source[span.start..span.end];
                
                return Err(WidowError::Lexer {
                    line,
                    column,
                    message: format!("Invalid token: '{}'", error_text),
                });
            }
        }
    }
    
    Ok(tokens)
}

/// Get the location in the source code from a token
pub fn get_location(token: &Token) -> Location {
    Location::new(token.line, token.column)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_basic_tokens() {
        let source = "x = 5";
        let tokens = tokenize(source).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].kind, TokenKind::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[1].kind, TokenKind::Assign));
        assert!(matches!(tokens[2].kind, TokenKind::IntLiteral(5)));
    }
    
    #[test]
    fn test_tokenize_keywords() {
        let source = "func if else ret";
        let tokens = tokenize(source).unwrap();
        
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].kind, TokenKind::Func));
        assert!(matches!(tokens[1].kind, TokenKind::If));
        assert!(matches!(tokens[2].kind, TokenKind::Else));
        assert!(matches!(tokens[3].kind, TokenKind::Ret));
    }
    
    #[test]
    fn test_tokenize_string_literal() {
        let source = "\"hello\\nworld\"";
        let tokens = tokenize(source).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::StringLiteral(ref s) if s == "hello\nworld"));
    }
}