use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr::hash;
use std::rc::Rc;
use crate::monkey::interpreter::builtin::{first, last, push, put, rest, str_len};
use crate::monkey::interpreter::object::{FALSE, NULL, Object, TRUE};
use crate::monkey::parser::ast::{ExpressionNode, Identifier, Operator, Program, StatementNode};
use crate::monkey::Result;

pub mod builtin;
mod object;


#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    store: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        let mut store: HashMap<String, Object> = HashMap::new();
        store.insert("len".to_string(), Object::BuiltIn {
            name: "len".to_string(),
            params: 1,
            program: str_len,
        });
        store.insert("first".to_string(), Object::BuiltIn {
            name: "first".to_string(),
            params: 1,
            program: first,
        });
        store.insert("last".to_string(), Object::BuiltIn {
            name: "last".to_string(),
            params: 1,
            program: last,
        });
        store.insert("push".to_string(), Object::BuiltIn {
            name: "push".to_string(),
            params: 2,
            program: push,
        });
        store.insert("rest".to_string(), Object::BuiltIn {
            name: "rest".to_string(),
            params: 1,
            program: rest,
        });
        store.insert("put". to_string(), Object::BuiltIn {
            name: "put".to_string(),
            params: 3,
            program: put,
        });

        Environment {
            store,
        }
    }
    pub fn store(&mut self, name: &str, value: &Object) {
        self.store.insert(name.to_string(), value.clone());
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            None => None,
            Some(val) => Some(val.clone())
        }
    }
}

pub struct Evaluate {}

impl Evaluate {
    pub fn new() -> Self {
        Evaluate {}
    }

    pub fn evaluate(&self, program: &Program) -> Result<Object> {
        let env = Rc::new(RefCell::new(Environment::new()));
        self.evaluate_program(program, &env)
    }

    pub fn evaluate_program(&self, program: &Program, env: &Rc<RefCell<Environment>>) -> Result<Object> {
        let result = self.evaluate_block(program, env)?;
        Ok(match result {
            Object::Return(val) => *val,
            val @ _ => val,
        })
    }

    pub fn evaluate_block(&self, program: &Program, env: &Rc<RefCell<Environment>>) -> Result<Object> {
        let mut result = NULL;

        for statement in &program.statements {
            result = match statement {
                StatementNode::Let(ident, expr) => self.eval_let_statement(ident, expr, env)?,
                StatementNode::If { condition, consequence, alternative } => self.eval_if_statement(condition, consequence, alternative, env)?,
                StatementNode::Return(expression) => Object::Return(Box::new(self.expression(expression, env)?)),
                StatementNode::Expression { expression } => self.expression(expression, env)?,
            };

            if let Object::Return(_) = result {
                break;
            }
        };

        Ok(result)
    }

    pub fn eval_let_statement(&self, identifier: &Identifier, expression: &ExpressionNode, env: &Rc<RefCell<Environment>>) -> Result<Object> {
        let result = self.expression(expression, env)?;
        if matches!(result, Object::Error(_)) {
            return Ok(result);
        }

        env.borrow_mut().store(&identifier.0, &result);

        Ok(result)
    }

    pub fn eval_if_statement(&self, condition: &ExpressionNode, consequence: &Program, alternative: &Option<Box<Program>>, env: &Rc<RefCell<Environment>>) -> Result<Object> {
        let result = self.expression(condition, env)?;

        Ok(
            if self.is_truthy(&result) {
                self.evaluate_block(&consequence, env)?
            } else if let Some(program) = alternative {
                self.evaluate_block(&program, env)?
            } else {
                NULL
            }
        )
    }

    pub fn is_truthy(&self, condition: &Object) -> bool {
        match condition {
            Object::Null => false,
            Object::Int(val) if *val == 0 => false,
            Object::Bool(val) => *val,
            _ => true,
        }
    }

    pub fn expression(&self, expression: &ExpressionNode, env: &Rc<RefCell<Environment>>) -> Result<Object> {
        Ok(match expression {
            ExpressionNode::Int(val) => Object::Int(*val),
            ExpressionNode::Bool(val) => match val {
                true => TRUE,
                false => FALSE,
            }
            ExpressionNode::Identifier(ident) => match env.borrow().get(&ident.0) {
                Some(val) => val,
                None => Object::Error(format!("identifier not found: {:?}", ident))
            },
            ExpressionNode::String(str) => Object::String(str.clone()),
            ExpressionNode::Prefix { operator, expression } => self.eval_prefix(operator, expression, env)?,
            ExpressionNode::Infix { operator, left, right } => {
                let left = self.expression(left, env)?;
                let right = self.expression(right, env)?;
                self.eval_infix(operator, &left, &right)?
            }
            ExpressionNode::Function { params, body } => Object::Function {
                ident: params.clone(),
                program: body.clone(),
                env: Rc::clone(env),
            },
            ExpressionNode::Call { function, params } => {
                let func = self.expression(function, env)?;

                if let Object::Function { ident, program, env: cap } = func {
                    let mut new_env = Environment::new();
                    for (name, obj) in &env.borrow().store {
                        new_env.store(name, obj);
                    }

                    for (_, (ident, item)) in ident.iter().zip(params).enumerate() {
                        let result = self.expression(item, env)?;
                        new_env.store(&ident.0, &result)
                    }

                    self.evaluate_program(&program, &Rc::new(RefCell::new(new_env)))?
                } else if let Object::BuiltIn { name, params: params_size, program } = func {
                    let mut func_params: Vec<Object> = Vec::new();
                    for item in params {
                        func_params.push(self.expression(item, env)?)
                    }

                    if func_params.len() != params_size {
                        return Ok(Object::Error(format!("{}(): Invalid number of argument - expected {} got {}", name, params_size, func_params.len())));
                    }

                    program(func_params)?
                } else if let Object::Error(message) = func {
                    panic!("{}", message)
                } else {
                    panic!()
                }
            }
            ExpressionNode::ArrayLiteral { params } => {
                let mut func_params: Vec<Object> = Vec::new();
                for item in params {
                    func_params.push(self.expression(item, env)?)
                }

                Object::Array(func_params)
            }
            ExpressionNode::Index { left, index } => {
                let left_obj = self.expression(left, env)?;
                match left_obj {
                    Object::Array(vec) => {
                        let index_obj = self.expression(index, env)?;
                        let Object::Int(offset) = &index_obj else {
                            return Ok(Object::Error(format!("Invalid index value: {:?}", index_obj)));
                        };

                        if offset.is_negative() || vec.len() < *offset as usize {
                            return Ok(NULL);
                        }

                        // To keep it simple we will clone...
                        vec[*offset as usize].clone()
                    }
                    Object::Hash(map) => {
                        let index_obj = self.expression(index, env)?;
                        map[&index_obj].clone()

                    },
                    _ => return Ok(Object::Error(format!("Can not index object type: {:?}", left_obj))),
                }

            }
            ExpressionNode::HashLiteral { params} => {
                let mut map: HashMap<Object, Object> = HashMap::new();

                for (key, val) in params {
                    let key_result = self.expression(key, env)?;
                    let val_result = self.expression(val, env)?;

                    map.insert(key_result, val_result);
                }

                Object::Hash(map)
            }
            _ => todo!()
        })
    }

    fn eval_prefix(&self, operator: &Operator, expression: &Box<ExpressionNode>, env: &Rc<RefCell<Environment>>) -> Result<Object> {
        Ok(match operator {
            Operator::Not => match self.expression(expression, env) {
                Ok(result) => match result {
                    Object::Bool(true) => FALSE,
                    Object::Bool(false) => TRUE,
                    Object::Int(0) => TRUE,
                    _ => FALSE,
                },
                Err(_) => panic!()
            },
            Operator::Neg => match self.expression(expression, env) {
                Ok(result) => match result {
                    Object::Int(val) => Object::Int(-val),
                    _ => Object::Error(format!("Unexpected value type: {:?}", result)),
                },
                Err(_) => panic!()
            },
            _ => Object::Error(format!("Unknown operator: {:?}", operator))
        })
    }

    fn eval_infix(&self, operator: &Operator, left: &Object, right: &Object) -> Result<Object> {
        Ok(match (left, right) {
            (Object::Int(left_val), Object::Int(right_val)) => match operator {
                Operator::Add => Object::Int(left_val + right_val),
                Operator::Sub => Object::Int(left_val - right_val),
                Operator::Mul => Object::Int(left_val * right_val),
                Operator::Div => Object::Int(left_val / right_val),
                Operator::Greater => Object::Bool(left_val > right_val),
                Operator::Less => Object::Bool(left_val < right_val),
                Operator::Equal => Object::Bool(left_val == right_val),
                Operator::NotEqual => Object::Bool(left_val != right_val),
                _ => Object::Error(format!("Unexpected Operator: {:?}", operator))
            },
            (Object::String(left_var), Object::String(right_var)) => match operator {
                Operator::Add => Object::String(format!("{}{}", left_var, right_var)),
                _ => Object::Error(format!("Unexpected Operator: {:?}", operator))
            }
            (Object::String(left_var), Object::Int(right_var)) => match operator {
                Operator::Add => Object::String(format!("{}{}", left_var, right_var)),
                _ => Object::Error(format!("Unexpected Operator: {:?}", operator))
            }
            _ => Object::Error(format!("Expected number, got {:?} and {:?}", left, right))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::monkey::lexer::MonkeyLexer;
    use crate::monkey::parser::ast::Operator;
    use crate::monkey::parser::Parser;
    use super::*;

    macro_rules! test_expression {
        ($out: ident, $command: expr) => {
            {
                let lexer = MonkeyLexer::new($command);
                let mut parser = Parser::new(&lexer);

                let program = parser.parse_program().unwrap();

                let mut eval = Evaluate::new();
                let out = eval.evaluate(&program);

                $out += &format!("{:?}\n", out)
            }
        }
    }

    #[test]
    fn text_eval_expression() {
        let mut result = String::new();

        test_expression!(result, "10");
        test_expression!(result, "152");
        test_expression!(result, "-5");
        test_expression!(result, "-1523");

        test_expression!(result, "true");
        test_expression!(result, "false");

        test_expression!(result, "!true");
        test_expression!(result, "!false");
        test_expression!(result, "!5");
        test_expression!(result, "!0");
        test_expression!(result, "!-5");
        test_expression!(result, "!-0");
        test_expression!(result, "!!5");
        test_expression!(result, "!!true");
        test_expression!(result, "!!false");

        test_expression!(result, "5 + 5 + 5 + 5 - 10");
        test_expression!(result, "2 * 2 * 2 * 2 * 2");
        test_expression!(result, "-50 + 100 + -50");
        test_expression!(result, "5 * 2 + 10");
        test_expression!(result, "5 + 2 * 10");
        test_expression!(result, "20 + 2 * -10");
        test_expression!(result, "50 / 2 * 2 + 10");
        test_expression!(result, "2 * (5 + 10)");
        test_expression!(result, "3 * 3 * 3 + 10");
        test_expression!(result, "3 * (3 * 3) + 10");
        test_expression!(result, "(5 + 10 * 2 + 15 / 3) * 2 + -10");

        test_expression!(result, "1 < 2");
        test_expression!(result, "1 > 2");
        test_expression!(result, "1 < 1");
        test_expression!(result, "1 > 1");
        test_expression!(result, "1 == 1");
        test_expression!(result, "1 != 1");
        test_expression!(result, "1 == 2");
        test_expression!(result, "1 != 2");

        test_expression!(result, "if (true) { 10 }");
        test_expression!(result, "if (false) { 10 }");
        test_expression!(result, "if (1) { 10 }");
        test_expression!(result, "if (1 < 2) { 10 }");
        test_expression!(result, "if (1 > 2) { 10 }");
        test_expression!(result, "if (1 > 2) { 10 } else { 20 }");
        test_expression!(result, "if (1 < 2) { 10 } else { 20 }");

        test_expression!(result, "return 10;");
        test_expression!(result, "return 10; 9;");
        test_expression!(result, "return 2 * 5; 9;");
        test_expression!(result, "9; return 2 * 5; 9;");

        test_expression!(result, "if (10 > 1) { if (10 > 1) { return 10; } } return 1;");

        test_expression!(result, "5 + true;");
        test_expression!(result, "-true;");
        test_expression!(result, "true + false");
        test_expression!(result, "if (10 > 1) { true + false; }");

        test_expression!(result, "let x = 5");
        test_expression!(result, "let x = 3 > 5");
        test_expression!(result, "let x = 100 > 5");
        test_expression!(result, "let x = 100 > 5; x");
        test_expression!(result, "let x = 100 > 5; y");
        test_expression!(result, "let x = 100; let y = 200; x + y");

        insta::assert_snapshot!(result)
    }

    #[test]
    fn test_eval_function() {
        let mut result = String::new();

        test_expression!(result, "fn() { let x = 5 }");
        test_expression!(result, "fn(a, b, c) { let x = 5 }");
        test_expression!(result, "let a = 5; let c = fn() { a }; let b = 5; c;");
        test_expression!(result, "fn(a) { fn() { } }");
        test_expression!(result, "let a = 5; let c = fn() { a }; c();");
        test_expression!(result, "let a = 5; let c = fn(b) { b + 10 }; c(a);");
        test_expression!(result, "let a = 5; let c = fn(b) { b + 10 }; c(2);");
        test_expression!(result, "let o = 5; let a = fn() { o }; let b = fn(m) { m() }; b(a);");

        insta::assert_snapshot!(result)
    }

    #[test]
    fn test_eval_string() {
        let mut result = String::new();

        test_expression!(result, "\"This is a test\"");
        test_expression!(result, "\"This \" + \"is\\n\"");
        test_expression!(result, "\"This \" + 5");

        insta::assert_snapshot!(result)
    }

    #[test]
    fn test_len() {
        let mut result = String::new();

        test_expression!(result, "len(\"This is a test\")");
        test_expression!(result, "len(12345)");
        test_expression!(result, "len(\"This is\", \"a test\")");

        insta::assert_snapshot!(result)
    }

    #[test]
    fn test_array() {
        let mut result = String::new();

        test_expression!(result, "[0, 1, 3]");
        test_expression!(result, "[0, 1, 3][1]");
        test_expression!(result, "[0, 1, 3][-1]");
        test_expression!(result, "[0, 1, 3][5]");
        test_expression!(result, "let a = 2; [0, 1, 3][a]");
        test_expression!(result, "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];");
        test_expression!(result, "let myArray = [1, 2, 3]; myArray[0] + myArray[myArray[0]] + myArray[myArray[0] + myArray[0]];");
        test_expression!(result, "len([1, 2, 3, 4])");
        test_expression!(result, "let myArray = [1, 2, 3]; len(myArray);");
        test_expression!(result, "first([1, 2, 3, 4])");
        test_expression!(result, "last([1, 2, 3, 4])");
        test_expression!(result, "push([1, 2], 5)");
        test_expression!(result, "rest([1, 2, 3])");

        insta::assert_snapshot!(result)
    }

    #[test]
    fn test_hash() {
        let mut result = String::new();

        test_expression!(result, "{\"a\": 5, \"b\": \"a\"}[\"b\"]");
        test_expression!(result, "{10: 5, 13: 5}[10]");
        test_expression!(result, "{true: 15, false: 65}[false]");
        test_expression!(result, "put({1: 15, 2: 65}, 3, 16)[3]");
        insta::assert_snapshot!(result)
    }
}