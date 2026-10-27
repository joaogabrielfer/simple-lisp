use crate::pattern::Pattern;
use crate::values::Ty;
use crate::values::Value;

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

                        self.run(
                            body,
                            &mut env.clone().join(fn_env).resolve_params(
                                self,
                                params.clone(),
                                operands,
                            )?,
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

                self.run(
                    &body,
                    &mut env
                        .clone()
                        .join(&fn_env)
                        .resolve_params(self, params, operands)?,
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

#[derive(Debug, Error)]
pub enum RuntimeErrorKind {
    #[error("arity mismatch: expected {0} arguments, got {1}")]
    ArityMismatch(usize, usize),
    #[error("unknown identifier '{0}'")]
    UnknownIdentifier(String),
    #[error("expected of type {expected}, got {}", .got.clone().map_or("none".to_string(), |t| t.to_string()))]
    ExpectedValue { expected: Ty, got: Option<Ty> },
    #[error("value `{got}` cannot match with pattern `{expected}`")]
    MismatchedPattern { expected: Pattern, got: Value },
}
