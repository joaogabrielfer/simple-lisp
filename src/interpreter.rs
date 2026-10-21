use std::fmt::Display;

use crate::{env::Env, parser::Expr};

pub struct Interpreter {
    ast: Vec<Expr>,
}

impl Interpreter {
    pub fn new(ast: Vec<Expr>) -> Self {
        Self { ast }
    }

    pub fn run_program(&mut self, env: &mut Env) -> Result<Vec<Value>, RuntimeError> {
        let mut values = vec![];
        for expr in self.ast.iter() {
            values.push(Interpreter::run(expr, env)?)
        }
        Ok(values)
    }

    pub fn run(expr: &Expr, env: &mut Env) -> Result<Value, RuntimeError> {
        match expr {
            Expr::AtomLit(a) => Ok(Value::Atom(a.clone())),
            Expr::NumberLit(n) => Ok(Value::Number(*n)),
            Expr::StringLit(s) => Ok(Value::String(s.clone())),
            Expr::Ident(name) => match env.get(name.clone()) {
                Some(v) => Ok((*v).clone()),
                None => todo!("report unknown identifier error"),
            },
            Expr::Application { operator, operand } => {
                let Expr::Ident(ref name) = **operator else {
                    todo!("report expected ident error")
                };

                match env.get(name.clone()) {
                    Some(Value::Function { params, body }) => {
                        if operand.len() != params.len() {
                            todo!("report wrong argument count error")
                        }

                        let curr_env = env.clone();
                        Interpreter::run(
                            body,
                            &mut env.clone_with(
                                params
                                    .iter()
                                    .map(|p| p.name.clone())
                                    .zip(
                                        operand
                                            .iter()
                                            .map(|op| Interpreter::run(op, &mut curr_env.clone()))
                                            .collect::<Result<Vec<_>, _>>()?,
                                    )
                                    .collect(),
                            ),
                        )
                    }
                    Some(Value::BuiltinFunction { params, body }) => {
                        if operand.len() != params.len() {
                            todo!("report wrong argument count error")
                        }

                        body(env, operand)
                    }
                    Some(_) => todo!("report expected function error"),
                    None => todo!("report unknown identifier {name} error"),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError;

#[derive(Clone, Debug)]
pub enum Value {
    Atom(String),
    Number(i32),
    String(String),
    Function {
        params: Vec<Parameter>,
        body: Expr,
    },
    BuiltinFunction {
        params: Vec<Parameter>,
        body: fn(&mut Env, &[Expr]) -> Result<Value, RuntimeError>,
    },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Atom(a) => write!(f, ":{a}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Function { .. } => write!(f, "<function>"),
            Value::BuiltinFunction { .. } => write!(f, "function"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
}
