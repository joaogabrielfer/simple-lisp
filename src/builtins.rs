use crate::{
    interpreter::{
        Interpreter, Parameter,
        Value::{self},
    },
    parser::Expr,
};

macro_rules! bind_arg {
    ($env:expr, $expr:expr, raw, $pat:pat) => {
        let $pat = $expr else {
            todo!("report runtime type error")
        };
    };

    ($env:expr, $expr:expr, eval, $pat:pat) => {
        #[allow(irrefutable_let_patterns)]
        let $pat = Interpreter::run($expr, $env)? else {
            todo!("report runtime type error")
        };
    };
}

macro_rules! builtin {
    (
        $name:ident |$env:ident| (
            $(
                $mode:ident $param:ident : $pat:pat
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
                #[allow(unused_variables)]
                body: |$env, args| {
                    let expected = 0usize $(+ {
                        let _ = stringify!($param);
                        1usize
                    })*;

                    if args.len() != expected {
                        todo!("report arity error")
                    }

                    let mut iter = args.iter();

                    $(
                        bind_arg!(
                            $env,
                            iter.next().unwrap(),
                            $mode,
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
            add |env| (
                eval a: Value::Number(a),
                eval b: Value::Number(b)
            ) {
                Ok(Value::Number(a + b))
            }
        },
        builtin! {
            mul |env| (
                eval a: Value::Number(a),
                eval b: Value::Number(b)
            ) {
                Ok(Value::Number(a * b))
            }
        },
        builtin! {
            println |env| (
                eval v: v
            ) {
                println!("{v}");
                Ok(atom!(unit))
            }
        },
        builtin! {
            def |env| (
                raw name: Expr::Ident(name),
                eval value: value
            ) {
                env.set(name.clone(), value);
                Ok(atom!(unit))
            }
        },
        builtin! {
            lambda |env| (
                raw raw_params: Expr::List(raw_params),
                raw body: Expr::List(body)
            ) {
                let mut params = vec![];
                for p in raw_params {
                    let Expr::Ident(p) = p else {
                        todo!("report wrong argument type error in lambda");
                    };

                    params.push(Parameter { name: p.clone() });
                }
                Ok(Value::Function { params, body: Expr::List(body.clone()) })
            }
        },
    ]
    .into()
}
