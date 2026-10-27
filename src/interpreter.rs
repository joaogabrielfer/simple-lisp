use std::fmt::Display;

use miette::{Diagnostic, NamedSource, Report, Result, SourceSpan};
use thiserror::Error;

use crate::{
    env::Env,
    interpreter::RuntimeErrorKind::ExpectedValue,
    parser::{Expr, ExprKind},
};

pub struct Interpreter {
    ast: Vec<Expr>,
    source: NamedSource<String>,
}

impl Interpreter {
    pub fn new(ast: Vec<Expr>, source: String, source_path: String) -> Self {
        Self {
            ast,
            source: NamedSource::new(source_path, source),
        }
    }

    pub fn run_program(&mut self, env: &mut Env) -> Result<Vec<Value>> {
        let mut values = vec![];
        for expr in self.ast.iter() {
            values.push(self.run(expr, env)?)
        }
        Ok(values)
    }

    pub fn run(&self, expr: &Expr, env: &mut Env) -> Result<Value> {
        match &expr.kind {
            ExprKind::AtomLit(a) => Ok(Value::Atom(a.clone())),
            ExprKind::NumberLit(n) => Ok(Value::Number(*n)),
            ExprKind::StringLit(s) => Ok(Value::String(s.clone())),
            ExprKind::Ident(name) => match env.get(name.clone()) {
                Some(v) => Ok((*v).clone()),
                None => {
                    Err(self.error(RuntimeErrorKind::UnknownIdentifier(name.clone()), expr.span))
                }
            },
            ExprKind::List(exprs) => self.run_list(exprs, expr, env),
        }
    }

    fn run_list(&self, exprs: &[Expr], expr: &Expr, env: &mut Env) -> Result<Value> {
        match exprs.first() {
            Some(Expr {
                kind: ExprKind::Ident(name),
                span,
            }) => {
                let operands = &exprs[1..];
                match env.get(name.clone()) {
                    Some(Value::Function {
                        params,
                        body,
                        env: fn_env,
                    }) => {
                        if operands.len() != params.len() {
                            return Err(self.error(
                                RuntimeErrorKind::ArityMismatch(params.len(), operands.len()),
                                *span,
                            ));
                        }

                        let curr_env = env.clone();
                        self.run(
                            body,
                            &mut env.join(fn_env).clone_with(
                                params
                                    .iter()
                                    .map(|p| p.name.clone())
                                    .zip(
                                        operands
                                            .iter()
                                            .map(|op| self.run(op, &mut curr_env.clone()))
                                            .collect::<Result<Vec<_>, _>>()?,
                                    )
                                    .collect(),
                            ),
                        )
                    }
                    Some(Value::BuiltinFunction { params, body }) => {
                        if operands.len() != params.len() {
                            return Err(self.error(
                                RuntimeErrorKind::ArityMismatch(params.len(), operands.len()),
                                *span,
                            ));
                        }

                        body(self, env, operands)
                    }
                    v => Err(self.error(
                        ExpectedValue {
                            expected: Ty::Function,
                            got: v.map(|v| v.ty()),
                        },
                        *span,
                    )),
                }
            }
            Some(e) => {
                let (params, body, fn_env) = match self.run(e, env)? {
                    Value::Function {
                        params,
                        body,
                        env: fn_env,
                    } => (params, body, fn_env),
                    other => {
                        return Err(self.error(
                            RuntimeErrorKind::ExpectedValue {
                                expected: Ty::Function,
                                got: Some(other.ty()),
                            },
                            e.span,
                        ));
                    }
                };
                let operands = &exprs[1..];
                if operands.len() != params.len() {
                    return Err(self.error(
                        RuntimeErrorKind::ArityMismatch(params.len(), operands.len()),
                        e.span,
                    ));
                }

                let curr_env = env.clone();
                self.run(
                    &body,
                    &mut env.join(&fn_env).clone_with(
                        params
                            .iter()
                            .map(|p| p.name.clone())
                            .zip(
                                operands
                                    .iter()
                                    .map(|op| self.run(op, &mut curr_env.clone()))
                                    .collect::<Result<Vec<_>, _>>()?,
                            )
                            .collect(),
                    ),
                )
            }
            None => Err(self.error(
                RuntimeErrorKind::ExpectedValue {
                    expected: Ty::Function,
                    got: None,
                },
                expr.span,
            )),
        }
    }
    pub fn error(&self, kind: RuntimeErrorKind, span: SourceSpan) -> Report {
        RuntimeError {
            src: self.source.clone(),
            span,
            kind,
        }
        .into()
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Atom(String),
    Number(i32),
    String(String),
    Function {
        params: Vec<Parameter>,
        body: Expr,
        env: Env,
    },
    BuiltinFunction {
        params: Vec<Parameter>,
        body: fn(&Interpreter, &mut Env, &[Expr]) -> Result<Value>,
    },
}

impl Value {
    pub fn ty(&self) -> Ty {
        match self {
            Value::Atom(_) => Ty::Atom,
            Value::Number(_) => Ty::Number,
            Value::String(_) => Ty::String,
            Value::Function { .. } | Value::BuiltinFunction { .. } => Ty::Function,
        }
    }
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

#[derive(Error, Debug, Diagnostic)]
#[error("{kind}")]
#[diagnostic(code(interpreter::runtime))]
pub struct RuntimeError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("error occurred here")]
    pub span: SourceSpan,

    pub kind: RuntimeErrorKind,
}

#[derive(Debug, Clone, Copy)]
pub enum Ty {
    Atom,
    Number,
    String,
    Function,
    Any,
    List,
    Ident,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Atom => write!(f, "Atom"),
            Ty::Number => write!(f, "Number"),
            Ty::String => write!(f, "String"),
            Ty::Function => write!(f, "Function"),
            Ty::Any => write!(f, "Any"),
            Ty::List => write!(f, "List"),
            Ty::Ident => write!(f, "Ident"),
        }
    }
}

#[derive(Debug, Error)]
pub enum RuntimeErrorKind {
    #[error("arity mismatch: expected {0} arguments, got {1}")]
    ArityMismatch(usize, usize),
    #[error("unknown identifier '{0}'")]
    UnknownIdentifier(String),
    #[error("expected of type {expected}, got {}", .got.map_or("none".to_string(), |t| t.to_string()))]
    ExpectedValue { expected: Ty, got: Option<Ty> },
}
