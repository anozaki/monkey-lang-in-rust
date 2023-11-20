#![allow(unused)]

use std::fs;
use clap::Parser;
use crate::monkey::interpreter::Evaluate;
use crate::monkey::lexer::MonkeyLexer;
use crate::repl::Repl;
mod monkey;
mod repl;

#[derive(Parser, Debug)]

struct Args {
    #[arg(short, long, default_value = "")]
    input: String,
}

fn main() {
    let args = Args::parse();

    if args.input.len() > 0 {
        let contents = fs::read_to_string(args.input).unwrap();
        let lexer = MonkeyLexer::new(&contents);
        let mut parser = monkey::parser::Parser::new(&lexer);

        let program = parser.parse_program().unwrap();

        let mut eval = Evaluate::new();
        let out = eval.evaluate(&program);

        println!("{}", out.unwrap());

    } else {
        let repl = Repl::new();
        repl.start();
    }
}
