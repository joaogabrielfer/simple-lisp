use std::{collections::HashMap, fmt::Display};

use miette::Report;

use crate::{
    parser::{Expr, ExprKind},
    values::Value,
};

#[derive(Clone, Debug)]
pub enum Pattern {
    Bind(String),
    Wildcard,
    Literal(Literal),
    Product(Box<Pattern>, Box<Pattern>),
}

impl Pattern {
    pub fn match_value(&self, value: Value) -> Option<Bindings> {
        match (self, value) {
            (Pattern::Bind(bind), v) => Some(HashMap::from([(bind.to_owned(), v)])),
            (Pattern::Wildcard, _) => Some(HashMap::new()),
            (Pattern::Literal(Literal::Atom(a)), Value::Atom(b)) if *a == b => Some(HashMap::new()),
            (Pattern::Literal(Literal::Number(a)), Value::Number(b)) if *a == b => {
                Some(HashMap::new())
            }
            (Pattern::Literal(Literal::String(a)), Value::String(b)) if *a == b => {
                Some(HashMap::new())
            }
            (Pattern::Product(p1, p2), Value::Product(v1, v2)) => {
                let mut m1 = p1.match_value(*v1)?;
                let m2 = p2.match_value(*v2)?;
                m1.extend(m2);
                Some(m1)
            }
            _ => None,
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Bind(b) => write!(f, "{b}"),
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Literal(literal) => write!(f, "{literal}"),
            Pattern::Product(p1, p2) => write!(f, "({p1}, {p2})"),
        }
    }
}

impl TryFrom<&Expr> for Pattern {
    type Error = Report;

    fn try_from(expr: &Expr) -> Result<Self, Self::Error> {
        match expr.kind.clone() {
            ExprKind::AtomLit(a) => Ok(Pattern::Literal(Literal::Atom(a))),
            ExprKind::NumberLit(n) => Ok(Pattern::Literal(Literal::Number(n))),
            ExprKind::StringLit(s) => Ok(Pattern::Literal(Literal::String(s))),
            ExprKind::Ident(i) => Ok(Pattern::Bind(i)),
            ExprKind::List(exprs) => match exprs
                .split_first()
                .map(|(head, tail)| (head.kind.clone(), tail))
            {
                Some((ExprKind::Ident(s), rest)) if s == "prod" => {
                    if rest.len() != 2 {
                        todo!()
                    } else {
                        Ok(Pattern::Product(
                            Box::new((&rest[0]).try_into()?),
                            Box::new((&rest[1]).try_into()?),
                        ))
                    }
                }
                _ => todo!(),
            },
        }
    }
}

pub type Bindings = HashMap<String, Value>;

#[derive(Clone, Debug)]
pub enum Literal {
    Atom(String),
    String(String),
    Number(i32),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Atom(a) => write!(f, ":{a}"),
            Literal::String(s) => write!(f, "{s}"),
            Literal::Number(n) => write!(f, "{n}"),
        }
    }
}

struct PatternError;
