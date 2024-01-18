// This file is a reference for the semantics of tree_unique_args

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Parallel,
    Sequential,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Id(i64);

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    Boolean(bool),
    Unit,
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Get(Box<Expr>, usize),
    Print(Box<Expr>),
    Read(Box<Expr>),
    Write(Box<Expr>, Box<Expr>),
    All(Order, Vec<Expr>),
    Switch(Box<Expr>, Vec<Expr>),
    Loop(Id, Box<Expr>, Box<Expr>),
    Let(Id, Box<Expr>, Box<Expr>),
    Arg(Id),
    Function(Id, Box<Expr>),
    Call(Id, Box<Expr>),
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
        Expr::Add(e1, e2) | Expr::Sub(e1, e2) | Expr::Mul(e1, e2) => {
            expect_type(e1, Type::Num)?;
            expect_type(e2, Type::Num)?;
            Ok(Type::Num)
        }
        Expr::LessThan(e1, e2) => {
            expect_type(e1, Type::Num)?;
            expect_type(e2, Type::Num)?;
            Ok(Type::Boolean)
        }
        Expr::And(e1, e2) | Expr::Or(e1, e2) => {
            expect_type(e1, Type::Num)?;
            expect_type(e2, Type::Num)?;
            Ok(Type::Boolean)
        }
        Expr::Not(e1) => {
            expect_type(e1, Type::Num)?;
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
        Expr::Loop(_, input, pred_output) => {
            let input_ty = typecheck(input, arg_ty)?;
            let pred_output_ty = typecheck(pred_output, &Some(input_ty.clone()))?;
            let expected_ty = Type::Tuple(vec![Type::Boolean, input_ty.clone()]);
            if pred_output_ty != expected_ty {
                return Err(TypeError::ExpectedType(
                    *pred_output.clone(),
                    expected_ty,
                    pred_output_ty,
                ));
            }
            Ok(input_ty)
        }
        Expr::Let(_, input, output) => {
            let input_ty = typecheck(input, arg_ty)?;
            typecheck(output, &Some(input_ty.clone()))
        }
        Expr::Arg(_) => arg_ty.clone().ok_or(TypeError::NoArg(e.clone())),
        // TODO: add an environment for functions so we can typecheck function calls correctly
        Expr::Function(_, output) => typecheck(output, &None),
        Expr::Call(_, arg) => typecheck(arg, arg_ty),
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
        Expr::Mul(e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("mul")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("mul")
            };
            Value::Num(n1 * n2)
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
        Expr::And(e1, e2) => {
            let Value::Boolean(b1) = interpret(e1, arg, vm) else {
                panic!("and")
            };
            let Value::Boolean(b2) = interpret(e2, arg, vm) else {
                panic!("and")
            };
            Value::Boolean(b1 && b2)
        }
        Expr::Or(e1, e2) => {
            let Value::Boolean(b1) = interpret(e1, arg, vm) else {
                panic!("or")
            };
            let Value::Boolean(b2) = interpret(e2, arg, vm) else {
                panic!("or")
            };
            Value::Boolean(b1 || b2)
        }
        Expr::Not(e1) => {
            let Value::Boolean(b1) = interpret(e1, arg, vm) else {
                panic!("not")
            };
            Value::Boolean(!b1)
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
        Expr::Loop(_, input, pred_output) => {
            let mut vals = interpret(input, arg, vm);
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
        Expr::Function(_, _) | Expr::Call(_, _) => todo!("interpret functions and calls"),
    }
}

#[test]
fn test_interpreter() {
    // numbers 1-10
    let e = Expr::Loop(
        Id(0),
        Box::new(Expr::Num(1)),
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
            Expr::Loop(
                Id(0),
                Box::new(Expr::Num(2)),
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
                                    Expr::Add(Box::new(Expr::Arg(Id(0))), Box::new(Expr::Num(1))),
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

#[derive(Debug, thiserror::Error)]
pub enum ExprParseError {
    #[error("invalid ListExpr")]
    InvalidListExpr,
    #[error("invalid Id")]
    InvalidId,
    #[error("literals must be wrapped")]
    UnwrappedLiteral,
    #[error("expected a grounded term, got a term with {0:?} in it")]
    UngroundedTerm(String),
    #[error("expected Get index to be positive")]
    NegativeGetIndex,
    #[error("order had arguments")]
    InvalidOrderArguments,
    #[error("expected Parallel or Sequential, found {0:?}")]
    InvalidOrder(String),
    #[error("unknown function {0:?} with arguments {1:?}")]
    UnknownFunction(String, Vec<egglog::ast::Expr>),
    #[error("{0}")]
    Egglog(String),
}

impl std::str::FromStr for Expr {
    type Err = ExprParseError;
    fn from_str(s: &str) -> Result<Expr, ExprParseError> {
        fn list_expr_to_vec(e: &egglog::ast::Expr) -> Result<Vec<Expr>, ExprParseError> {
            if let egglog::ast::Expr::Call(f, xs) = e {
                match (f.as_str(), xs.as_slice()) {
                    ("Nil", []) => return Ok(Vec::new()),
                    ("Cons", [head, tail]) => {
                        let head = egglog_expr_to_expr(head)?;
                        let mut tail = list_expr_to_vec(tail)?;
                        tail.insert(0, head);
                        return Ok(tail);
                    }
                    _ => {}
                }
            }
            Err(ExprParseError::InvalidListExpr)
        }
        fn egglog_expr_to_id(e: &egglog::ast::Expr) -> Result<Id, ExprParseError> {
            if let egglog::ast::Expr::Call(f, xs) = e {
                if let ("Id", [egglog::ast::Expr::Lit(egglog::ast::Literal::Int(int))]) =
                    (f.as_str(), xs.as_slice())
                {
                    return Ok(Id(*int));
                }
            }
            Err(ExprParseError::InvalidId)
        }
        fn egglog_expr_to_expr(e: &egglog::ast::Expr) -> Result<Expr, ExprParseError> {
            match e {
                egglog::ast::Expr::Lit(_) => Err(ExprParseError::UnwrappedLiteral),
                egglog::ast::Expr::Var(s) => {
                    Err(ExprParseError::UngroundedTerm(s.as_str().to_owned()))
                }
                egglog::ast::Expr::Call(f, xs) => match (f.as_str(), xs.as_slice()) {
                    ("Num", [_id, egglog::ast::Expr::Lit(egglog::ast::Literal::Int(i))]) => {
                        Ok(Expr::Num(*i))
                    }
                    ("Boolean", [_id, egglog::ast::Expr::Lit(egglog::ast::Literal::Bool(b))]) => {
                        Ok(Expr::Boolean(*b))
                    }
                    ("UnitExpr", [_id]) => Ok(Expr::Unit),
                    ("Add", [x, y]) => Ok(Expr::Add(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("Sub", [x, y]) => Ok(Expr::Sub(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("Mul", [x, y]) => Ok(Expr::Mul(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("LessThan", [x, y]) => Ok(Expr::LessThan(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("And", [x, y]) => Ok(Expr::And(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("Or", [x, y]) => Ok(Expr::Or(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("Not", [x]) => Ok(Expr::Not(Box::new(egglog_expr_to_expr(x)?))),
                    ("Get", [x, egglog::ast::Expr::Lit(egglog::ast::Literal::Int(i))]) => {
                        Ok(Expr::Get(
                            Box::new(egglog_expr_to_expr(x)?),
                            (*i).try_into()
                                .map_err(|_| ExprParseError::NegativeGetIndex)?,
                        ))
                    }
                    ("Print", [x]) => Ok(Expr::Print(Box::new(egglog_expr_to_expr(x)?))),
                    ("Read", [x]) => Ok(Expr::Read(Box::new(egglog_expr_to_expr(x)?))),
                    ("Write", [x, y]) => Ok(Expr::Write(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("All", [egglog::ast::Expr::Call(order, empty), xs]) => {
                        if !empty.is_empty() {
                            return Err(ExprParseError::InvalidOrderArguments);
                        }
                        let order = match order.as_str() {
                            "Parallel" => Ok(Order::Parallel),
                            "Sequential" => Ok(Order::Sequential),
                            s => Err(ExprParseError::InvalidOrder(s.to_owned())),
                        }?;
                        Ok(Expr::All(order, list_expr_to_vec(xs)?))
                    }
                    ("Switch", [pred, branches]) => Ok(Expr::Switch(
                        Box::new(egglog_expr_to_expr(pred)?),
                        list_expr_to_vec(branches)?,
                    )),
                    ("Loop", [id, input, other]) => Ok(Expr::Loop(
                        egglog_expr_to_id(id)?,
                        Box::new(egglog_expr_to_expr(input)?),
                        Box::new(egglog_expr_to_expr(other)?),
                    )),
                    ("Let", [id, input, other]) => Ok(Expr::Let(
                        egglog_expr_to_id(id)?,
                        Box::new(egglog_expr_to_expr(input)?),
                        Box::new(egglog_expr_to_expr(other)?),
                    )),
                    ("Arg", [id]) => Ok(Expr::Arg(egglog_expr_to_id(id)?)),
                    ("Function", [id, body]) => Ok(Expr::Function(
                        egglog_expr_to_id(id)?,
                        Box::new(egglog_expr_to_expr(body)?),
                    )),
                    ("Call", [id, arg]) => Ok(Expr::Call(
                        egglog_expr_to_id(id)?,
                        Box::new(egglog_expr_to_expr(arg)?),
                    )),
                    (f, xs) => Err(ExprParseError::UnknownFunction(f.to_owned(), xs.to_vec())),
                },
            }
        }
        let parser = egglog::ast::parse::ExprParser::new();
        let egglog_expr = parser
            .parse(s)
            .map_err(|e| ExprParseError::Egglog(format!("{e}")))?;
        egglog_expr_to_expr(&egglog_expr)
    }
}

#[test]
fn test_expr_parser() {
    let s = "(Loop
(Id 1)
(Num (Id 0) 1)
(All (Sequential)
    (Cons (LessThan (Num (Id 1) 2) (Num (Id 1) 3))
        (Cons (Switch (Boolean (Id 1) true) (Cons (Num (Id 1) 4) (Cons (Num (Id 1) 5) (Nil))))
            (Nil)))))
";
    let build = s.parse::<Expr>().unwrap();
    let check = Expr::Loop(
        Id(1),
        Box::new(Expr::Num(1)),
        Box::new(Expr::All(
            Order::Sequential,
            vec![
                Expr::LessThan(Box::new(Expr::Num(2)), Box::new(Expr::Num(3))),
                Expr::Switch(
                    Box::new(Expr::Boolean(true)),
                    vec![Expr::Num(4), Expr::Num(5)],
                ),
            ],
        )),
    );
    assert_eq!(build, check);
}
