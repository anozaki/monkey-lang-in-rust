use crate::monkey::error::Error;

pub mod error;

pub mod lexer;
pub mod helper;
pub mod parser;
pub mod token;


pub mod interpreter;

pub type Result<T> = core::result::Result<T, Error>;
