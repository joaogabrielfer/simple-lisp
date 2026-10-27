use crate::{Env, Interpreter, parser::Expr};
use miette::Result;
use std::fmt::Display;

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

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Ty {
    Atom,
    Number,
    String,
    Function,
    Any,
    List,
    Ident,
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
            Ty::Ident => write!(f, "Ident"),
            Ty::Prod(a, b) => write!(f, "Prod({a}, {b})"),
        }
    }
}
