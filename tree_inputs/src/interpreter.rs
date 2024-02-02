use std::rc::Rc;

use crate::schema::{BinaryOp, Constant, Ctx, Expr, Order, Program, RcExpr, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Const(Constant),
    Tuple(Vec<Value>),
}

impl Value {
    fn to_expr(&self) -> RcExpr {
        match self {
            Value::Const(c) => Rc::new(Expr::Const(Ctx::Global, c.clone())),
            Value::Tuple(vs) => {
                let mut es = vec![];
                for v in vs {
                    es.push(v.to_expr());
                }
                Rc::new(Expr::All(Ctx::Global, Order::Parallel, es))
            }
        }
    }
}

// Interpret the program with this argument.
// Assumes a valid program (typechecks and
// follows the region invariants in semantics.md).
pub fn interpret(program: &Program, arg: Value) -> Value {
    if !program.functions.is_empty() {
        unimplemented!("Multiple functions not supported yet")
    }
    interpret_expr(program.entry.clone(), arg)
}

/// Interpret a `Let`'s body with the given
/// argument.
/// First, finds the input to the `Let` and
/// evaluates it.
/// Substitutes the resulting value for the
/// input, and evaluates the body.
fn interpret_let(expr: RcExpr, arg: Value) -> Value {
    let found_input = expr.clone().find_input();
    // first, find the input
    let input = match found_input {
        Some(iexpr) => match iexpr.as_ref().clone() {
            Expr::Arg(_) => panic!("Found argument in let"),
            Expr::Input(in_expr) => in_expr,
            _ => panic!("find_input returned invalid input"),
        },
        None => return interpret_expr(expr, arg),
    };
    // substitute the input with the new value
    let substituted = input
        .clone()
        .substitute(input.clone(), interpret_expr(input.clone(), arg).to_expr());
    interpret_expr(substituted, Value::Tuple(vec![]))
}

/// Interpret an expression with the given arg.
/// The `expr` refers only to `Arg` directly,
/// and to `Input` only under a new `Let`.
fn interpret_expr(expr: RcExpr, arg: Value) -> Value {
    match expr.as_ref() {
        Expr::Const(_, c) => Value::Const(c.clone()),
        Expr::Bop(op, left, right) => {
            let l_res = interpret_expr(left.clone(), arg.clone());
            let r_res = interpret_expr(right.clone(), arg);
            interpret_binary_op(op.clone(), l_res, r_res)
        }
        Expr::Uop(op, e) => {
            let res = interpret_expr(e.clone(), arg);
            interpret_unary_op(op.clone(), res)
        }
        Expr::Get(child, i) => {
            let child_res = interpret_expr(child.clone(), arg);
            match child_res {
                Value::Tuple(vs) => vs[*i as usize].clone(),
                _ => panic!("Get called on non-tuple value"),
            }
        }
        Expr::DoWhile(inputs, pred, body) => {
            let mut current_value = interpret_expr(inputs.clone(), arg);
            let mut continue_loop = true;
            while continue_loop {
                let next_value = interpret_expr(body.clone(), current_value.clone());
                continue_loop = matches!(
                    interpret_expr(pred.clone(), current_value),
                    Value::Const(Constant::Bool(true)),
                );
                current_value = next_value;
            }
            current_value
        }
        Expr::All(_ctx, _order, es) => {
            // TODO when order is parallel, interpreter
            // is free to do these in any order.
            // We should test our programs with different orders.
            let mut res = vec![];
            for e in es {
                res.push(interpret_expr(e.clone(), arg.clone()));
            }
            Value::Tuple(res)
        }
        Expr::Function(_name, _arg_ty, _ret_ty, body) => {
            // interpret the body
            interpret_expr(body.clone(), arg)
        }
        Expr::Switch(cond, cases) => {
            let cond_res = interpret_expr(cond.clone(), arg.clone());
            let Value::Const(Constant::Int(index)) = cond_res else {
                panic!("Switch condition not an integer. Got {:?}", cond_res)
            };
            interpret_expr(cases[index as usize].clone(), arg)
        }
        Expr::If(cond, then_case, else_case) => {
            let cond_res = interpret_expr(cond.clone(), arg.clone());
            let Value::Const(Constant::Bool(b)) = cond_res else {
                panic!("If condition not a boolean. Got {:?}", cond_res)
            };
            if b {
                interpret_expr(then_case.clone(), arg)
            } else {
                interpret_expr(else_case.clone(), arg)
            }
        }
        Expr::Let(expr) => interpret_let(expr.clone(), arg),
        Expr::Call(_name, _arg) => unimplemented!("Call not implemented yet"),
        Expr::Read(_e, _ty) => unimplemented!("Read not implemented yet"),
        Expr::Arg(_ty) => arg,
        Expr::Input(_) => panic!("Input found outside of let"),
    }
}

fn interpret_binary_op(op: BinaryOp, left_arg: Value, right_arg: Value) -> Value {
    use BinaryOp::*;
    use Constant::*;
    use Value::*;
    match (op.clone(), left_arg.clone(), right_arg.clone()) {
        (Add, Const(Int(l)), Const(Int(r))) => Const(Int(l + r)),
        (Sub, Const(Int(l)), Const(Int(r))) => Const(Int(l - r)),
        (Mul, Const(Int(l)), Const(Int(r))) => Const(Int(l * r)),
        (LessThan, Const(Int(l)), Const(Int(r))) => Const(Bool(l < r)),
        (And, Const(Bool(l)), Const(Bool(r))) => Const(Bool(l && r)),
        (Or, Const(Bool(l)), Const(Bool(r))) => Const(Bool(l || r)),
        (Write, _, _) => unimplemented!("Write not implemented yet"),
        _ => panic!(
            "Invalid binary op {:?} for args {:?} and {:?}",
            op, left_arg, right_arg
        ),
    }
}

fn interpret_unary_op(op: UnaryOp, arg: Value) -> Value {
    use Constant::*;
    use UnaryOp::*;
    use Value::*;
    match (op.clone(), arg.clone()) {
        (Not, Const(Bool(b))) => Const(Bool(!b)),
        (Print, Const(_c)) => {
            unimplemented!("Print not implemented yet")
        }
        _ => panic!("Invalid unary op {:?} for arg {:?}", op, arg),
    }
}
