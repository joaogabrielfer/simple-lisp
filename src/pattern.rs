use std::{collections::HashMap, fmt::Display};

use miette::Report;

use crate::{
    parser::{Expr, ExprKind},
    values::{Ty, Value},
};

#[derive(Clone, Debug)]
pub enum Pattern {
    Bind(String),
    Wildcard,
    Literal(Literal),
    Product(Box<Pattern>, Box<Pattern>),
    Typed(Box<Pattern>, Ty),
    Range(i32, i32),
    List(Vec<Pattern>),
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
            (Pattern::Typed(pat, ty), v) if v.ty() == *ty => pat.match_value(v),
            (Pattern::List(pats), Value::List(vals)) => pats
                .iter()
                .zip(vals)
                .map(|(p, v)| p.match_value(v))
                .reduce(|mut bindings, curr| {
                    if let Some(b) = curr
                        && let Some(ref mut bindings) = bindings
                    {
                        bindings.extend(b);
                    }
                    bindings
                })?,
            (Pattern::Range(a, b), Value::Number(n)) => {
                if (*a..*b).contains(&n) {
                    Some(HashMap::new())
                } else {
                    None
                }
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
            Pattern::Typed(pat, ty) => write!(f, "({pat}: {ty})"),
            Pattern::List(l) => write!(
                f,
                "({})",
                l.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Pattern::Range(a, b) => write!(f, "{a}..{b}"),
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
                // TODO: refactor this as builtins also, making the builtin array be eithe
                // ValueBuiltin or PatternBuiltin, with the interpreter consuming values and
                // patterns being consumed here.
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
                Some((ExprKind::Ident(s), rest)) if s == "range" => {
                    if rest.len() != 2 {
                        todo!()
                    } else {
                        let (ExprKind::NumberLit(a), ExprKind::NumberLit(b)) =
                            (&rest[0].kind, &rest[1].kind)
                        else {
                            todo!("report expected numbers")
                        };

                        if a > b {
                            todo!("ranges cannot be decreasing yet")
                        }

                        Ok(Pattern::Range(*a, *b))
                    }
                }
                Some((ExprKind::Ident(i), [])) => Ok(Pattern::Bind(i)),
                _ => todo!("invalid pattern"),
            },
            ExprKind::UnderscoreLit => Ok(Pattern::Wildcard),
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
