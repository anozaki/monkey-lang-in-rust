use std::collections::HashMap;
use std::rc::Rc;

use crate::monkey::error::Error;
use crate::monkey::parser::ast::{ExpressionNode, Identifier, Operator, StatementNode};
use crate::monkey::parser::Parser;
use crate::monkey::Result;
use crate::monkey::token::{Token, TokenType};
use crate::try_next;

#[derive(Copy, Clone)]
pub enum OrderOfOps {
    Invalid,
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Result<StatementNode> {
        let expression = self.parse_expression_node(OrderOfOps::Lowest)?;

        self.next();

        Ok(StatementNode::Expression { expression })
    }

    fn order_of_operation(&self, token_type: TokenType) -> OrderOfOps {
        match token_type {
            TokenType::Equal | TokenType::NotEqual => OrderOfOps::Equals,
            TokenType::LessThan | TokenType::GreaterThan => OrderOfOps::LessGreater,
            TokenType::Plus | TokenType::Minus => OrderOfOps::Sum,
            TokenType::Asterisk | TokenType::Slash => OrderOfOps::Product,
            TokenType::LeftParen => OrderOfOps::Call,
            TokenType::LeftBracket => OrderOfOps::Index,
            _ => OrderOfOps::Invalid,
        }
    }
    pub fn parse_expression_node(&mut self, precedence: OrderOfOps) -> Result<Box<ExpressionNode>> {
        let mut left_result: Box<ExpressionNode> = match self.current().token_type {
            TokenType::Identifier => self.parse_identifier()?,
            TokenType::Integer => self.parse_integer_literal()?,
            TokenType::Bang => self.parse_prefix(Operator::Not)?,
            TokenType::Minus => self.parse_prefix(Operator::Neg)?,
            TokenType::True => self.parse_bool()?,
            TokenType::False => self.parse_bool()?,
            TokenType::LeftParen => self.parse_group()?,
            TokenType::Function => self.parse_fn()?,
            TokenType::String => self.parse_string()?,
            TokenType::LeftBracket => self.parse_array_literal()?,
            TokenType::LeftBrace => self.parse_hash_literal()?,
            _ => return Err(Error::InvalidTokenError(Rc::clone(self.current())))
        };

        let precedence_order = precedence as isize;
        while self.peek().token_type != TokenType::Semicolon && precedence_order < self.order_of_operation(self.peek().token_type) as isize {
            self.next();

            let infix = self.parse_infix_node(left_result)?;

            left_result = infix;
        }

        Ok(left_result)
    }

    pub fn parse_identifier(&self) -> Result<Box<ExpressionNode>> {
        let result = ExpressionNode::Identifier(Identifier(self.current().literal.clone()));
        Ok(Box::new(result))
    }

    fn parse_infix_node(&mut self, left: Box<ExpressionNode>) -> Result<Box<ExpressionNode>> {
        Ok(match self.current().token_type {
            TokenType::Plus => self.parse_infix(left, Operator::Add)?,
            TokenType::Minus => self.parse_infix(left, Operator::Sub)?,
            TokenType::Slash => self.parse_infix(left, Operator::Div)?,
            TokenType::Asterisk => self.parse_infix(left, Operator::Mul)?,
            TokenType::Equal => self.parse_infix(left, Operator::Equal)?,
            TokenType::NotEqual => self.parse_infix(left, Operator::NotEqual)?,
            TokenType::LessThan => self.parse_infix(left, Operator::Less)?,
            TokenType::GreaterThan => self.parse_infix(left, Operator::Greater)?,
            TokenType::LeftParen => self.parse_call(left, Operator::Call)?,
            TokenType::LeftBracket => self.parse_index(left, Operator::Index)?,
            token @ _ => return Err(Error::InvalidTokenError(Rc::clone(self.current())))
        })
    }

    fn parse_prefix(&mut self, operator: Operator) -> Result<Box<ExpressionNode>> {
        let token: Rc<Token> = Rc::clone(self.current());

        self.next();

        let expression = self.parse_expression_node(OrderOfOps::Prefix)?;

        Ok(Box::new(ExpressionNode::Prefix { operator, expression }))
    }

    fn parse_infix(&mut self, left: Box<ExpressionNode>, operator: Operator) -> Result<Box<ExpressionNode>> {
        let token: Rc<Token> = Rc::clone(self.current());
        let precedence = self.order_of_operation(self.current().token_type);

        self.next();

        let right = self.parse_expression_node(precedence)?;

        Ok(Box::new(ExpressionNode::Infix { operator, left, right }))
    }

    fn parse_group(&mut self) -> Result<Box<ExpressionNode>> {
        let token: Rc<Token> = Rc::clone(self.current());

        self.next();

        let expression = self.parse_expression_node(OrderOfOps::Lowest)?;

        self.next();

        Ok(expression)
    }


    fn parse_fn(&mut self) -> Result<Box<ExpressionNode>> {
        let token = Rc::clone(&self.token_current);
        self.next();

        try_next!(self, TokenType::LeftParen);
        let params = self.parse_fn_param()?;
        try_next!(self, TokenType::RightParen);

        try_next!(self, TokenType::LeftBrace);
        let body = self.parse_program()?;

        if self.peek().token_type == TokenType::Semicolon {
            try_next!(self, TokenType::RightBrace);
        }

        Ok(Box::new(ExpressionNode::Function { params, body }))
    }

    fn parse_fn_param(&mut self) -> Result<Vec<Identifier>> {
        let mut result: Vec<Identifier> = Vec::new();

        if self.current().token_type == TokenType::RightParen {
            return Ok(result);
        }

        loop {
            result.push(Identifier(self.current().literal.clone()));
            self.next();

            if self.current().token_type == TokenType::RightParen {
                break;
            }
            try_next!(self, TokenType::Comma);
        }

        Ok(result)
    }
    fn parse_call(&mut self, function: Box<ExpressionNode>, _: Operator) -> Result<Box<ExpressionNode>> {
        let token = Rc::clone(&self.token_current);

        try_next!(self, TokenType::LeftParen);
        let params = self.parse_expression_list(TokenType::RightParen)?;
        // try_next!(self, TokenType::RightParen);

        Ok(Box::new(ExpressionNode::Call { function, params }))
    }

    fn parse_index(&mut self, left: Box<ExpressionNode>, _: Operator) -> Result<Box<ExpressionNode>> {
        let token = Rc::clone(&self.token_current);

        try_next!(self, TokenType::LeftBracket);
        let index = self.parse_expression_node(OrderOfOps::Lowest)?;
        self.next();

        return Ok(Box::new(
            ExpressionNode::Index { left, index }
        ));
    }

    fn parse_expression_list(&mut self, end_token: TokenType) -> Result<Vec<Box<ExpressionNode>>> {
        let mut result: Vec<Box<ExpressionNode>> = Vec::new();

        if self.current().token_type == end_token {
            return Ok(result);
        }

        loop {
            result.push(self.parse_expression_node(OrderOfOps::Lowest)?);
            self.next();

            if self.current().token_type == end_token {
                break;
            }
            try_next!(self, TokenType::Comma);
        }

        Ok(result)
    }

    pub fn parse_integer_literal(&mut self) -> Result<Box<ExpressionNode>> {
        let Ok(value) = self.current().literal.parse::<isize>() else {
            return Err(Error::InvalidTokenError(Rc::clone(self.current())));
        };

        Ok(Box::new(ExpressionNode::Int(value)))
    }

    pub fn parse_bool(&mut self) -> Result<Box<ExpressionNode>> {
        let value = match self.current().token_type {
            TokenType::True => true,
            TokenType::False => false,
            _ => false,
        };

        Ok(Box::new(ExpressionNode::Bool(value)))
    }
    fn parse_string(&self) -> Result<Box<ExpressionNode>> {
        Ok(Box::new(
            ExpressionNode::String(
                self.current().literal.clone()
            )
        ))
    }

    fn parse_array_literal(&mut self) -> Result<Box<ExpressionNode>> {
        let token = Rc::clone(&self.token_current);

        try_next!(self, TokenType::LeftBracket);
        let params = self.parse_expression_list(TokenType::RightBracket)?;

        Ok(Box::new(ExpressionNode::ArrayLiteral { params }))
    }

    fn parse_hash_literal(&mut self) -> Result<Box<ExpressionNode>> {
        let mut params: Vec<(Box<ExpressionNode>, Box<ExpressionNode>)> = Vec::new();

        if self.current().token_type == TokenType::RightBrace {
            return Ok(Box::new(ExpressionNode::HashLiteral { params }));
        }

        try_next!(self, TokenType::LeftBrace);
        loop {
            let key = self.parse_expression_node(OrderOfOps::Lowest)?;
            self.next();


            try_next!(self, TokenType::Colon);
            let val = self.parse_expression_node(OrderOfOps::Lowest)?;
            self.next();

            params.push((key, val));

            if self.current().token_type == TokenType::RightBrace {
                break;
            }
            try_next!(self, TokenType::Comma);
        }

        return Ok(Box::new(ExpressionNode::HashLiteral { params }));
    }
}

