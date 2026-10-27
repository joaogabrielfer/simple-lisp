use std::{
    fs::read_to_string,
    io::{Write, stdin, stdout},
};

use clap::Parser;
use lexer::*;
use miette::{IntoDiagnostic, Result};

use crate::{env::Env, interpreter::Interpreter};

mod builtins;
mod env;
mod interpreter;
mod lexer;
mod parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Command {
    Run { path: String },
    Eval { code: String },
    Repl,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (source_path, source, is_eval) = match args.command {
        Command::Run { path } => (path.clone(), read_to_string(path).into_diagnostic()?, false),
        Command::Eval { code } => ("<eval>".to_string(), code, true),
        Command::Repl => {
            let mut env = Env::default().clone_with(builtins::builtins());
            loop {
                print!("> ");
                stdout().flush().unwrap();
                let mut buffer = String::new();
                stdin().read_line(&mut buffer).unwrap();
                let lexer = Lexer::new("<repl>".to_string(), buffer);
                let mut parser = parser::Parser::new(lexer);
                let values = Interpreter::new(parser.parse().unwrap())
                    .run_program(&mut env)
                    .unwrap();
                for v in values {
                    println!("{v}")
                }
            }
        }
    };

    let lexer = Lexer::new(source_path, source);

    // for tk in lexer.clone() {
    //     println!("{}", tk.unwrap())
    // }
    let mut parser = parser::Parser::new(lexer);
    let mut env = Env::default().clone_with(builtins::builtins());
    let values = Interpreter::new(
        parser.parse().unwrap(),
        parser.lexer.source,
        parser.lexer.source_path,
    )
    .run_program(&mut env)?;

    if is_eval {
        for v in values {
            println!("{v}")
        }
    }

    Ok(())
}
