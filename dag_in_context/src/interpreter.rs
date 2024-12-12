//! Interpret DAG programs. Each expression is evaluated once, so side-effects only happen
//! once. All the dependencies of an expression are evaluted first.
//! The interpreter relies on the invariant that common subexpressions are
//! shared as the same Rc pointer. Otherwise, effects may be executed multiple times.
//! The invariant is maintained by translation from RVSDG, type checking, and translation from egglog.

use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Shl, Shr},
    rc::Rc,
};

use crate::{
    schema::{BinaryOp, Constant, Expr, RcExpr, TernaryOp, TreeProgram, UnaryOp},
    tuplev,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pointer {
    // start address of this pointer
    start_addr: usize,
    // how many elements are in the allocated region
    size: usize,
    // offset from the start address
    offset: i64,
}

impl Pointer {
    pub(crate) fn new(addr: usize, size: usize, offset: i64) -> Self {
        Pointer {
            start_addr: addr,
            size,
            offset,
        }
    }

    // gets the address of this pointer, panicking
    // if the pointer is out of bounds
    fn addr(&self) -> usize {
        if self.offset < 0 || self.offset as usize >= self.size {
            panic!("Pointer out of bounds {:?}", self);
        }
        self.start_addr + self.offset as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Const(Constant),
    Ptr(Pointer),
    Tuple(Vec<Value>),
    StateV,
}

impl Value {
    pub fn bril_print(&self) -> String {
        match self {
            Const(Constant::Int(n)) => format!("{}", n),
            Const(Constant::Bool(b)) => format!("{}", b),
            Const(Constant::Float(f)) => {
                if f.is_infinite() {
                    format!("{}Infinity", if f.is_sign_positive() { "" } else { "-" })
                } else if f.is_nan() {
                    "NaN".to_string()
                } else if f.into_inner() == 0.0 {
                    // handles +0.0 and -0.0 cases
                    "0.00000000000000000".to_string()
                } else {
                    format!("{:.17}", f)
                }
            }
            Ptr(Pointer { .. }) => todo!("How does bril print pointers?"),
            Tuple(_vs) => {
                panic!("Tried to print tuple as Bril value. There are no tuples in Bril.");
            }
            Value::StateV => {
                panic!(
                    "Tried to print state value as Bril value. There are no state values in Bril."
                );
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Const(constant) => write!(f, "{}", constant),
            Ptr(Pointer {
                start_addr: addr,
                size,
                offset,
            }) => {
                write!(f, "Pointer::new({addr}, {size}, {offset})")
            }
            Tuple(vs) => {
                write!(f, "(")?;
                for v in vs {
                    write!(f, "{}, ", v)?;
                }
                write!(f, ")")
            }
            Value::StateV => write!(f, "StateV"),
        }
    }
}

use ordered_float::OrderedFloat;
use Value::{Const, Ptr, Tuple};

/// Keeps track of state while running
/// the given TreeProgram.
pub(crate) struct VirtualMachine<'a> {
    program: &'a TreeProgram,
    /// Next address for allocating memory.
    next_addr: usize,
    /// All of memory
    memory: HashMap<usize, Value>,
    /// Values for already evaluated expressions
    eval_cache: HashMap<*const Expr, Value>,
    /// Print log
    log: Vec<String>,
}

/// Represents the result of running a
/// TreeProgram.
pub struct BrilState {
    /// Resulting memory state
    pub mem: HashMap<usize, Value>,
    /// Pring log
    pub log: Vec<String>,
    /// Return value from the program
    pub value: Value,
}

/// Interprets a program, returning the value
/// returned by the program and the print log.
/// The interpreter relies on the invariant that common subexpressions are
/// shared as the same Rc pointer. Otherwise, effects may be executed multiple times.
pub fn interpret_dag_prog(prog: &TreeProgram, arg: &Value) -> (Value, Vec<String>) {
    let mut vm = VirtualMachine {
        program: prog,
        next_addr: 0,
        memory: HashMap::new(),
        eval_cache: HashMap::new(),
        log: vec![],
    };
    let ret_val = vm.interpret_call(&prog.entry.func_name().unwrap(), arg);
    (ret_val, vm.log)
}

/// Interprets an expression, returning the value
pub fn interpret_expr(expr: &RcExpr, func_arg: &Value) -> BrilState {
    let mut vm = VirtualMachine {
        program: &TreeProgram {
            // expr should be call-free so this doesn't matter
            entry: expr.clone(),
            functions: vec![],
        },
        next_addr: 0,
        eval_cache: HashMap::new(),
        memory: HashMap::new(),
        log: vec![],
    };
    let value = vm.interpret_expr(expr, func_arg);
    BrilState {
        mem: vm.memory,
        log: vm.log,
        value,
    }
}

impl<'a> VirtualMachine<'a> {
    fn interp_int_expr(&mut self, e: &RcExpr, arg: &Value) -> i64 {
        match self.interpret_expr(e, arg) {
            Const(Constant::Int(n)) => n,
            other => panic!("Expected integer. Got {:?} from expr {:?}", other, e),
        }
    }

    fn interp_float_expr(&mut self, e: &RcExpr, arg: &Value) -> OrderedFloat<f64> {
        match self.interpret_expr(e, arg) {
            Const(Constant::Float(n)) => n,
            other => panic!("Expected integer. Got {:?} from expr {:?}", other, e),
        }
    }

    fn interp_bool_expr(&mut self, e: &RcExpr, arg: &Value) -> bool {
        match self.interpret_expr(e, arg) {
            Const(Constant::Bool(b)) => b,
            other => panic!("Expected boolean. Got {:?} from expr {:?}", other, e),
        }
    }

    fn interp_pointer_expr(&mut self, e: &RcExpr, arg: &Value) -> Pointer {
        match self.interpret_expr(e, arg) {
            Ptr(ptr) => ptr,
            other => panic!("Expected pointer. Got {:?} from expr {:?}", other, e),
        }
    }

    fn interpret_top(
        &mut self,
        top: &TernaryOp,
        e1: &RcExpr,
        e2: &RcExpr,
        e3: &RcExpr,
        arg: &Value,
    ) -> Value {
        let get_pointer = |e: &RcExpr, vm: &mut Self| vm.interp_pointer_expr(e, arg);
        match top {
            TernaryOp::Write => {
                let pointer = get_pointer(e1, self);
                let val = self.interpret_expr(e2, arg).clone();
                let state_val = self.interpret_expr(e3, arg);
                assert_eq!(state_val, Value::StateV);
                self.memory.insert(pointer.addr(), val);
                Value::StateV
            }
            TernaryOp::Select => {
                let get_bool = |e: &RcExpr, vm: &mut Self| vm.interp_bool_expr(e, arg);
                if get_bool(e1, self) {
                    self.interpret_expr(e2, arg)
                } else {
                    self.interpret_expr(e3, arg)
                }
            }
        }
    }

    fn interpret_bop(&mut self, bop: &BinaryOp, e1: &RcExpr, e2: &RcExpr, arg: &Value) -> Value {
        let get_int = |e: &RcExpr, vm: &mut Self| vm.interp_int_expr(e, arg);
        let get_float = |e: &RcExpr, vm: &mut Self| vm.interp_float_expr(e, arg);
        let get_bool = |e: &RcExpr, vm: &mut Self| vm.interp_bool_expr(e, arg);
        let get_pointer = |e: &RcExpr, vm: &mut Self| vm.interp_pointer_expr(e, arg);
        match bop {
            BinaryOp::Add => Const(Constant::Int(
                get_int(e1, self).wrapping_add(get_int(e2, self)),
            )),
            BinaryOp::Sub => Const(Constant::Int(
                get_int(e1, self).wrapping_sub(get_int(e2, self)),
            )),
            BinaryOp::Mul => Const(Constant::Int(
                get_int(e1, self).wrapping_mul(get_int(e2, self)),
            )),
            BinaryOp::Div => Const(Constant::Int(
                get_int(e1, self).wrapping_div(get_int(e2, self)),
            )),
            BinaryOp::Smax => {
                let a = get_int(e1, self);
                let b = get_int(e2, self);
                Const(Constant::Int(if a > b { a } else { b }))
            }
            BinaryOp::Smin => {
                let a = get_int(e1, self);
                let b = get_int(e2, self);
                Const(Constant::Int(if a < b { a } else { b }))
            }
            BinaryOp::Shl => Const(Constant::Int(get_int(e1, self).shl(get_int(e2, self)))),
            BinaryOp::Shr => Const(Constant::Int(get_int(e1, self).shr(get_int(e2, self)))),
            BinaryOp::Eq => Const(Constant::Bool(get_int(e1, self) == get_int(e2, self))),
            BinaryOp::LessThan => Const(Constant::Bool(get_int(e1, self) < get_int(e2, self))),
            BinaryOp::GreaterThan => Const(Constant::Bool(get_int(e1, self) > get_int(e2, self))),
            BinaryOp::LessEq => Const(Constant::Bool(get_int(e1, self) <= get_int(e2, self))),
            BinaryOp::GreaterEq => Const(Constant::Bool(get_int(e1, self) >= get_int(e2, self))),
            BinaryOp::Load => {
                let ptr = self.interp_pointer_expr(e1, arg);
                let state_val = self.interpret_expr(e2, arg);
                assert_eq!(state_val, Value::StateV);
                if let Some(val) = self.memory.get(&ptr.addr()) {
                    tuplev!(val.clone(), Value::StateV)
                } else {
                    panic!("No value bound at memory address {:?}", ptr.addr())
                }
            }
            BinaryOp::Free => {
                let ptr = get_pointer(e1, self);
                let state_val = self.interpret_expr(e2, arg);
                assert_eq!(state_val, Value::StateV);
                self.memory.remove(&ptr.addr());
                Value::StateV
            }
            BinaryOp::Print => {
                let val = self.interpret_expr(e1, arg);
                let state_val = self.interpret_expr(e2, arg);
                assert_eq!(state_val, Value::StateV);
                let v_str = val.bril_print().to_string();
                self.log.push(v_str.clone());
                Value::StateV
            }
            BinaryOp::And => {
                let b1 = get_bool(e1, self);
                let b2 = get_bool(e2, self);
                Const(Constant::Bool(b1 && b2))
            }
            BinaryOp::Or => {
                let b1 = get_bool(e1, self);
                let b2 = get_bool(e2, self);
                Const(Constant::Bool(b1 || b2))
            }
            BinaryOp::PtrAdd => {
                let Pointer {
                    start_addr: addr,
                    size,
                    offset,
                } = get_pointer(e1, self);
                Ptr(Pointer::new(addr, size, offset + get_int(e2, self)))
            }
            BinaryOp::FAdd => Const(Constant::Float(get_float(e1, self) + get_float(e2, self))),
            BinaryOp::FSub => Const(Constant::Float(get_float(e1, self) - get_float(e2, self))),
            BinaryOp::FMul => Const(Constant::Float(get_float(e1, self) * (get_float(e2, self)))),
            BinaryOp::FDiv => Const(Constant::Float(get_float(e1, self) / (get_float(e2, self)))),
            BinaryOp::FEq => Const(Constant::Bool(get_float(e1, self) == get_float(e2, self))),
            BinaryOp::FLessThan => Const(Constant::Bool(get_float(e1, self) < get_float(e2, self))),
            BinaryOp::FGreaterThan => {
                Const(Constant::Bool(get_float(e1, self) > get_float(e2, self)))
            }
            BinaryOp::FLessEq => Const(Constant::Bool(get_float(e1, self) <= get_float(e2, self))),
            BinaryOp::FGreaterEq => {
                Const(Constant::Bool(get_float(e1, self) >= get_float(e2, self)))
            }
            BinaryOp::Fmax => {
                let a = get_float(e1, self);
                let b = get_float(e2, self);
                Const(Constant::Float(if a > b { a } else { b }))
            }
            BinaryOp::Fmin => {
                let a = get_float(e1, self);
                let b = get_float(e2, self);
                Const(Constant::Float(if a < b { a } else { b }))
            }
        }
    }

    fn interpret_uop(&mut self, uop: &UnaryOp, e: &RcExpr, arg: &Value) -> Value {
        match uop {
            UnaryOp::Not => Const(Constant::Bool(!self.interp_bool_expr(e, arg))),
        }
    }

    // TODO: refactor to return a Result<Value, RuntimeError>
    // struct RuntimeError { BadRead(Value) }
    // in_contexts e typechecks
    pub fn interpret_call(&mut self, func_name: &str, arg: &Value) -> Value {
        let func = self.program.get_function(func_name).unwrap();
        self.interpret_region(
            func.func_body()
                .expect("Expected function in interpret_call"),
            arg,
        )
    }

    pub fn interpret_region(&mut self, expr: &RcExpr, arg: &Value) -> Value {
        let mut memo_before = HashMap::new();
        // save the memo before, since we are evaluating in a new region
        std::mem::swap(&mut self.eval_cache, &mut memo_before);
        // evaluate expression with brand new memo
        let res = self.interpret_expr(expr, arg);
        // restore the old memo now that we are back in the previous region
        std::mem::swap(&mut self.eval_cache, &mut memo_before);
        res
    }

    pub fn interpret_expr(&mut self, expr: &RcExpr, arg: &Value) -> Value {
        if let Some(val) = self.eval_cache.get(&Rc::as_ptr(expr)) {
            return val.clone();
        }
        let res = match expr.as_ref() {
            Expr::Const(c, _ty, _ctx) => Const(c.clone()),
            Expr::Bop(bop, e1, e2) => self.interpret_bop(bop, e1, e2, arg),
            Expr::Uop(uop, e) => self.interpret_uop(uop, e, arg),
            Expr::Top(top, e1, e2, e3) => self.interpret_top(top, e1, e2, e3, arg),
            Expr::Get(e_tuple, i) => {
                let Tuple(vals) = self.interpret_expr(e_tuple, arg) else {
                    panic!(
                        "get expects a tuple as its first argument. Got {:?}",
                        e_tuple
                    )
                };
                if *i >= vals.len() {
                    panic!(
                        "get index out of bounds. Got index {} for tuple {:?}. Expression:\n{}",
                        i, vals, expr
                    )
                }
                vals[*i].clone()
            }
            // in_context this is type checked, so ignore type
            Expr::Alloc(_id, e_size, state_expr, _ty) => {
                let size = self.interp_int_expr(e_size, arg);
                let state_val = self.interpret_expr(state_expr, arg);
                assert_eq!(state_val, Value::StateV);
                let addr = self.next_addr;
                self.next_addr += usize::try_from(size).unwrap();

                // make a new pointer at the address, with an initial offset of 0
                tuplev!(Ptr(Pointer::new(addr, size as usize, 0)), Value::StateV)
            }
            Expr::Empty(_ty, _ctx) => Tuple(vec![]),
            Expr::Single(e) => Tuple(vec![self.interpret_expr(e, arg)]),
            Expr::Concat(e1, e2) => {
                let Tuple(mut v1) = self.interpret_expr(e1, arg) else {
                    panic!("expected tuple in extend's first argument in: {:?}", e1)
                };
                let Tuple(v2) = self.interpret_expr(e2, arg) else {
                    panic!("expected tuple in extend's second argument in {:?}", e2)
                };
                v1.extend(v2);
                Tuple(v1)
            }
            Expr::Switch(pred, input, branches) => {
                let index = self.interp_int_expr(pred, arg);
                if index < 0 || index as usize >= branches.len() {
                    // TODO refactor to return a Result
                    panic!("switch index out of bounds")
                }
                let input_val = self.interpret_expr(input, arg);
                self.interpret_region(&branches[index as usize], &input_val)
            }
            Expr::If(pred, input, then, els) => {
                let pred_evaluated = self.interp_bool_expr(pred, arg);
                let input_evaluated = self.interpret_expr(input, arg);
                if pred_evaluated {
                    self.interpret_region(then, &input_evaluated)
                } else {
                    self.interpret_region(els, &input_evaluated)
                }
            }
            Expr::DoWhile(input, pred_output) => {
                let Tuple(mut vals) = self.interpret_expr(input, arg) else {
                    panic!("expected tuple for input in do-while")
                };

                // Because it's a do-while, we always execute the body at least once
                let mut pred = Const(Constant::Bool(true));
                while pred == Const(Constant::Bool(true)) {
                    let Tuple(pred_output_val) =
                        self.interpret_region(pred_output, &Tuple(vals.clone()))
                    else {
                        panic!("expected tuple for pred_output in do-while")
                    };
                    assert_eq!(
                        pred_output_val.len(),
                        1 + vals.len(),
                        "expected pred_output to have one more element than input in {:?}",
                        pred_output
                    );
                    pred = pred_output_val[0].clone();
                    vals = pred_output_val[1..].to_vec();
                }
                Tuple(vals)
            }
            Expr::Arg(_ty, _ctx) => arg.clone(),
            Expr::Function(..) => panic!("Function should not be interpreted as an expression"),
            Expr::Call(func_name, e) => {
                let e_val = self.interpret_expr(e, arg);
                self.interpret_call(func_name, &e_val)
            }
            Expr::Symbolic(_, _ty) => panic!("found symbolic"),
        };
        self.eval_cache.insert(Rc::as_ptr(expr), res.clone());
        res
    }
}

#[test]
fn test_interpret_calls() {
    use crate::ast::*;
    let expr = program!(
        function(
            "func1",
            base(intt()),
            base(intt()),
            mul(call("func2", sub(arg(), int(1))), int(2))
        ),
        function("func2", base(intt()), base(intt()), add(arg(), int(1))),
    );
    let res = interpret_dag_prog(&expr, &Const(Constant::Int(5))).0;
    assert_eq!(res, Const(Constant::Int(10)));
}

#[test]
fn test_interpret_recursive() {
    use crate::ast::*;
    let expr = program!(function(
        "fib",
        base(intt()),
        base(intt()),
        tif(
            less_than(arg(), int(2)),
            arg(),
            arg(),
            add(
                call("fib", sub(arg(), int(1))),
                call("fib", sub(arg(), int(2)))
            )
        )
    ),);
    let res = interpret_dag_prog(&expr, &Const(Constant::Int(10))).0;
    assert_eq!(res, Const(Constant::Int(55)));
}

#[test]
fn test_interpreter() {
    use crate::ast::*;
    // print numbers 1-10
    let expr = get(
        dowhile(
            parallel!(int(1), arg()),
            parallel!(
                less_than(getat(0), int(10)),
                add(getat(0), int(1)),
                tprint(getat(0), getat(1))
            ),
        ),
        0,
    );
    let res = interpret_expr(&expr, &statev());
    assert_eq!(res.value, Const(Constant::Int(11)));
    assert_eq!(
        res.log,
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .into_iter()
            .map(|i| format!("{}", i))
            .collect::<Vec<String>>()
    );
}

#[test]
fn test_recursive_interp() {}
