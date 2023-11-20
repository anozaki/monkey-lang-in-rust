use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::monkey::interpreter::Environment;
use crate::monkey::parser::ast::{Identifier, Program};
use crate::monkey::Result;

pub type BuiltInFn = fn(Vec<Object>) -> Result<Object>;

#[derive(Clone, PartialEq)]
pub enum Object {
    Null,
    Int(isize),
    Bool(bool),
    Return(Box<Object>),
    Function {
        ident: Vec<Identifier>,
        program: Box<Program>,
        env: Rc<RefCell<Environment>>,
    },
    BuiltIn {
        name: String,
        params: usize,
        program: BuiltInFn,
    },
    String(String),
    Error(String),
    Array(Vec<Object>),
    Hash(HashMap<Object, Object>),
}

// FIXME: is there any easier way to do this?
impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Null => write!(f, "Null"),
            Object::Int(val) => write!(f, "Int({:?})", val),
            Object::Bool(val) => write!(f, "Bool({:?})", val),
            Object::Return(val) => write!(f, "Return({:?})", val),

            // env could contain self - to prevent inf recursion we omit printing env.
            Object::Function { ident, program, env } => write!(f, "Function {{ ident: {:?}, program: {:?} }}", ident, program),

            // Omit program since we use this for test and we want predictable output.
            Object::BuiltIn { name, params, program } => write!(f, "BuiltIn {{ name: {:?}, params: {:?} }}", name, params),

            Object::String(val) => write!(f, "String({:?})", val),
            Object::Error(val) => write!(f, "Error({:?})", val),
            Object::Array(val) => write!(f, "Array({:?})", val),
            Object::Hash(result) => write!(f, "Hash({:?})", result),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Null => write!(f, "Null"),
            Object::Int(val) => write!(f, "{}", val),
            Object::Bool(val) => write!(f, "{}", val),
            Object::String(val) => write!(f, "\"{}\"", val),
            Object::Error(val) => write!(f, "Error: {}", val),
            Object::Array(val) => {
                write!(f, "[ ");
                let mut first = true;
                for v in val {
                    if !first {
                        write!(f, ", ");
                    }
                    first = false;
                    write!(f, "{}", v);
                }
                write!(f, " ]")
            }
            Object::Hash(val) => {
                write!(f, "{{ ");
                let mut first = true;
                for (key, val) in val {
                    if !first {
                        write!(f, ", ");
                    }
                    first = false;
                    write!(f, "{}: {}", key, val);
                }
                write!(f, " }}")
            }
            val @ _ => Debug::fmt(self, f)
        }
    }
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::Int(val) => val.hash(state),
            Object::Bool(val) => val.hash(state),
            Object::String(val) => val.hash(state),

            _ => todo!("Not supported"),
        }
    }
}

pub const TRUE: Object = Object::Bool(true);
pub const FALSE: Object = Object::Bool(false);

pub const NULL: Object = Object::Null;
