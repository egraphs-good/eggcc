use std::{collections::HashMap, fmt::Display};

use crate::schema::{BinaryOp, Constant, Expr, Order, RcExpr, UnaryOp};

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
        let get_int = |e: &RcExpr, vm: &mut Self| match vm.interpret(e, arg) {
            Const(Constant::Int(n)) => n,
            other => panic!(
                "Expected integer in binary operation {:?}. Got {:?} from expr {:?}",
                bop, other, e
            ),
        };
        let get_bool = |e: &RcExpr, vm: &mut Self| match vm.interpret(e, arg) {
            Const(Constant::Bool(b)) => b,
            _ => panic!(
                "Expected boolean in binary operation {:?}. Got {:?}",
                bop, e
            ),
        };
        match bop {
            BinaryOp::Add => Const(Constant::Int(get_int(e1, self) + get_int(e2, self))),
            BinaryOp::Sub => Const(Constant::Int(get_int(e1, self) - get_int(e2, self))),
            BinaryOp::Mul => Const(Constant::Int(get_int(e1, self) * get_int(e2, self))),
            BinaryOp::LessThan => Const(Constant::Bool(get_int(e1, self) < get_int(e2, self))),
            BinaryOp::And => Const(Constant::Bool(get_bool(e1, self) && get_bool(e2, self))),
            BinaryOp::Or => Const(Constant::Bool(get_bool(e1, self) || get_bool(e2, self))),
            BinaryOp::Write => {
                let addr = get_int(e1, self) as usize;
                let val = self.interpret(e2, arg).clone();
                self.mem.insert(addr, val);
                Tuple(vec![])
            }
        }
    }

    fn interpret_uop(&mut self, uop: &UnaryOp, e: &RcExpr, arg: &Option<Value>) -> Value {
        match uop {
            UnaryOp::Not => {
                let Const(Constant::Bool(b)) = self.interpret(e, arg) else {
                    panic!("expected boolean in not")
                };
                Const(Constant::Bool(!b))
            }
            UnaryOp::Print => {
                let val = self.interpret(e, arg);
                let v_str = format!("{}", val);
                self.log.push(v_str.clone());
                Tuple(vec![])
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
                let Const(Constant::Int(addr)) = self.interpret(e_addr, arg) else {
                    panic!("expected integer address in read")
                };

                // TODO cast to correct type?
                if let Some(res) = self.mem.get(&(addr as usize)) {
                    res.clone()
                } else {
                    panic!("No value bound at memory address {:?}", addr)
                }
            }
            Expr::Empty => Tuple(vec![]),
            Expr::Single(e) => Tuple(vec![self.interpret(e, arg)]),
            Expr::Extend(order, e1, e2) => {
                let (v1_tuple, v2_tuple) = match order {
                    // Always parallel execute sequentially
                    // We could also test other orders for parallel tuples
                    Order::Sequential | Order::Parallel => {
                        (self.interpret(e1, arg), self.interpret(e2, arg))
                    }
                };
                let Tuple(v1) = v1_tuple else {
                    panic!("expected tuple in push's first argument")
                };
                let Tuple(mut v2) = v2_tuple else {
                    panic!("expected tuple in push's second argument")
                };
                v2.extend(v1);
                Tuple(v2)
            }
            Expr::Switch(pred, branches) => {
                let Const(Constant::Int(index)) = self.interpret(pred, arg) else {
                    panic!("expected integer in switch")
                };
                if index < 0 || index as usize >= branches.len() {
                    // TODO refactor to return a Result
                    panic!("switch index out of bounds")
                }
                self.interpret(&branches[index as usize], arg)
            }
            Expr::If(pred, then, els) => {
                let Const(Constant::Bool(pred_evaluated)) = self.interpret(pred, arg) else {
                    panic!("expected boolean in if")
                };
                if pred_evaluated {
                    self.interpret(then, arg)
                } else {
                    self.interpret(els, arg)
                }
            }
            Expr::DoWhile(input, pred_output) => {
                let Tuple(mut vals) = self.interpret(input, arg) else {
                    panic!("expected tuple for input in do-while")
                };
                let mut pred = Const(Constant::Bool(true));
                while pred == Const(Constant::Bool(true)) {
                    let Tuple(pred_output_val) =
                        self.interpret(pred_output, &Some(Tuple(vals.clone())))
                    else {
                        panic!("expected tuple for pred_output in do-while")
                    };
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
            Expr::Arg => {
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
    let expr = get(
        dowhile(
            parallel!(int(1)),
            parallel!(
                less_than(getat(0), int(10)),
                first(parallel!(add(getat(0), int(1)), tprint(getat(0))))
            ),
        ),
        0,
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

#[test]
fn test_interpreter_fib_using_memory() {
    use crate::ast::*;
    use crate::schema::Type::*;
    let nth = 10;
    let fib_nth = 55;
    let expr = tlet(
        sequence!(
            twrite(int(0), int(0)), // address 0, value 0
            twrite(int(1), int(1)), // address 1, value 1
        ),
        tlet(
            dowhile(
                parallel!(int(2)),
                parallel!(
                    less_than(getat(0), int(nth)),
                    get(
                        parallel!(
                            add(getat(0), int(1)),
                            twrite(
                                getat(0),
                                add(
                                    read(sub(getat(0), int(1)), IntT),
                                    read(sub(getat(0), int(2)), IntT)
                                )
                            )
                        ),
                        0
                    )
                ),
            ),
            push_par(read(int(nth), IntT), arg()),
        ),
    );

    let res = interpret(&expr, &None);
    assert_eq!(
        res.value,
        Tuple(vec![
            Const(Constant::Int(nth + 1)),
            Const(Constant::Int(fib_nth))
        ])
    );
    assert_eq!(res.mem[&(nth as usize)], Const(Constant::Int(fib_nth)));
    assert!(!res.mem.contains_key(&(nth as usize + 1)));
}
