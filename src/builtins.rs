use crate::{
    interpreter::{Interpreter, Parameter, Value},
    parser::Expr,
};

pub fn builtins() -> Vec<(String, Value)> {
    [
        (
            "add".to_string(),
            Value::BuiltinFunction {
                params: vec![
                    Parameter {
                        name: "a".to_string(),
                    },
                    Parameter {
                        name: "b".to_string(),
                    },
                ],
                body: |env, args| {
                    if args.len() != 2 {
                        todo!("return wrong parameter count error")
                    }

                    let mut args = args.to_vec();
                    args.reverse();

                    let Ok(Value::Number(a)) = Interpreter::run(&args.pop().unwrap(), env) else {
                        todo!("report runtime type error")
                    };

                    let Ok(Value::Number(b)) = Interpreter::run(&args.pop().unwrap(), env) else {
                        todo!("report runtime type error")
                    };

                    Ok(Value::Number(a + b))
                },
            },
        ),
        (
            "mul".to_string(),
            Value::BuiltinFunction {
                params: vec![
                    Parameter {
                        name: "a".to_string(),
                    },
                    Parameter {
                        name: "b".to_string(),
                    },
                ],
                body: |env, args| {
                    if args.len() != 2 {
                        todo!("return wrong parameter count error")
                    }

                    let mut args = args.to_vec();
                    args.reverse();

                    let Ok(Value::Number(a)) = Interpreter::run(&args.pop().unwrap(), env) else {
                        todo!("report runtime type error")
                    };

                    let Ok(Value::Number(b)) = Interpreter::run(&args.pop().unwrap(), env) else {
                        todo!("report runtime type error")
                    };

                    Ok(Value::Number(a * b))
                },
            },
        ),
        (
            "println".to_string(),
            Value::BuiltinFunction {
                params: vec![Parameter {
                    name: "s".to_string(),
                }],
                body: |env, args| {
                    if args.len() != 1 {
                        todo!("return wrong parameter count error")
                    }

                    let mut args = args.to_vec();
                    args.reverse();

                    let v = Interpreter::run(&args.pop().unwrap(), env)?;

                    println!("{v}");

                    Ok(Value::Atom("unit".to_string()))
                },
            },
        ),
        (
            "def".to_string(),
            Value::BuiltinFunction {
                params: vec![
                    Parameter {
                        name: "name".to_string(),
                    },
                    Parameter {
                        name: "value".to_string(),
                    },
                ],
                body: |env, args| {
                    if args.len() != 2 {
                        todo!("return wrong parameter count error")
                    }

                    let mut args = args.to_vec();
                    args.reverse();

                    let Expr::Ident(i) = args.pop().unwrap() else {
                        todo!("report runtime type error")
                    };

                    let v = Interpreter::run(&args.pop().unwrap(), env)?;

                    env.set(i, v);

                    Ok(Value::Atom("ok".to_string()))
                },
            },
        ),
    ]
    .to_vec()
}
