// This file is a reference for the semantics of tree_unique_args

#[cfg(test)]
use crate::ast::{
    add, arg, get, lessthan, num, parallel, sequence, sub, switch, tint, tloop, tprint, tread,
    ttrue, twrite,
};

use crate::{
    expr::Expr,
    expr::{
        Id::{self, Shared, Unique},
        PureBOp, PureUOp,
        TreeType::{self, Bril, Tuple},
        Value,
    },
    expr::{Order, TypeError},
};
use bril_rs::Type::{Bool, Int};
use egglog::ast::Literal;
use std::collections::HashMap;

pub fn typecheck(e: &Expr, arg_ty: &Option<TreeType>) -> Result<TreeType, TypeError> {
    let expect_type = |sub_e: &Expr, expected_ty: TreeType| -> Result<(), TypeError> {
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
        Expr::Program(_) => panic!("Found non top level program."),
        Expr::Num(..) => Ok(Bril(Int)),
        Expr::Boolean(..) => Ok(Bril(Bool)),
        Expr::BOp(op, e1, e2) => {
            let expected = op.input_types();
            expect_type(e1, Bril(expected.0))?;
            expect_type(e2, Bril(expected.1))?;
            Ok(Bril(op.output_type()))
        }
        Expr::UOp(op, e) => {
            expect_type(e, Bril(op.input_type()))?;
            Ok(Bril(op.output_type()))
        }
        Expr::Get(tuple, i) => {
            let ty_tuple = typecheck(tuple, arg_ty)?;
            match ty_tuple {
                TreeType::Tuple(tys) => Ok(tys[*i].clone()),
                _ => Err(TypeError::ExpectedTupleType(
                    *tuple.clone(),
                    ty_tuple.clone(),
                )),
            }
        }
        Expr::Print(e) => {
            // right now, only print nums
            expect_type(e, Bril(Int))?;
            Ok(TreeType::Tuple(vec![]))
        }
        Expr::Read(addr, ty) => {
            // right now, all memory holds nums.
            // read could also take a static type to interpret
            // the memory as.
            expect_type(addr, ty.clone())?;
            Ok(ty.clone())
        }
        Expr::Write(addr, data) => {
            expect_type(addr, Bril(Int))?;
            expect_type(data, Bril(Int))?;
            Ok(TreeType::Tuple(vec![]))
        }
        Expr::All(_, _, exprs) => {
            let tys = exprs
                .iter()
                .map(|expr| typecheck(expr, arg_ty))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TreeType::Tuple(tys))
        }
        Expr::Switch(pred, branches) => {
            expect_type(pred, Bril(Int))?;
            let ty = typecheck(&branches[0], arg_ty)?;
            for branch in branches {
                expect_type(branch, ty.clone())?;
            }
            Ok(ty)
        }
        Expr::Branch(_id, child) => typecheck(child, arg_ty),
        Expr::Loop(_, input, pred_output) => {
            let input_ty = typecheck(input, arg_ty)?;
            let pred_output_ty = typecheck(pred_output, &Some(input_ty.clone()))?;
            let expected_ty = TreeType::Tuple(vec![Bril(Bool), input_ty.clone()]);
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
        Expr::Function(_, _name, in_ty, out_ty, output) => {
            let output_ty = typecheck(output, &Some(in_ty.clone()))?;
            if output_ty != *out_ty {
                return Err(TypeError::ExpectedType(
                    *output.clone(),
                    out_ty.clone(),
                    output_ty,
                ));
            }
            Ok(out_ty.clone())
        }
        Expr::Call(_, _name, arg) => typecheck(arg, arg_ty),
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
        Expr::Program(_) => todo!("interpret programs"),
        Expr::Num(_id, x) => Value::Num(*x),
        Expr::Boolean(_id, x) => Value::Boolean(*x),
        Expr::BOp(PureBOp::Add, e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("add")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("add")
            };
            Value::Num(n1 + n2)
        }
        Expr::BOp(PureBOp::Sub, e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("sub")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("sub")
            };
            Value::Num(n1 - n2)
        }
        Expr::BOp(PureBOp::Mul, e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("mul")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("mul")
            };
            Value::Num(n1 * n2)
        }
        Expr::BOp(PureBOp::LessThan, e1, e2) => {
            let Value::Num(n1) = interpret(e1, arg, vm) else {
                panic!("lessthan")
            };
            let Value::Num(n2) = interpret(e2, arg, vm) else {
                panic!("lessthan")
            };
            Value::Boolean(n1 < n2)
        }
        Expr::BOp(PureBOp::And, e1, e2) => {
            let Value::Boolean(b1) = interpret(e1, arg, vm) else {
                panic!("and")
            };
            let Value::Boolean(b2) = interpret(e2, arg, vm) else {
                panic!("and")
            };
            Value::Boolean(b1 && b2)
        }
        Expr::BOp(PureBOp::Or, e1, e2) => {
            let Value::Boolean(b1) = interpret(e1, arg, vm) else {
                panic!("or")
            };
            let Value::Boolean(b2) = interpret(e2, arg, vm) else {
                panic!("or")
            };
            Value::Boolean(b1 || b2)
        }
        Expr::UOp(PureUOp::Not, e1) => {
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
            Value::Tuple(vec![])
        }
        Expr::Read(e_addr, _ty) => {
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
            Value::Tuple(vec![])
        }
        Expr::All(_, _, exprs) => {
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
        Expr::Branch(_id, child) => interpret(child, arg, vm),
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
        Expr::Function(_, _, _, _, _) | Expr::Call(_, _, _) => {
            todo!("interpret functions and calls")
        }
    }
}

#[test]
fn test_interpreter() {
    // numbers 1-10
    let e = tloop(
        num(1),
        parallel!(
            // pred: i < 10
            lessthan(arg(), num(10)),
            // output
            get(
                parallel!(
                    // i = i + 1
                    add(arg(), num(1)),
                    // print(i)
                    tprint(arg())
                ),
                0,
            )
        ),
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
    let e = sequence!(
        twrite(num(0), num(0)),
        twrite(num(1), num(1)),
        tloop(
            num(2),
            parallel!(
                // pred: i < nth
                lessthan(arg(), num(nth)),
                // output
                Expr::Get(
                    Box::new(Expr::All(
                        Unique(0),
                        Order::Parallel,
                        vec![
                            // i = i + 1
                            add(arg(), num(1)),
                            // mem[i] = mem[i - 1] + mem[i - 2]
                            Expr::Write(
                                Box::new(Expr::Arg(Unique(0))),
                                Box::new(add(
                                    tread(sub(arg(), num(1)), tint()),
                                    tread(sub(arg(), num(2)), tint()),
                                )),
                            ),
                        ],
                    )),
                    0,
                )
            )
        ),
        tread(num(nth), tint())
    );
    let mut vm = VirtualMachine {
        mem: HashMap::new(),
        log: vec![],
    };
    let res = interpret(&e, &None, &mut vm);
    assert_eq!(
        res,
        Value::Tuple(vec![
            Value::Tuple(vec![]),
            Value::Tuple(vec![]),
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
    #[error("expected a type, got {0:?}")]
    UnknownType(egglog::ast::Expr),
    #[error("expected an op, got {0:?}")]
    UnknownOp(egglog::ast::Expr),
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

        fn egglog_type_to_type(e: &egglog::ast::Expr) -> Result<TreeType, ExprParseError> {
            if let egglog::ast::Expr::Call(f, xs) = e {
                match (f.as_str(), xs.as_slice()) {
                    ("BoolT", []) => Ok(Bril(Bool)),
                    ("IntT", []) => Ok(Bril(Int)),
                    ("TupleT", xs) => {
                        let tys = xs
                            .iter()
                            .map(egglog_type_to_type)
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Tuple(tys))
                    }
                    _ => Err(ExprParseError::UnknownType(e.clone())),
                }
            } else {
                Err(ExprParseError::UnknownType(e.clone()))
            }
        }

        fn egglog_expr_to_id(e: &egglog::ast::Expr) -> Result<Id, ExprParseError> {
            if let egglog::ast::Expr::Call(f, xs) = e {
                match (f.as_str(), xs.as_slice()) {
                    ("Id", [egglog::ast::Expr::Lit(egglog::ast::Literal::Int(int))]) => {
                        Ok(Unique(*int))
                    }
                    ("Shared", []) => Ok(Shared),
                    _ => Err(ExprParseError::InvalidId),
                }
            } else {
                Err(ExprParseError::InvalidId)
            }
        }

        fn egglog_binop_to_binop(e: &egglog::ast::Expr) -> Result<PureBOp, ExprParseError> {
            if let egglog::ast::Expr::Call(str, xs) = e {
                if let (Ok(op), []) = (PureBOp::from_str(&str.to_string()), xs.as_slice()) {
                    Ok(op)
                } else {
                    Err(ExprParseError::UnknownOp(e.clone()))
                }
            } else {
                Err(ExprParseError::UnknownOp(e.clone()))
            }
        }

        fn egglog_unaryop_to_unaryop(e: &egglog::ast::Expr) -> Result<PureUOp, ExprParseError> {
            if let egglog::ast::Expr::Call(str, xs) = e {
                if let (Ok(op), []) = (PureUOp::from_str(&str.to_string()), xs.as_slice()) {
                    Ok(op)
                } else {
                    Err(ExprParseError::UnknownOp(e.clone()))
                }
            } else {
                Err(ExprParseError::UnknownOp(e.clone()))
            }
        }

        fn egglog_expr_to_expr(e: &egglog::ast::Expr) -> Result<Expr, ExprParseError> {
            match e {
                egglog::ast::Expr::Lit(_) => Err(ExprParseError::UnwrappedLiteral),
                egglog::ast::Expr::Var(s) => {
                    Err(ExprParseError::UngroundedTerm(s.as_str().to_owned()))
                }
                egglog::ast::Expr::Call(f, xs) => match (f.as_str(), xs.as_slice()) {
                    ("Num", [id, egglog::ast::Expr::Lit(egglog::ast::Literal::Int(i))]) => {
                        Ok(Expr::Num(egglog_expr_to_id(id)?, *i))
                    }
                    ("Boolean", [id, egglog::ast::Expr::Lit(egglog::ast::Literal::Bool(b))]) => {
                        Ok(Expr::Boolean(egglog_expr_to_id(id)?, *b))
                    }
                    ("BOp", [op, x, y]) => Ok(Expr::BOp(
                        egglog_binop_to_binop(op)?,
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("UOp", [op, x]) => Ok(Expr::UOp(
                        egglog_unaryop_to_unaryop(op)?,
                        Box::new(egglog_expr_to_expr(x)?),
                    )),
                    ("Get", [x, egglog::ast::Expr::Lit(egglog::ast::Literal::Int(i))]) => {
                        Ok(Expr::Get(
                            Box::new(egglog_expr_to_expr(x)?),
                            (*i).try_into()
                                .map_err(|_| ExprParseError::NegativeGetIndex)?,
                        ))
                    }
                    ("Print", [x]) => Ok(Expr::Print(Box::new(egglog_expr_to_expr(x)?))),
                    ("Read", [x, ty]) => Ok(Expr::Read(
                        Box::new(egglog_expr_to_expr(x)?),
                        egglog_type_to_type(ty)?,
                    )),
                    ("Write", [x, y]) => Ok(Expr::Write(
                        Box::new(egglog_expr_to_expr(x)?),
                        Box::new(egglog_expr_to_expr(y)?),
                    )),
                    ("All", [id, egglog::ast::Expr::Call(order, empty), xs]) => {
                        if !empty.is_empty() {
                            return Err(ExprParseError::InvalidOrderArguments);
                        }
                        let order = match order.as_str() {
                            "Parallel" => Ok(Order::Parallel),
                            "Sequential" => Ok(Order::Sequential),
                            s => Err(ExprParseError::InvalidOrder(s.to_owned())),
                        }?;
                        Ok(Expr::All(
                            egglog_expr_to_id(id)?,
                            order,
                            list_expr_to_vec(xs)?,
                        ))
                    }
                    ("Switch", [pred, branches]) => Ok(Expr::Switch(
                        Box::new(egglog_expr_to_expr(pred)?),
                        list_expr_to_vec(branches)?,
                    )),
                    ("Branch", [id, child]) => Ok(Expr::Branch(
                        egglog_expr_to_id(id)?,
                        Box::new(egglog_expr_to_expr(child)?),
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
                    (
                        "Function",
                        [id, egglog::ast::Expr::Lit(Literal::String(name)), in_ty, out_ty, body],
                    ) => Ok(Expr::Function(
                        egglog_expr_to_id(id)?,
                        name.to_string(),
                        egglog_type_to_type(in_ty)?,
                        egglog_type_to_type(out_ty)?,
                        Box::new(egglog_expr_to_expr(body)?),
                    )),
                    ("Call", [id, egglog::ast::Expr::Lit(Literal::String(name)), arg]) => {
                        Ok(Expr::Call(
                            egglog_expr_to_id(id)?,
                            name.to_string(),
                            Box::new(egglog_expr_to_expr(arg)?),
                        ))
                    }
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
(All (Id 1) (Sequential)
    (Cons (BOp (LessThan) (Num (Id 1) 2) (Num (Id 1) 3))
        (Cons (Switch (Boolean (Id 1) true) (Cons (Branch (Id 2) (Num (Id 2) 4)) (Cons (Branch (Id 3) (Num (Id 3) 5)) (Nil))))
            (Nil)))))
";
    let build = s.parse::<Expr>().unwrap();
    let check = tloop(
        num(1),
        sequence!(
            lessthan(num(2), num(3)),
            switch!(
                ttrue(),
                Expr::Branch(Unique(2), Box::new(Expr::Num(Unique(2), 4))),
                Expr::Branch(Unique(3), Box::new(Expr::Num(Unique(2), 5))),
            )
        ),
    );
    assert_eq!(build, check);
}
