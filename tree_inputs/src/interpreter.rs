use std::collections::HashMap;

use crate::schema::{BinaryOp, Constant, Expr, RcExpr, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Const(Constant),
    Tuple(Vec<Value>),
}

pub(crate) struct VirtualMachine {
    mem: HashMap<usize, Value>,
    log: Vec<String>,
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
            Value::Const(Constant::Int(n)) => n,
            _ => panic!(
                "Expected integer in binary operation {:?}. Got {:?}",
                bop, e
            ),
        };
        let mut get_bool = |e: &RcExpr| match self.interpret(e, arg) {
            Value::Const(Constant::Bool(b)) => b,
            _ => panic!(
                "Expected boolean in binary operation {:?}. Got {:?}",
                bop, e
            ),
        };
        match bop {
            BinaryOp::Add => Value::Const(Constant::Int(get_int(e1) + get_int(e2))),
            BinaryOp::Sub => Value::Const(Constant::Int(get_int(e1) - get_int(e2))),
            BinaryOp::Mul => Value::Const(Constant::Int(get_int(e1) * get_int(e2))),
            BinaryOp::LessThan => Value::Const(Constant::Bool(get_int(e1) < get_int(e2))),
            BinaryOp::And => Value::Const(Constant::Bool(get_bool(e1) && get_bool(e2))),
            BinaryOp::Or => Value::Const(Constant::Bool(get_bool(e1) || get_bool(e2))),
            BinaryOp::Write => panic!("Write is not a binary operation"),
        }
    }

    fn interpret_uop(&mut self, uop: &UnaryOp, e: &RcExpr, arg: &Option<Value>) -> Value {
        match uop {
            UnaryOp::Not => {
                let Value::Const(Constant::Bool(b)) = self.interpret(e, arg);
                Value::Const(Constant::Bool(!b))
            }
            UnaryOp::Print => {
                todo!("print")
            }
        }
    }

    // TODO: refactor to return a Result<Value, RuntimeError>
    // struct RuntimeError { BadRead(Value) }
    // assumes e typechecks and that memory is written before read
    pub fn interpret(&mut self, expr: &RcExpr, arg: &Option<Value>) -> Value {
        match expr.as_ref() {
            Expr::Const(c) => Value::Const(c.clone()),
            Expr::Bop(bop, e1, e2) => self.interpret_bop(bop, e1, e2, arg),
            Expr::Get(e_tuple, i) => {
                let Value::Tuple(vals) = self.interpret(e_tuple, arg) else {
                    panic!("get")
                };
                vals[*i].clone()
            }
            Expr::Read(e_addr, ty) => {
                let Value::Const(Constant::Int(addr)) = self.interpret(e_addr, arg);
                self.mem[&(addr as usize)].clone()
            }
            Expr::All(_order, exprs) => {
                // this always executes sequentially
                // in the future we should test other orders for parallel tuples
                let vals = exprs
                    .iter()
                    .map(|expr| self.interpret(expr, arg))
                    .collect::<Vec<_>>();
                Value::Tuple(vals)
            }
            Expr::Switch(pred, branches) => {
                let Value::Const(Constant::Int(index)) = self.interpret(pred, arg);
                if index < 0 || index as usize >= branches.len() {
                    // TODO refactor to return a Result
                    panic!("switch index out of bounds")
                }
                self.interpret(&branches[index as usize], arg)
            }
            Expr::DoWhile(input, pred_output) => {
                let mut vals = self.interpret(input, arg);
                let mut pred = Value::Const(Constant::Bool(true));
                while pred == Value::Const(Constant::Bool(true)) {
                    let Value::Tuple(pred_output_val) =
                        self.interpret(pred_output, &Some(vals.clone()));
                    assert!(pred_output_val.len() == 1 + vals.len());
                    pred = pred_output_val[0].clone();
                    vals = pred_output_val[1..].to_vec();
                }
                vals
            }
            Expr::Let(_, input, output) => {
                let vals = interpret(input, arg, self);
                interpret(output, &Some(vals.clone()), self)
            }
            Expr::Arg(_) => {
                let Some(v) = arg else { panic!("arg") };
                v.clone()
            }
            Expr::Function(_, _) | Expr::Call(_, _) => todo!("interpret functions and calls"),
        }
    }
}

#[test]
fn test_interpreter() {
    // numbers 1-10
    let e = Expr::Loop(
        Id(0),
        Box::new(Expr::Num(1)),
        Box::new(Expr::All(
            Id(0),
            Order::Parallel,
            vec![
                // pred: i < 10
                Expr::LessThan(Box::new(Expr::Arg(Id(0))), Box::new(Expr::Num(10))),
                // output
                Expr::Get(
                    Box::new(Expr::All(
                        Id(0),
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
            if let egglog::ast::Expr::Call((), f, xs) = e {
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
            if let egglog::ast::Expr::Call((), f, xs) = e {
                if let ("Id", [egglog::ast::Expr::Lit((), egglog::ast::Literal::Int(int))]) =
                    (f.as_str(), xs.as_slice())
                {
                    return Ok(Id(*int));
                }
            }
            Err(ExprParseError::InvalidId)
        }
        fn egglog_expr_to_expr(e: &egglog::ast::Expr) -> Result<Expr, ExprParseError> {
            match e {
                egglog::ast::Expr::Lit((), _) => Err(ExprParseError::UnwrappedLiteral),
                egglog::ast::Expr::Var((), s) => {
                    Err(ExprParseError::UngroundedTerm(s.as_str().to_owned()))
                }
                egglog::ast::Expr::Call((), f, xs) => match (f.as_str(), xs.as_slice()) {
                    ("Num", [_id, egglog::ast::Expr::Lit((), egglog::ast::Literal::Int(i))]) => {
                        Ok(Expr::Num(*i))
                    }
                    (
                        "Boolean",
                        [_id, egglog::ast::Expr::Lit((), egglog::ast::Literal::Bool(b))],
                    ) => Ok(Expr::Boolean(*b)),
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
                    ("Get", [x, egglog::ast::Expr::Lit((), egglog::ast::Literal::Int(i))]) => {
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
                    ("All", [id, egglog::ast::Expr::Call((), order, empty), xs]) => {
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
(All (Id 1) (Sequential)
    (Cons (LessThan (Num (Id 1) 2) (Num (Id 1) 3))
        (Cons (Switch (Boolean (Id 1) true) (Cons (Num (Id 1) 4) (Cons (Num (Id 1) 5) (Nil))))
            (Nil)))))
";
    let build = s.parse::<Expr>().unwrap();
    let check = Expr::Loop(
        Id(1),
        Box::new(Expr::Num(1)),
        Box::new(Expr::All(
            Id(1),
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
