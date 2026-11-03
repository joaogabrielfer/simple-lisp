use crate::{
    interpreter::RuntimeErrorKind,
    parser::{Expr, ExprKind},
    pattern::{Literal, Pattern},
    values::{Ty, Value},
};

macro_rules! bind_arg_raw {
    ($i:expr, $env:expr, $expr:expr, $expected:expr, $pat:pat) => {
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
}

macro_rules! bind_eval_arg {
    ($i:expr, $env:expr, $expr:expr, Any, $param:ident) => {
        let expr = $expr;
        let value = $i.run(expr, $env)?;
        let got = value.ty();

        let $param = value;
    };
    ($i:expr, $env:expr, $expr:expr, $expected:ident, $param:ident) => {
        let expr = $expr;
        let value = $i.run(expr, $env)?;
        let got = value.ty();

        let Value::$expected($param) = value else {
            return Err($i.error(
                RuntimeErrorKind::ExpectedValue {
                    expected: Ty::$expected,
                    got: Some(got),
                },
                expr.span,
            ));
        };
    };
}

macro_rules! builtin {
    // ---- RAW arguments ----

    // No pattern: use the parameter name
    (@arg $i:ident, $env:ident, $expr:expr,
        raw $param:ident: $ty:ident
    ) => {
        builtin!(@arg $i, $env, $expr,
            raw $param: $ty => $param);
    };

    // Explicit pattern
    (@arg $i:ident, $env:ident, $expr:expr,
        raw $param:ident: $ty:ident => $pat:pat
    ) => {
        let expr = $expr;

        #[allow(irrefutable_let_patterns)]
        let $pat = expr else {
            return Err($i.error(
                RuntimeErrorKind::ExpectedValue {
                    expected: Ty::$ty,
                    got: None,
                },
                expr.span,
            ));
        };
    };

    // ---- EVALUATED arguments ----

    // Any: no type checking
    (@arg $i:ident, $env:ident, $expr:expr,
        eval $param:ident: Any
    ) => {
        let $param = $i.run($expr, $env)?;
    };

    // Any with an explicit pattern
    (@arg $i:ident, $env:ident, $expr:expr,
        eval $param:ident: Any => $pat:pat
    ) => {
        let expr = $expr;
        let value = $i.run(expr, $env)?;

        #[allow(irrefutable_let_patterns)]
        let $pat = value else {
            return Err($i.error(
                RuntimeErrorKind::ExpectedValue {
                    expected: Ty::Any,
                    got: Some(value.ty()),
                },
                expr.span,
            ));
        };
    };

    // Typed argument without pattern
    (@arg $i:ident, $env:ident, $expr:expr,
        eval $param:ident: $ty:ident
    ) => {
        builtin!(@arg $i, $env, $expr,
            eval $param: $ty => $param);
    };

    // Typed argument with pattern
    (@arg $i:ident, $env:ident, $expr:expr,
        eval $param:ident: $ty:ident => $pat:pat
    ) => {
        let expr = $expr;
        let value = $i.run(expr, $env)?;
        let got = value.ty();

        #[allow(irrefutable_let_patterns)]
        let Value::$ty($pat) = value else {
            return Err($i.error(
                RuntimeErrorKind::ExpectedValue {
                    expected: Ty::$ty,
                    got: Some(got),
                },
                expr.span,
            ));
        };
    };

    // ---- PUBLIC ENTRY POINT ----

    (
        $name:ident |$i:ident, $env:ident| (
            $(
                $mode:ident $param:ident: $ty:ident
                $(=> $pat:pat)?
            ),* $(,)?
        ) $body:block
    ) => {
        (
            stringify!($name).to_string(),
            Value::BuiltinFunction {
                params: vec![
                    $(
                        Pattern::Typed(
                            Box::new(Pattern::Bind(
                                stringify!($param).to_string()
                            )),
                            Ty::$ty
                        ),
                    )*
                ],

                #[allow(unused_variables, unused_mut)]
                body: |$i, $env, args| {
                    let expected = 0usize $(
                        + {
                            let _ = stringify!($param);
                            1usize
                        }
                    )*;

                    if args.len() != expected {
                        miette::bail!(
                            "Expected {} arguments, got {}",
                            expected,
                            args.len()
                        );
                    }

                    let mut iter = args.iter();

                    $(
                        builtin!(@arg
                            $i,
                            $env,
                            iter.next().unwrap(),
                            $mode $param: $ty $(=> $pat)?
                        );
                    )*

                    $body
                },
            }
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
                eval a: Number,
                eval b: Number
            ) {
                Ok(Value::Number(a + b))
            }
        },
        builtin! {
            mul |i, env| (
                eval a: Number,
                eval b: Number
            ) {
                Ok(Value::Number(a * b))
            }
        },
        builtin! {
            println |i, env| (
                eval v: Any,
            ) {
                println!("{v}");
                Ok(atom!(unit))
            }
        },
        builtin! {
            def |i, env| (
                raw name: Any => e,
                eval value: Any,
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
                raw raw_params: List => Expr {kind: ExprKind::List(raw_params), span},
                raw body: Any => body
            ) {
                let mut params = Vec::new();

                for p in raw_params {
                    let pat = match p.kind.clone() {
                        ExprKind::AtomLit(a) => Pattern::Literal(Literal::Atom(a)),
                        ExprKind::NumberLit(n) => Pattern::Literal(Literal::Number(n)),
                        ExprKind::StringLit(s) => Pattern::Literal(Literal::String(s)),
                        ExprKind::Ident(i) => Pattern::Bind(i),
                        ExprKind::List(exprs) => {
                            miette::bail!("list cannot be parameters as of now");
                        }
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
                eval a: Any,
                eval b: Any
                ) {

                Ok(Value::Product(Box::new(a), Box::new(b)))
            }
        },
        builtin! {
            case |i, env| (
                eval value: Any,
                raw branches: List => Expr { kind: ExprKind::List(list), span }
                ) {

                for expr in list {
                    let ExprKind::List(ref inner) = expr.kind else {
                        return Err(i.error(RuntimeErrorKind::ExpectedExpression { expected: Ty::List, got: Some(expr.kind.clone()) }, *span))
                    };

                    let [ref pattern, ref body] = inner[..] else {
                        todo!("expected 2 expressions in list")
                    };

                    let pattern: Pattern = pattern.try_into()?;

                    if let Some(bindings) = pattern.match_value(value.clone()) {
                        let mut env = env.clone();
                        env.bind(bindings);
                        return i.run(body, &mut env)
                    }
                }


                Err(i.error(RuntimeErrorKind::UnexhaustivePatternMatch, *span))
            }
        }
    ]
    .into()
}
