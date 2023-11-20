use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::monkey::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Not,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Greater,
    Less,
    Equal,
    NotEqual,
    Call,
    Index,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<StatementNode>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode {
    Let(Identifier, Box<ExpressionNode>),
    Return(Box<ExpressionNode>),
    If {
        condition: Box<ExpressionNode>,
        consequence: Box<Program>,
        alternative: Option<Box<Program>>,
    },
    Expression {
        expression: Box<ExpressionNode>
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode {
    Identifier(Identifier),
    Bool(bool),
    Int(isize),
    String(String),
    Call {
        function: Box<ExpressionNode>,
        params: Vec<Box<ExpressionNode>>,
    },
    Function {
        params: Vec<Identifier>,
        body: Box<Program>,
    },
    Infix {
        operator: Operator,
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
    },
    Prefix {
        operator: Operator,
        expression: Box<ExpressionNode>,
    },
    ArrayLiteral {
        params: Vec<Box<ExpressionNode>>,
    },
    Index {
        left: Box<ExpressionNode>,
        index: Box<ExpressionNode>,
    },
    HashLiteral {
        params: Vec<(Box<ExpressionNode>, Box<ExpressionNode>)>,
    },
}

pub struct Node {
    token: Rc<Token>,
    program: Program,
}
