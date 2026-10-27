use crate::{
    interpreter::RuntimeErrorKind,
    parser::{Expr, ExprKind},
    pattern::{Literal, Pattern},
    values::{Ty, Value},
};

macro_rules! bind_arg {
    ($i:expr, $env:expr, $expr:expr, raw, $expected:expr, $pat:pat) => {
        let expr = $expr;

        #[allow(irrefutable_let_patterns)]
        let $pat = expr else {
            return Err($i.error(
                RuntimeErrorKind::ExpectedValue {
                    expected: $expected,
                    got: None,
                },
                expr.span,
            ));
        };
    };

    ($i:expr, $env:expr, $expr:expr, eval, $expected:expr, $pat:pat) => {
        let expr = $expr;
        let value = $i.run(expr, $env)?;
        let got = value.ty();

        #[allow(irrefutable_let_patterns)]
        let $pat = value else {
            return Err($i.error(
                RuntimeErrorKind::ExpectedValue {
                    expected: $expected,
                    got: Some(got),
                },
                expr.span,
            ));
        };
    };
}

macro_rules! builtin {
    (
        $name:ident |$i:ident, $env:ident| (
            $(
                $mode:ident $param:expr => $expected:expr => $pat:pat
            ),* $(,)?
        ) $body:block
    ) => {
        (
            stringify!($name).to_string(),
            Value::BuiltinFunction {
                params: vec![
                    $(
                        $param,
                    )*
                ],

                #[allow(unused_variables, unused_mut)]
                body: |$i, $env, args| {
                    let expected = 0usize $(+ {
                        let _ = stringify!($param);
                        1usize
                    })*;

                    if args.len() != expected {
                        miette::bail!(
                            "Expected {} arguments, got {}",
                            expected,
                            args.len()
                        );
                    }

                    let mut iter = args.iter();

                    $(
                        bind_arg!(
                            $i,
                            $env,
                            iter.next().unwrap(),
                            $mode,
                            $expected,
                            $pat
                        );
                    )*

                    $body
                },
            },
        )
    };
}

macro_rules! atom {
    ($a:ident) => {
        Value::Atom(stringify!($a).into())
    };
}

pub fn builtins() -> Vec<(String, Value)> {
    [
        builtin! {
            add |i, env| (
                eval Pattern::Bind("a".to_string()) => Ty::Number => Value::Number(a),
                eval Pattern::Bind("b".to_string()) => Ty::Number => Value::Number(b)
            ) {
                Ok(Value::Number(a + b))
            }
        },
        builtin! {
            mul |i, env| (
                eval Pattern::Bind("a".to_string()) => Ty::Number => Value::Number(a),
                eval Pattern::Bind("b".to_string()) => Ty::Number => Value::Number(b)
            ) {
                Ok(Value::Number(a * b))
            }
        },
        builtin! {
            println |i, env| (
                eval Pattern::Bind("v".to_string()) => Ty::Any => v
            ) {
                println!("{v}");
                Ok(atom!(unit))
            }
        },
        builtin! {
            def |i, env| (
                raw Pattern::Bind("name".to_string()) => Ty::Any => e,
                eval Pattern::Bind("value".to_string()) => Ty::Any => value
            ) {
                let pat: Pattern = e.try_into()?;
                match pat.match_value(value) {
                    Some(binds) => env.bind(binds),
                    None => todo!()
                }
                Ok(atom!(unit))
            }
        },
        builtin! {
            lambda |i, env| (
                raw Pattern::Bind("raw_params".to_string()) => Ty::List => Expr {kind: ExprKind::List(raw_params), span},
                raw Pattern::Bind("body".to_string()) => Ty::Any => body
            ) {
                let mut params = Vec::new();

                for p in raw_params {
                    let pat = match p.kind.clone() {
                        ExprKind::AtomLit(a) => Pattern::Literal(Literal::Atom(a)),
                        ExprKind::NumberLit(n) => Pattern::Literal(Literal::Number(n)),
                        ExprKind::StringLit(s) => Pattern::Literal(Literal::String(s)),
                        ExprKind::Ident(i) => Pattern::Bind(i),
                        ExprKind::List(exprs) => miette::bail!("list cannot be parameters as of now"),
                    };

                    params.push(pat)
                }

                Ok(Value::Function {
                    params,
                    body: body.clone(),
                    env: env.clone()
                })
            }
        },
        builtin! {
            prod |i, env| (
                eval Pattern::Bind("a".to_string()) => Ty::Any => a,
                eval Pattern::Bind("b".to_string()) => Ty::Any => b,
                ) {

                Ok(Value::Product(Box::new(a), Box::new(b)))
            }
        },
    ]
    .into()
}
