use std::rc::Rc;

use crate::try_next;
use crate::monkey::error::Error;
use crate::monkey::parser::ast::{Identifier, StatementNode};
use crate::monkey::parser::expression::OrderOfOps;
use crate::monkey::parser::Parser;
use crate::monkey::Result;
use crate::monkey::token::TokenType;

impl <'a> Parser<'a> {
    pub fn parse_let(&mut self) -> Result<StatementNode> {
        let let_token = Rc::clone(&self.current());
        self.next();

        let identifier = Identifier(self.current().literal.clone());
        self.next();

        try_next!(self, TokenType::Assign);

        let expression = self.parse_expression_node(OrderOfOps::Lowest)?;

        self.next();

        Ok(
            StatementNode::Let(
                identifier, expression,
            )
        )
    }

    pub fn parse_if(&mut self) -> Result<StatementNode> {
        let token = Rc::clone(&self.token_current);

        self.next();

        try_next!(self, TokenType::LeftParen);

        let expression = self.parse_expression_node(OrderOfOps::Lowest)?;

        self.next();

        try_next!(self, TokenType::RightParen);

        try_next!(self, TokenType::LeftBrace);

        let consequence = self.parse_program()?;

        try_next!(self, TokenType::RightBrace);

        let alternative = if self.current().token_type == TokenType::Else {
            self.next();

            try_next!(self, TokenType::LeftBrace);

            match self.parse_program() {
                Ok(result) => {
                    self.next();
                    Some(result)
                }
                Err(err) => return Err(err),
            }
        } else {
            None
        };

        Ok(
            StatementNode::If {
                condition: expression,
                consequence,
                alternative,
            }
        )
    }

    pub fn parse_return(&mut self) -> Result<StatementNode> {
        self.next();

        let expression = self.parse_expression_node(OrderOfOps::Lowest)?;

        self.next();

        Ok(StatementNode::Return(expression))
    }
}

