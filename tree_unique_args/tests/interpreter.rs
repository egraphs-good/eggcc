// This file is a reference for the semantics of tree_unique_args

use std::collections::HashMap;

#[derive(Clone)]
pub enum Order {
    Parallel,
    Sequential,
}

#[derive(Clone)]
pub struct Id(i64);

#[derive(Clone)]
pub enum Expr {
    Num(i64),
    Boolean(bool),
    Unit,
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    // TODO: other pure ops
    Get(Box<Expr>, usize),
    Print(Box<Expr>),
    Read(Box<Expr>),
    Write(Box<Expr>, Box<Expr>),
    All(Order, Vec<Expr>),
    Switch(Box<Expr>, Vec<Expr>),
    Loop(Id, Box<Expr>),
    Let(Id, Box<Expr>, Box<Expr>),
    Arg(Id),
    // TODO: call and functions
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(i64),
    Boolean(bool),
    Unit,
    Tuple(Vec<Value>),
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Num,
    Boolean,
    Unit,
    Tuple(Vec<Type>),
}

pub enum TypeError {
    ExpectedType(Expr, Type, Type),
    ExpectedTupleType(Expr, Type),
    ExpectedLoopOutputType(Expr, Type),
    NoArg(Expr),
}

pub fn typecheck(e: &Expr, arg_ty: &Option<Type>) -> Result<Type, TypeError> {
    let expect_type = |sub_e: &Expr, expected_ty: Type| -> Result<(), TypeError> {
        let actual_ty = typecheck(sub_e, arg_ty)?;
        if actual_ty == expected_ty {
            Ok(())
        } else {
            Err(TypeError::ExpectedType(
                sub_e.clone(),
                expected_ty,
                actual_ty,
            ))
        }
    };
    match e {
        Expr::Num(_) => Ok(Type::Num),
        Expr::Boolean(_) => Ok(Type::Boolean),
        Expr::Unit => Ok(Type::Unit),
        Expr::Add(e1, e2) => {
            expect_type(e1, Type::Num)?;
            expect_type(e2, Type::Num)?;
            Ok(Type::Num)
        }
        Expr::Sub(e1, e2) => {
            expect_type(e1, Type::Num)?;
            expect_type(e2, Type::Num)?;
            Ok(Type::Num)
        }
        Expr::LessThan(e1, e2) => {
            expect_type(e1, Type::Num)?;
            expect_type(e2, Type::Num)?;
            Ok(Type::Boolean)
        }
        Expr::Get(tuple, i) => {
            let ty_tuple = typecheck(tuple, arg_ty)?;
            match ty_tuple {
                Type::Tuple(tys) => Ok(tys[*i].clone()),
                _ => Err(TypeError::ExpectedTupleType(
                    *tuple.clone(),
                    ty_tuple.clone(),
                )),
            }
        }
        Expr::Print(e) => {
            // right now, only print nums
            expect_type(e, Type::Num)?;
            Ok(Type::Unit)
        }
        Expr::Read(addr) => {
            // right now, all memory holds nums.
            // read could also take a static type to interpret
            // the memory as.
            expect_type(addr, Type::Num)?;
            Ok(Type::Num)
        }
        Expr::Write(addr, data) => {
            expect_type(addr, Type::Num)?;
            expect_type(data, Type::Num)?;
            Ok(Type::Unit)
        }
        Expr::All(_, exprs) => {
            let tys = exprs
                .iter()
                .map(|expr| typecheck(expr, arg_ty))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Tuple(tys))
        }
        Expr::Switch(pred, branches) => {
            expect_type(pred, Type::Num)?;
            let ty = typecheck(&branches[0], arg_ty)?;
            for branch in branches {
                expect_type(branch, ty.clone())?;
            }
            Ok(ty)
        }
        Expr::Loop(_, pred_output) => {
            let args_types = arg_ty.clone().ok_or(TypeError::NoArg(e.clone()))?;
            let pred_output_ty = typecheck(pred_output, &Some(args_types.clone()))?;
            let expected_ty = Type::Tuple(vec![Type::Boolean, args_types.clone()]);
            if pred_output_ty != expected_ty {
                return Err(TypeError::ExpectedType(
                    *pred_output.clone(),
                    expected_ty,
                    pred_output_ty,
                ));
            }
            Ok(args_types)
        }
        Expr::Let(_, input, output) => {
            let input_ty = typecheck(input, arg_ty)?;
            typecheck(output, &Some(input_ty.clone()))
        }
        Expr::Arg(_) => arg_ty.clone().ok_or(TypeError::NoArg(e.clone())),
    }
}

pub struct VirtualMachine {
    mem: HashMap<usize, Value>,
    log: Vec<i64>,
}

// TODO: refactor to return a Result<Value, RuntimeError>
// struct RuntimeError { BadRead(Value) }

// assumes e typechecks and that memory is written before read
pub fn interpret(e: &Expr, arg: &Option<Value>, vm: &mut VirtualMachine) -> Value {
    match e {
        Expr::Num(x) => Value::Num(*x),
        Expr::Boolean(x) => Value::Boolean(*x),
        Expr::Unit => Value::Unit,
        Expr::Add(e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("add")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("add")
            };
            Value::Num(n1 + n2)
        }
        Expr::Sub(e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("sub")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("sub")
            };
            Value::Num(n1 - n2)
        }
        Expr::LessThan(e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("lessthan")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("lessthan")
            };
            Value::Boolean(n1 < n2)
        }
        Expr::Get(e_tuple, i) => {
            let Value::Tuple(vals) = interpret(e_tuple, arg, vm) else {
                panic!("get")
            };
            vals[*i].clone()
        }
        Expr::Print(e) => {
            let Value::Num(n) = interpret(e, arg, vm) else {
                panic!("print")
            };
            vm.log.push(n);
            Value::Unit
        }
        Expr::Read(e_addr) => {
            let Value::Num(addr) = interpret(e_addr, arg, vm) else {
                panic!("read")
            };
            vm.mem[&(addr as usize)].clone()
        }
        Expr::Write(e_addr, e_data) => {
            let Value::Num(addr) = interpret(e_addr, arg, vm) else {
                panic!("write")
            };
            let data = interpret(e_data, arg, vm);
            vm.mem.insert(addr as usize, data);
            Value::Unit
        }
        Expr::All(_, exprs) => {
            // this always executes sequentially (which is a valid way to
            // execute parallel tuples)
            let vals = exprs
                .iter()
                .map(|expr| interpret(expr, arg, vm))
                .collect::<Vec<_>>();
            Value::Tuple(vals)
        }
        Expr::Switch(pred, branches) => {
            let Value::Num(pred) = interpret(pred, arg, vm) else {
                panic!("switch")
            };
            interpret(&branches[pred as usize], arg, vm)
        }
        Expr::Loop(_, pred_output) => {
            let mut vals = arg.clone().unwrap_or(Value::Unit);
            let mut pred = Value::Boolean(true);
            while pred == Value::Boolean(true) {
                let Value::Tuple(pred_output_val) = interpret(pred_output, &Some(vals.clone()), vm)
                else {
                    panic!("loop")
                };
                let [new_pred, new_vals] = pred_output_val.as_slice() else {
                    panic!("loop")
                };
                pred = new_pred.clone();
                vals = new_vals.clone();
            }
            vals
        }
        Expr::Let(_, input, output) => {
            let vals = interpret(input, arg, vm);
            interpret(output, &Some(vals.clone()), vm)
        }
        Expr::Arg(_) => {
            let Some(v) = arg else { panic!("arg") };
            v.clone()
        }
    }
}

#[test]
fn test_interpreter() {
    // numbers 1-10
    let e = Expr::Let(
        Id(4),
        Box::new(Expr::Num(1)),
        Box::new(Expr::Loop(
            Id(0),
            Box::new(Expr::All(
                Order::Parallel,
                vec![
                    // pred: i < 10
                    Expr::LessThan(Box::new(Expr::Arg(Id(0))), Box::new(Expr::Num(10))),
                    // output
                    Expr::Get(
                        Box::new(Expr::All(
                            Order::Parallel,
                            vec![
                                // i = i + 1
                                Expr::Add(Box::new(Expr::Arg(Id(0))), Box::new(Expr::Num(1))),
                                // print(i)
                                Expr::Print(Box::new(Expr::Arg(Id(0)))),
                            ],
                        )),
                        0,
                    ),
                ],
            )),
        )),
    );
    let mut vm = VirtualMachine {
        mem: HashMap::new(),
        log: vec![],
    };
    let res = interpret(&e, &None, &mut vm);
    assert_eq!(res, Value::Num(11));
    assert_eq!(vm.log, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
}

#[test]
fn test_interpreter_fib_using_memory() {
    let nth = 10;
    let fib_nth = 55;
    let e = Expr::All(
        Order::Sequential,
        vec![
            Expr::Write(Box::new(Expr::Num(0)), Box::new(Expr::Num(0))),
            Expr::Write(Box::new(Expr::Num(1)), Box::new(Expr::Num(1))),
            Expr::Let(
                Id(3),
                Box::new(Expr::Num(2)),
                Box::new(Expr::Loop(
                    Id(0),
                    Box::new(Expr::All(
                        Order::Parallel,
                        vec![
                            // pred: i < nth
                            Expr::LessThan(Box::new(Expr::Arg(Id(0))), Box::new(Expr::Num(nth))),
                            // output
                            Expr::Get(
                                Box::new(Expr::All(
                                    Order::Parallel,
                                    vec![
                                        // i = i + 1
                                        Expr::Add(
                                            Box::new(Expr::Arg(Id(0))),
                                            Box::new(Expr::Num(1)),
                                        ),
                                        // mem[i] = mem[i - 1] + mem[i - 2]
                                        Expr::Write(
                                            Box::new(Expr::Arg(Id(0))),
                                            Box::new(Expr::Add(
                                                Box::new(Expr::Read(Box::new(Expr::Sub(
                                                    Box::new(Expr::Arg(Id(0))),
                                                    Box::new(Expr::Num(1)),
                                                )))),
                                                Box::new(Expr::Read(Box::new(Expr::Sub(
                                                    Box::new(Expr::Arg(Id(0))),
                                                    Box::new(Expr::Num(2)),
                                                )))),
                                            )),
                                        ),
                                    ],
                                )),
                                0,
                            ),
                        ],
                    )),
                )),
            ),
            Expr::Read(Box::new(Expr::Num(nth))),
        ],
    );
    let mut vm = VirtualMachine {
        mem: HashMap::new(),
        log: vec![],
    };
    let res = interpret(&e, &None, &mut vm);
    assert_eq!(
        res,
        Value::Tuple(vec![
            Value::Unit,
            Value::Unit,
            Value::Num(nth + 1),
            Value::Num(fib_nth)
        ])
    );
    assert_eq!(vm.mem[&(nth as usize)], Value::Num(fib_nth));
    assert!(!vm.mem.contains_key(&(nth as usize + 1)));
}
