use dialoguer::Input;
use crate::monkey::interpreter::Evaluate;

use crate::monkey::lexer::{eval_lexer, MonkeyLexer};
use crate::monkey::parser::Parser;

pub struct Repl {}

impl Repl {
    pub fn new() -> Self {
        Repl {}
    }
    pub fn start(&self) {
        loop {
            let stop_loop = false;
            while !stop_loop {
                let command: String = Input::new()
                    .with_prompt(">> ")
                    .interact_text()
                    .unwrap();

                let lexer = MonkeyLexer::new(&command);
                let mut parser = Parser::new(&lexer);

                let program = parser.parse_program().unwrap();

                let mut eval = Evaluate::new();
                let out = eval.evaluate(&program);

                println!("{:?}", out);
            }
        }
    }
}