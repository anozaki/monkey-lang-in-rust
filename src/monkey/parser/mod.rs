use std::rc::Rc;

use crate::monkey::lexer::Lexer;
use crate::monkey::parser::ast::{Program, StatementNode};
use crate::monkey::Result;
use crate::monkey::token::{Token, TokenType};
use crate::monkey::token::TokenType::Semicolon;

pub mod parser;
pub mod expression;
pub mod ast;


#[macro_export]
macro_rules! try_next {
    ($self: ident, $token: expr) => {
        if $self.current().token_type != $token {
            return Err(Error::InvalidTokenError(Rc::clone($self.current())));
        }
        $self.next();
    }
}

fn next_token(lexer: &dyn Lexer) -> Token {
    match lexer.token() {
        Ok(token) => token,
        Err(message) => panic!("Unable to parse: {message}")
    }
}

pub struct Parser<'a> {
    lexer: &'a dyn Lexer,

    token_current: Rc<Token>,
    token_peek: Rc<Token>,
}

impl <'a> Parser <'a> {
    pub fn new(lexer: &'a dyn Lexer) -> Self {
        let token_current = Rc::new(next_token(lexer));
        let token_peek = Rc::new(next_token(lexer));
        Parser {
            lexer,
            token_current,
            token_peek,
        }
    }

    pub fn current(&self) -> &Rc<Token> {
        return &self.token_current;
    }

    pub fn peek(&self) -> &Rc<Token> {
        return &self.token_peek;
    }

    pub fn next(&mut self) {
        self.token_current = Rc::clone(&self.token_peek);
        self.token_peek = Rc::new(next_token(self.lexer));
    }

    pub fn skip_semicolon(&mut self) {
        while self.current().token_type == Semicolon {
            self.next()
        }
    }

    pub fn parse_program(&mut self) -> Result<Box<Program>> {
        let mut program: Box<Program> = Box::new(Program::default());

        while self.current().token_type != TokenType::EndOfFile && self.current().token_type != TokenType::RightBrace {
            program.statements.push(self.parse_statement()?);
            self.skip_semicolon();
        }

        Ok(program)
    }

    pub fn parse_statement(&mut self) -> Result<StatementNode> {
        Ok(match self.current().token_type {
            TokenType::Let => self.parse_let()?,
            TokenType::Return => self.parse_return()?,
            TokenType::If => self.parse_if()?,
            _ => self.parse_expression()?,
        })
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use crate::monkey::lexer::MonkeyLexer;

    use super::*;

    pub fn evaluate(source: &str) -> String {
        let contents = fs::read_to_string(source).unwrap();
        let lexer = MonkeyLexer::new(&contents);

        let mut parser = Parser::new(&lexer);
        let program = parser.parse_program().unwrap();

        let mut output = String::new();
        for statement in &program.statements {
            output += &format!("{:?}\n", statement);
        }

        output
    }

    macro_rules! evaluate {
        ($name:ident, $file:literal) => {
            #[test]
            fn $name() {
                let output = evaluate(&format!("monkey/{}", $file));
                insta::assert_snapshot!(output)
            }
        }
    }

    evaluate!(test_parse_let, "test_parser_let.mky");
    evaluate!(test_parse_return, "test_parser_return.mky");
    evaluate!(test_parser_infix, "test_parser_infix.mky");
    evaluate!(test_parser_expression, "test_parser_expression.mky");
    evaluate!(test_parser_expression_prec, "test_parser_expression_prec.mky");
    evaluate!(test_parser_if, "test_parser_if.mky");
    evaluate!(test_parser_function_literals, "test_parser_function_literals.mky");
    evaluate!(test_parser_closure, "test_parser_closure.mky");
    evaluate!(test_parser_string, "test_parser_string.mky");
    evaluate!(test_parser_index, "test_parser_index.mky");
    evaluate!(test_parser_hash, "test_parser_hash.mky");
}
