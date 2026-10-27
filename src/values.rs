use crate::{
    Env, Interpreter,
    parser::Expr,
    pattern::{Literal, Pattern},
};
use miette::Result;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum Value {
    Atom(String),
    Number(i32),
    String(String),
    Function {
        params: Vec<Pattern>,
        body: Expr,
        env: Env,
    },
    BuiltinFunction {
        params: Vec<Pattern>,
        body: fn(&Interpreter, &mut Env, &[Expr]) -> Result<Value>,
    },
    Product(Box<Value>, Box<Value>),
}

impl Value {
    pub fn ty(&self) -> Ty {
        match self {
            Value::Atom(_) => Ty::Atom,
            Value::Number(_) => Ty::Number,
            Value::String(_) => Ty::String,
            Value::Function { .. } | Value::BuiltinFunction { .. } => Ty::Function,
            Value::Product(a, b) => Ty::Prod(Box::new(a.ty()), Box::new(b.ty())),
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
            Value::BuiltinFunction { .. } => write!(f, "<function>"),
            Value::Product(a, b) => write!(f, "({a}, {b})"),
        }
    }
}

impl From<Literal> for Value {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Atom(a) => Value::Atom(a),
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Ty {
    Atom,
    Number,
    String,
    Function,
    Any,
    List,
    Prod(Box<Ty>, Box<Ty>),
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
            Ty::Prod(a, b) => write!(f, "Prod({a}, {b})"),
        }
    }
}
