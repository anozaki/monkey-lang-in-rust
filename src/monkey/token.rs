use std::fmt::{Debug, Formatter};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    Illegal,
    EndOfFile,

    Comment,

    String,

    // Identifier/Literals
    Identifier,
    Integer,

    // Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,

    LessThan,
    GreaterThan,

    Equal,
    NotEqual,
    LessThanEqual,
    GreaterThanEqual,

    // Delimiters
    Comma,
    Colon,
    Semicolon,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    // Keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

#[derive(PartialEq, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line_start: usize,
    pub line_end: usize,
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "[L{}-{:?}:{:?}]",
            self.line_start, self.start, self.end
        )
    }
}

#[derive(PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,

    pub span: Span,
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "Token({:?}, {:?} {:?})",
            self.token_type, self.literal, self.span
        )
    }
}
