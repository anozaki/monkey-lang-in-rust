use std::rc::Rc;
use thiserror::Error;
use crate::monkey::token::Token;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("Generic {0}")]
    TodoError(String),

    #[error("Invalid token found at {0:?}")]
    InvalidTokenError(Rc<Token>),

    #[error("Invalid escape char {0:?}")]
    InvalidCharError(char),
}