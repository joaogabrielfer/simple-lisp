use crate::{
    interpreter::RuntimeErrorKind,
    parser::{Expr, ExprKind},
    values::{Parameter, Ty, Value},
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
                $mode:ident $param:ident : $expected:expr => $pat:pat
            ),* $(,)?
        ) $body:block
    ) => {
        (
            stringify!($name).to_string(),
            Value::BuiltinFunction {
                params: vec![
                    $(
                        Parameter {
                            name: stringify!($param).to_string(),
                        },
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
                eval a: Ty::Number => Value::Number(a),
                eval b: Ty::Number => Value::Number(b)
            ) {
                Ok(Value::Number(a + b))
            }
        },
        builtin! {
            mul |i, env| (
                eval a: Ty::Number => Value::Number(a),
                eval b: Ty::Number => Value::Number(b)
            ) {
                Ok(Value::Number(a * b))
            }
        },
        builtin! {
            println |i, env| (
                eval v: Ty::Any => v
            ) {
                println!("{v}");
                Ok(atom!(unit))
            }
        },
        builtin! {
            def |i, env| (
                raw name: Ty::Ident => Expr{kind: ExprKind::Ident(name), span},
                eval value: Ty::Any => value
            ) {
                env.set(name.clone(), value);
                Ok(atom!(unit))
            }
        },
        builtin! {
            lambda |i, env| (
                raw raw_params: Ty::List => Expr {kind: ExprKind::List(raw_params), span},
                raw body: Ty::Any => body
            ) {
                let mut params = Vec::new();

                for p in raw_params {
                    let ExprKind::Ident(name) = &p.kind else {
                        miette::bail!(
                            "Lambda parameters must be identifiers"
                        );
                    };

                    params.push(Parameter {
                        name: name.clone(),
                    });
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
                eval a: Ty::Any => a,
                eval b: Ty::Any => b,
                ) {

                Ok(Value::Product(Box::new(a), Box::new(b)))
            }
        },
    ]
    .into()
}
