use miette::Result;

use crate::{
    interpreter::{Interpreter, RuntimeErrorKind},
    parser::Expr,
    pattern::{Bindings, Pattern},
    values::Value,
};
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct Env(HashMap<String, Value>);

impl Env {
    pub fn get(&self, name: String) -> Option<&Value> {
        self.0.get(&name)
    }

    pub fn set(&mut self, name: String, value: Value) -> Option<Value> {
        self.0.insert(name, value)
    }

    pub fn clone_with(&self, binds: Vec<(String, Value)>) -> Self {
        let mut env = self.clone();
        for b in binds {
            env.set(b.0, b.1);
        }
        env
    }

    // this gives priority to the `other` env
    pub fn join(&self, other: &Env) -> Self {
        let mut env = self.clone();
        for (key, value) in other.0.iter() {
            env.set(key.clone(), value.clone());
        }
        env
    }

    pub fn resolve_params(
        self,
        interpreter: &Interpreter,
        params: Vec<Pattern>,
        operands: &[Expr],
    ) -> Result<Env> {
        let mut env = self;
        let mut bindings = HashMap::new();
        for (param, expr) in params.into_iter().zip(operands) {
            let span = expr.span;
            let value = interpreter.run(expr, &mut env)?;
            let Some(binding) = param.match_value(value.clone()) else {
                return Err(interpreter.error(
                    RuntimeErrorKind::MismatchedPattern {
                        expected: param,
                        got: value,
                    },
                    span,
                ));
            };
            bindings.extend(binding);
        }
        env.0.extend(bindings);
        Ok(env)
    }

    pub fn bind(&mut self, binds: Bindings) {
        self.0.extend(binds);
    }
}
