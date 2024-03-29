use std::{collections::HashMap, fmt::Display};

use crate::schema::{BinaryOp, Constant, Expr, Order, RcExpr, TreeProgram, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Const(Constant),
    Ptr(Pointer),
    Tuple(Vec<Value>),
}

impl Value {
    pub fn bril_print(&self) -> String {
        match self {
            Const(Constant::Int(n)) => format!("{}", n),
            Const(Constant::Bool(b)) => format!("{}", b),
            Ptr(Pointer { .. }) => todo!("How does bril print pointers?"),
            Tuple(_vs) => {
                panic!("Tried to print tuple as Bril value. There are no tuples in Bril.");
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
        }
    }
}

use Value::{Const, Ptr, Tuple};

/// Keeps track of state while running
/// the given TreeProgram.
pub(crate) struct VirtualMachine<'a> {
    program: &'a TreeProgram,
    /// Next address for allocating memory.
    next_addr: usize,
    /// All of memory
    mem: HashMap<usize, Value>,
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
pub fn interpret_tree_prog(prog: &TreeProgram, arg: &Value) -> (Value, Vec<String>) {
    let mut vm = VirtualMachine {
        program: prog,
        next_addr: 0,
        mem: HashMap::new(),
        log: vec![],
    };
    let ret_val = vm.interpret(&prog.entry.func_name().unwrap(), arg);
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
        mem: HashMap::new(),
        log: vec![],
    };
    let value = vm.interpret_expr(expr, func_arg);
    BrilState {
        mem: vm.mem,
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

    fn interpret_bop(&mut self, bop: &BinaryOp, e1: &RcExpr, e2: &RcExpr, arg: &Value) -> Value {
        let get_int = |e: &RcExpr, vm: &mut Self| vm.interp_int_expr(e, arg);
        let get_bool = |e: &RcExpr, vm: &mut Self| vm.interp_bool_expr(e, arg);
        let get_pointer = |e: &RcExpr, vm: &mut Self| vm.interp_pointer_expr(e, arg);
        match bop {
            BinaryOp::Add => Const(Constant::Int(get_int(e1, self) + get_int(e2, self))),
            BinaryOp::Sub => Const(Constant::Int(get_int(e1, self) - get_int(e2, self))),
            BinaryOp::Mul => Const(Constant::Int(get_int(e1, self) * get_int(e2, self))),
            BinaryOp::Div => Const(Constant::Int(get_int(e1, self) / get_int(e2, self))),
            BinaryOp::Eq => Const(Constant::Bool(get_int(e1, self) == get_int(e2, self))),
            BinaryOp::LessThan => Const(Constant::Bool(get_int(e1, self) < get_int(e2, self))),
            BinaryOp::GreaterThan => Const(Constant::Bool(get_int(e1, self) > get_int(e2, self))),
            BinaryOp::LessEq => Const(Constant::Bool(get_int(e1, self) <= get_int(e2, self))),
            BinaryOp::GreaterEq => Const(Constant::Bool(get_int(e1, self) >= get_int(e2, self))),
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
            BinaryOp::Write => {
                let pointer = get_pointer(e1, self);
                let val = self.interpret_expr(e2, arg).clone();
                self.mem.insert(pointer.addr(), val);
                Tuple(vec![])
            }
            BinaryOp::PtrAdd => {
                let Pointer {
                    start_addr: addr,
                    size,
                    offset,
                } = get_pointer(e1, self);
                Ptr(Pointer::new(addr, size, offset + get_int(e2, self)))
            }
        }
    }

    fn interpret_uop(&mut self, uop: &UnaryOp, e: &RcExpr, arg: &Value) -> Value {
        match uop {
            UnaryOp::Not => Const(Constant::Bool(!self.interp_bool_expr(e, arg))),
            UnaryOp::Print => {
                let val = self.interpret_expr(e, arg);
                let v_str = val.bril_print().to_string();
                self.log.push(v_str.clone());
                Tuple(vec![])
            }
            UnaryOp::Load => {
                let ptr = self.interp_pointer_expr(e, arg);
                if let Some(val) = self.mem.get(&ptr.addr()) {
                    val.clone()
                } else {
                    panic!("No value bound at memory address {:?}", ptr.addr())
                }
            }
            UnaryOp::Free => {
                let ptr = self.interp_pointer_expr(e, arg);
                self.mem.remove(&ptr.addr());
                Tuple(vec![])
            }
        }
    }

    // TODO: refactor to return a Result<Value, RuntimeError>
    // struct RuntimeError { BadRead(Value) }
    // in_contexts e typechecks
    pub fn interpret(&mut self, func_name: &str, arg: &Value) -> Value {
        let func = self.program.get_function(func_name).unwrap();
        self.interpret_expr(func.func_body().unwrap(), arg)
    }

    pub fn interpret_expr(&mut self, expr: &RcExpr, arg: &Value) -> Value {
        match expr.as_ref() {
            Expr::Const(c, _ty) => Const(c.clone()),
            Expr::Bop(bop, e1, e2) => self.interpret_bop(bop, e1, e2, arg),
            Expr::Uop(uop, e) => self.interpret_uop(uop, e, arg),
            Expr::InContext(_assumption, e) => self.interpret_expr(e, arg),
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
            Expr::Alloc(e_size, _ty) => {
                let size = self.interp_int_expr(e_size, arg);
                let addr = self.next_addr;
                self.next_addr += usize::try_from(size).unwrap();
                Ptr(Pointer::new(addr, size as usize, 0))
            }
            Expr::Empty(_ty) => Tuple(vec![]),
            Expr::Single(e) => Tuple(vec![self.interpret_expr(e, arg)]),
            Expr::Concat(order, e1, e2) => {
                let (v1_tuple, v2_tuple) = match order {
                    // Always execute sequentially
                    // We could also test other orders for parallel tuples
                    Order::Sequential | Order::Parallel => {
                        (self.interpret_expr(e1, arg), self.interpret_expr(e2, arg))
                    }
                    Order::Reversed => {
                        let v2 = self.interpret_expr(e2, arg);
                        let v1 = self.interpret_expr(e1, arg);
                        (v1, v2)
                    }
                };
                let Tuple(mut v1) = v1_tuple else {
                    panic!("expected tuple in extend's first argument in: {:?}", e1)
                };
                let Tuple(v2) = v2_tuple else {
                    panic!("expected tuple in extend's second argument in {:?}", e2)
                };
                v1.extend(v2);
                Tuple(v1)
            }
            Expr::Switch(pred, branches) => {
                let index = self.interp_int_expr(pred, arg);
                if index < 0 || index as usize >= branches.len() {
                    // TODO refactor to return a Result
                    panic!("switch index out of bounds")
                }
                self.interpret_expr(&branches[index as usize], arg)
            }
            Expr::If(pred, then, els) => {
                let pred_evaluated = self.interp_bool_expr(pred, arg);
                if pred_evaluated {
                    self.interpret_expr(then, arg)
                } else {
                    self.interpret_expr(els, arg)
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
                        self.interpret_expr(pred_output, &Tuple(vals.clone()))
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
            Expr::Let(input, output) => {
                let input_val = self.interpret_expr(input, arg);
                // Evaluate the output with the result
                // bound to `(Arg)`
                self.interpret_expr(output, &input_val)
            }
            Expr::Arg(_ty) => arg.clone(),
            Expr::Function(..) => panic!("Function should not be interpreted as an expression"),
            Expr::Call(func_name, e) => {
                let e_val = self.interpret_expr(e, arg);
                self.interpret(func_name, &e_val)
            }
        }
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
        function(
            "func2",
            base(intt()),
            base(intt()),
            tlet(arg(), add(arg(), int(1)))
        ),
    );
    let res = interpret_tree_prog(&expr, &Const(Constant::Int(5))).0;
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
            add(
                call("fib", sub(arg(), int(1))),
                call("fib", sub(arg(), int(2)))
            )
        )
    ),);
    let res = interpret_tree_prog(&expr, &Const(Constant::Int(10))).0;
    assert_eq!(res, Const(Constant::Int(55)));
}

#[test]
fn test_interpreter() {
    use crate::ast::*;
    // print numbers 1-10
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
    let res = interpret_expr(&expr, &val_empty());
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
fn test_interpreter_fib_using_memory() {
    use crate::ast::*;
    let nth = 10;
    let fib_nth = 55;
    let expr = tlet(
        alloc(int(nth + 2), base(intt())),
        tlet(
            concat_seq(
                twrite(arg(), int(0)), // address 0, value 0
                concat_seq(
                    twrite(ptradd(arg(), int(1)), int(1)), // address 1, value 1
                    single(arg()),
                ),
            ), // [ptr(0)]
            tlet(
                dowhile(
                    parallel!(ptradd(getat(0), int(2)), int(2)), // [ptr(2), 2]
                    cons_par(
                        less_than(getat(1), int(nth)),
                        concat_par(
                            twrite(
                                getat(0),
                                add(
                                    load(ptradd(getat(0), int(-1))),
                                    load(ptradd(getat(0), int(-2))),
                                ),
                            ),
                            parallel!(ptradd(getat(0), int(1)), add(getat(1), int(1))),
                        ),
                    ),
                ),
                parallel!(load(ptradd(getat(0), int(-1))), getat(1)),
            ),
        ),
    );

    let res = interpret_expr(&expr, &val_empty());
    assert_eq!(
        res.value,
        Tuple(vec![
            Const(Constant::Int(fib_nth)),
            Const(Constant::Int(11))
        ])
    );
    assert_eq!(res.mem[&(nth as usize)], Const(Constant::Int(fib_nth)));
    assert!(!res.mem.contains_key(&(nth as usize + 1)));
}
