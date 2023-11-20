use std::cell::Cell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;
use crate::monkey::error::Error::{InvalidTokenError, TodoError, InvalidCharError};
use crate::monkey::Result;

use crate::monkey::helper::{is_digit, is_identifier, is_whitespace};
use crate::monkey::token::{Span, Token, TokenType};

pub trait Lexer {
    fn token(&self) -> Result<Token>;
}

pub struct MonkeyLexer {
    /// Input
    input: Vec<char>,

    column: Cell<usize>,
    position: Cell<usize>,
    read_position: Cell<usize>,
    line: Cell<usize>,
    ch: Cell<char>,
}

impl Lexer for MonkeyLexer {
    fn token(&self) -> Result<Token> {
        self.read()
    }
}


impl MonkeyLexer {
    pub fn new(input: &str) -> Self {
        let input: Vec<char> = input.chars().collect();
        let ch: char = input[0];
        Self {
            input,
            column: Cell::new(0),
            position: Cell::new(0),
            read_position: Cell::new(1),
            line: Cell::new(0),
            ch: Cell::new(ch.into()),
        }
    }

    fn peek(&self) -> char {
        self.input[self.read_position.get()]
    }

    fn handle_whitespace(&self) {
        while is_whitespace(self.ch.get()) {
            self.next()
        }
    }

    fn handle_single_token(&self, token_type: TokenType) -> Result<Token> {
        let token = self.tokenize(1, 0, token_type);
        self.next();

        token
    }

    fn handle_double_token(&self, token_type: TokenType) -> Result<Token> {
        let token = self.tokenize(2, 0, token_type);
        self.next();
        self.next();

        token
    }

    fn read_operator_double(&self, token_type: TokenType) -> Result<Token> {
        if self.peek() == '=' {
            match self.ch.get() {
                '=' => self.handle_double_token(TokenType::Equal),
                '!' => self.handle_double_token(TokenType::NotEqual),
                '<' => self.handle_double_token(TokenType::GreaterThanEqual),
                '>' => self.handle_double_token(TokenType::LessThanEqual),
                c @ _ => Err(TodoError(format!("unsupported token {}", c))),
            }
        } else {
            self.handle_single_token(token_type)
        }
    }

    fn read(&self) -> Result<Token> {
        self.handle_whitespace();
        match self.ch.get() {
            '=' => self.read_operator_double(TokenType::Assign),
            '+' => self.handle_single_token(TokenType::Plus),
            '-' => self.handle_single_token(TokenType::Minus),
            '!' => self.read_operator_double(TokenType::Bang),
            '*' => self.handle_single_token(TokenType::Asterisk),
            '/' => self.handle_single_token(TokenType::Slash),
            '<' => self.read_operator_double(TokenType::LessThan),
            '>' => self.read_operator_double(TokenType::GreaterThan),
            ';' => self.handle_single_token(TokenType::Semicolon),
            '(' => self.handle_single_token(TokenType::LeftParen),
            ')' => self.handle_single_token(TokenType::RightParen),
            ',' => self.handle_single_token(TokenType::Comma),
            '{' => self.handle_single_token(TokenType::LeftBrace),
            '}' => self.handle_single_token(TokenType::RightBrace),
            '[' => self.handle_single_token(TokenType::LeftBracket),
            ']' => self.handle_single_token(TokenType::RightBracket),
            ':' => self.handle_single_token(TokenType::Colon),
            '"' => self.handle_string_token(),
            '\0' => self.tokenize(0, 0, TokenType::EndOfFile),
            ch if is_identifier(ch) => self.read_identifier(),
            ch if is_digit(ch) => self.read_digit(),
            c @ _ => Err(TodoError(format!("unsupported token {}", c))),
        }
    }

    fn read_identifier(&self) -> Result<Token> {
        let position = self.position.get();
        while is_identifier(self.ch.get()) {
            self.next();
        }
        let end_position = self.position.get();
        let length = end_position - position;

        let identifier: &str = &self.input[position..end_position].iter().collect::<String>();
        match identifier {
            "let" => self.tokenize(length, length, TokenType::Let),
            "fn" => self.tokenize(length, length, TokenType::Function),
            "true" => self.tokenize(length, length, TokenType::True),
            "false" => self.tokenize(length, length, TokenType::False),
            "if" => self.tokenize(length, length, TokenType::If),
            "else" => self.tokenize(length, length, TokenType::Else),
            "return" => self.tokenize(length, length, TokenType::Return),
            _ => self.tokenize(length, length, TokenType::Identifier),
        }
    }

    fn read_digit(&self) -> Result<Token> {
        let position = self.position.get();
        while is_digit(self.ch.get()) {
            self.next()
        }
        let end_position = self.position.get();
        let length = end_position - position;

        self.tokenize(length, length, TokenType::Integer)
    }

    /// Move the processing position to the next token
    fn next(&self) {
        if self.input.len() <= self.read_position.get() {
            self.ch.set(0.into());
            self.column.set(self.column.get() + 1);
        } else {
            if self.input[self.position.get()] == '\n' {
                self.line.set(self.line.get() + 1);
                self.column.set(0)
            } else {
                self.column.set(self.column.get() + 1);
            }
            self.ch.set(self.input[self.read_position.get()])
        }

        self.position.set(self.position.get() + 1);
        self.read_position.set(self.read_position.get() + 1);
    }

    fn handle_string_token(&self) -> Result<Token> {
        let position = self.position.get();
        let start = self.column.get();

        self.next();

        let mut chars: Vec<char> = Vec::new();

        while self.ch.get() != '"' && self.ch.get() != '\0' {
            if self.ch.get() == '\\' {
                self.next();
                match self.ch.get() {
                    'n' => chars.push('\n'),
                    't' => chars.push('\t'),
                    '"' => chars.push('"'),
                    '\\' => chars.push('\\'),
                    val @ _ => return Err(InvalidCharError(self.ch.get())),
                }
            } else {
                chars.push(self.ch.get())
            }
            self.next()
        }

        let end_position = self.position.get();
        let length = end_position - position;
        let end = self.column.get();

        self.next();

        // FIXME: there is some code duplication here - some how merge this with tokenizer.
        Ok(Token {
            token_type: TokenType::String,
            literal: chars.iter().collect::<String>(),
            span: Span {
                start,
                end,
                line_start: self.line.get(),
                line_end: self.line.get(),
            },
        })
    }

    fn tokenize(&self, length: usize, offset: usize, token_type: TokenType) -> Result<Token> {
        let position = self.position.get() - offset;

        let literal = if length > 0 {
            self.input[position..position + length].to_vec()
        } else {
            vec![]
        };

        let column = self.column.get() - offset;
        let end = if length > 0 { column + length - 1 } else { column };
        Ok(Token {
            token_type,
            literal: literal.iter().collect::<String>(),
            span: Span {
                start: column,
                end,
                line_start: self.line.get(),
                line_end: self.line.get(),
            },
        })
    }
}

pub fn run_lexer(source: &str) -> Result<String> {
    let contents = fs::read_to_string(source).unwrap();
    eval_lexer(&contents)
}

pub fn eval_lexer(contents: &String) -> Result<String> {
    let lines: Vec<usize> = contents
        .chars()
        .enumerate()
        .filter(|(_, ch)| *ch == '\n')
        .map(|(idx, _)| idx)
        .collect();

    let lexer = MonkeyLexer::new(&*contents);

    let mut tokens = VecDeque::new();
    let mut stop_loop = false;
    while !stop_loop {
        let result = lexer.token();
        if let Ok(token) = result {
            stop_loop = token.token_type == TokenType::EndOfFile;
            tokens.push_back(token);
        } else if let Err(message) = result {
            return Err(message);
        }
    }

    let snapshot = token_snapshot(&contents, &lines, &tokens);

    Ok(snapshot)
}

pub fn token_snapshot(program: &String, lines: &Vec<usize>, tokens: &VecDeque<Token>) -> String {
    let mut output = String::new();
    let mut working_line = usize::MAX;
    for token in tokens {
        if token.span.line_start != token.span.line_end {
            panic!("multiline command not supported 😞")
        }

        if token.span.line_start != working_line {
            working_line = token.span.line_start;
            let start = if working_line != 0 {
                lines[working_line - 1]
            } else {
                0
            };

            let end = if working_line < lines.len() {
                lines[working_line]
            } else {
                program.len() - 1
            };
            output += &program[start..=end];
        }

        output += &" ".repeat(token.span.start);
        output += &"^".repeat(token.span.end + 1 - token.span.start);
        output += &format!(" {token:?}");
        output += &"\n";
    }

    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lexer() {
        let result = run_lexer("monkey/lexer.mky").unwrap();

        insta::assert_snapshot!(result)
    }
}
