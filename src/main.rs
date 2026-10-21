use std::fs::read_to_string;

use clap::Parser;
use lexer::*;

use crate::{env::Env, interpreter::Interpreter};

mod builtins;
mod env;
mod interpreter;
mod lexer;
mod parser;
mod source;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file_path: String,
}

fn main() {
    let args = Args::parse();

    let lexer = Lexer::new(
        args.file_path.clone(),
        read_to_string(args.file_path).unwrap(),
    );

    // for tk in lexer.clone() {
    //     println!("{}", tk.unwrap())
    // }
    let mut parser = parser::Parser::new(lexer);
    let mut env = Env::default().clone_with(builtins::builtins());
    Interpreter::new(parser.parse().unwrap())
        .run_program(&mut env)
        .unwrap();
}
