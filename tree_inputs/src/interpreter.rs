use std::{collections::HashMap, fmt::Display};

use crate::schema::{BinaryOp, Constant, Expr, RcExpr, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Const(Constant),
    Tuple(Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Const(constant) => write!(f, "{}", constant),
            Tuple(vs) => {
                write!(f, "(")?;
                for v in vs {
                    write!(f, "{}, ", v)?;
                }
                write!(f, ")")
            }
        }
    }
}

use Value::{Const, Tuple};

pub(crate) struct VirtualMachine {
    mem: HashMap<usize, Value>,
    log: Vec<String>,
}

pub struct BrilState {
    pub mem: HashMap<usize, Value>,
    pub log: Vec<String>,
    pub value: Value,
}

pub fn interpret(expr: &RcExpr, arg: &Option<Value>) -> BrilState {
    let mut vm = VirtualMachine {
        mem: HashMap::new(),
        log: vec![],
    };
    let value = vm.interpret(expr, arg);
    BrilState {
        mem: vm.mem,
        log: vm.log,
        value,
    }
}

impl VirtualMachine {
    fn interpret_bop(
        &mut self,
        bop: &BinaryOp,
        e1: &RcExpr,
        e2: &RcExpr,
        arg: &Option<Value>,
    ) -> Value {
        let mut get_int = |e: &RcExpr| match self.interpret(e, arg) {
            Const(Constant::Int(n)) => n,
            _ => panic!(
                "Expected integer in binary operation {:?}. Got {:?}",
                bop, e
            ),
        };
        let mut get_bool = |e: &RcExpr| match self.interpret(e, arg) {
            Const(Constant::Bool(b)) => b,
            _ => panic!(
                "Expected boolean in binary operation {:?}. Got {:?}",
                bop, e
            ),
        };
        match bop {
            BinaryOp::Add => Const(Constant::Int(get_int(e1) + get_int(e2))),
            BinaryOp::Sub => Const(Constant::Int(get_int(e1) - get_int(e2))),
            BinaryOp::Mul => Const(Constant::Int(get_int(e1) * get_int(e2))),
            BinaryOp::LessThan => Const(Constant::Bool(get_int(e1) < get_int(e2))),
            BinaryOp::And => Const(Constant::Bool(get_bool(e1) && get_bool(e2))),
            BinaryOp::Or => Const(Constant::Bool(get_bool(e1) || get_bool(e2))),
            BinaryOp::Write => panic!("Write is not a binary operation"),
        }
    }

    fn interpret_uop(&mut self, uop: &UnaryOp, e: &RcExpr, arg: &Option<Value>) -> Value {
        match uop {
            UnaryOp::Not => {
                let Const(Constant::Bool(b)) = self.interpret(e, arg);
                Const(Constant::Bool(!b))
            }
            UnaryOp::Print => {
                let val = self.interpret(e, arg);
                let v_str = format!("{}", val);
                self.log.push(v_str.clone());
                val
            }
        }
    }

    // TODO: refactor to return a Result<Value, RuntimeError>
    // struct RuntimeError { BadRead(Value) }
    // assumes e typechecks and that memory is written before read
    pub fn interpret(&mut self, expr: &RcExpr, arg: &Option<Value>) -> Value {
        match expr.as_ref() {
            Expr::Const(c) => Const(c.clone()),
            Expr::Bop(bop, e1, e2) => self.interpret_bop(bop, e1, e2, arg),
            Expr::Uop(uop, e) => self.interpret_uop(uop, e, arg),
            Expr::Assume(_assumption, e) => self.interpret(e, arg),
            Expr::Get(e_tuple, i) => {
                let Tuple(vals) = self.interpret(e_tuple, arg) else {
                    panic!("get")
                };
                vals[*i].clone()
            }
            Expr::Read(e_addr, ty) => {
                let Const(Constant::Int(addr)) = self.interpret(e_addr, arg);
                self.mem[&(addr as usize)].clone()
            }
            Expr::Unit() => Tuple(vec![]),
            Expr::Push(_order, e1, e2) => {
                // Always execute sequentially
                // We could also test other orders for parallel tuples
                let v1 = self.interpret(e1, arg);
                let Tuple(mut v2) = self.interpret(e2, arg);
                v2.push(v1);
                Tuple(v2)
            }
            Expr::Switch(pred, branches) => {
                let Const(Constant::Int(index)) = self.interpret(pred, arg);
                if index < 0 || index as usize >= branches.len() {
                    // TODO refactor to return a Result
                    panic!("switch index out of bounds")
                }
                self.interpret(&branches[index as usize], arg)
            }
            Expr::If(pred, then, els) => {
                let Const(Constant::Bool(pred_evaluated)) = self.interpret(pred, arg);
                if pred_evaluated {
                    self.interpret(then, arg)
                } else {
                    self.interpret(els, arg)
                }
            }
            Expr::DoWhile(input, pred_output) => {
                let Tuple(mut vals) = self.interpret(input, arg);
                let mut pred = Const(Constant::Bool(true));
                while pred == Const(Constant::Bool(true)) {
                    let Tuple(pred_output_val) =
                        self.interpret(pred_output, &Some(Tuple(vals.clone())));
                    assert!(pred_output_val.len() == 1 + vals.len());
                    pred = pred_output_val[0].clone();
                    vals = pred_output_val[1..].to_vec();
                }
                Tuple(vals)
            }
            Expr::Let(input, output) => {
                let vals = self.interpret(input, arg);
                self.interpret(output, &Some(vals.clone()))
            }
            Expr::Arg(_) => {
                let Some(v) = arg else { panic!("arg") };
                v.clone()
            }
            Expr::Function(_, _, _, _) | Expr::Call(_, _) => todo!("interpret functions and calls"),
        }
    }
}

#[test]
fn test_interpreter() {
    use crate::ast::*;
    // numbers 1-10
    let expr = dowhile(
        int(1),
        parallel!(
            less_than(iarg(), int(10)),
            first(parallel!(add(iarg(), int(1)), tprint(iarg())))
        ),
    );
    let res = interpret(&expr, &None);
    assert_eq!(res.value, Const(Constant::Int(11)));
    assert_eq!(
        res.log,
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .into_iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
    );
}
/*
#[test]
fn test_interpreter_fib_using_memory() {
    let nth = 10;
    let fib_nth = 55;
    let e = Expr::All(
        Id(-1),
        Order::Sequential,
        vec![
            Expr::Write(Box::new(Expr::Num(0)), Box::new(Expr::Num(0))),
            Expr::Write(Box::new(Expr::Num(1)), Box::new(Expr::Num(1))),
            Expr::Loop(
                Id(0),
                Box::new(Expr::Num(2)),
                Box::new(Expr::All(
                    Id(0),
                    Order::Parallel,
                    vec![
                        // pred: i < nth
                        Expr::LessThan(Box::new(Expr::Arg(Id(0))), Box::new(Expr::Num(nth))),
                        // output
                        Expr::Get(
                            Box::new(Expr::All(
                                Id(0),
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
        Tuple(vec![
            Tuple(vec![]),
            Tuple(vec![]),
            Num(nth + 1),
            Num(fib_nth)
        ])
    );
    assert_eq!(vm.mem[&(nth as usize)], Num(fib_nth));
    assert!(!vm.mem.contains_key(&(nth as usize + 1)));
}

*/
