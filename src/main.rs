use std::fs::read_to_string;

use clap::Parser;
use colored::Colorize;
use lexer::*;
use miette::{IntoDiagnostic, Result};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{env::Env, interpreter::Interpreter};

mod builtins;
mod env;
mod interpreter;
mod lexer;
mod parser;
mod values;

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
            let mut rl = DefaultEditor::new().unwrap();

            loop {
                match rl.readline(&format!("{} ", "λ>".blue())) {
                    Ok(line) => {
                        rl.add_history_entry(line.as_str()).unwrap();
                        let lexer = Lexer::new("<repl>".to_string(), line);
                        let mut parser = parser::Parser::new(lexer);
                        match parser.parse().and_then(|ast| {
                            Interpreter::new(ast, parser.lexer.source, parser.lexer.source_path)
                                .run_program(&mut env)
                        }) {
                            Ok(values) => {
                                for v in values {
                                    println!("{v}")
                                }
                            }
                            Err(e) => eprintln!("{} {e}", "ERROR:".red()),
                        }
                    }
                    Err(ReadlineError::Interrupted) => (),
                    Err(ReadlineError::Eof) => {
                        println!("exiting...");
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("{} {e}", "ERROR:".red())
                    }
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
