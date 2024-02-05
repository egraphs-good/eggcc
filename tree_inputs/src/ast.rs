use crate::schema::{Assumption, BinaryOp, Expr, Order, RcExpr, TreeProgram, Type, UnaryOp};

pub fn add(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Add, l.clone(), r.clone()))
}

pub fn sub(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Sub, l.clone(), r.clone()))
}

pub fn mul(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Mul, l.clone(), r.clone()))
}

pub fn less_than(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::LessThan, l.clone(), r.clone()))
}

pub fn and(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::And, l.clone(), r.clone()))
}

pub fn or(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Or, l.clone(), r.clone()))
}

pub fn not(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Uop(UnaryOp::Not, e.clone()))
}

pub fn print(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Uop(UnaryOp::Print, e.clone()))
}

pub fn get(e: RcExpr, i: i64) -> RcExpr {
    RcExpr::new(Expr::Get(e.clone(), i))
}

pub fn read(e: RcExpr, ty: Type) -> RcExpr {
    RcExpr::new(Expr::Read(e.clone(), ty))
}

pub fn call(s: &str, e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Call(s.to_string(), e.clone()))
}
/// a macro that wraps the children in
/// a vec for program
#[macro_export]
macro_rules! program {
    ($main:expr, $($x:expr),* $(,)?) => ($crate::ast::program_vec(vec![$main, $($x),*]))
}
pub use program;

pub fn program_vec(entry: RcExpr, functions: Vec<RcExpr>) -> TreeProgram {
    TreeProgram { entry, functions }
}

#[macro_export]
macro_rules! switch {
    ($arg:expr; $($x:expr),* $(,)?) => ($crate::ast::switch_vec($arg, vec![$($x),*]))
}
pub use switch;

pub fn switch_vec(cond: RcExpr, cases: Vec<RcExpr>) -> RcExpr {
    RcExpr::new(Expr::Switch(cond.clone(), cases))
}

pub fn unit() -> RcExpr {
    RcExpr::new(Expr::Empty)
}

pub fn single(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Single(e.clone()))
}

pub fn push_parallel(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Parallel, single(l), r))
}

pub fn push_sequential(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Sequential, single(l), r))
}

pub fn extend_parallel(tuple: RcExpr, onto: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Parallel, tuple, onto))
}

pub fn extend_sequential(tuple: RcExpr, onto: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Sequential, tuple, onto))
}

#[macro_export]
macro_rules! parallel {
    ($($x:expr),* $(,)?) => ($crate::ast::parallel_vec(vec![$($x),*]))
}
pub use parallel;

pub fn parallel_vec(es: Vec<RcExpr>) -> RcExpr {
    let mut res = unit();
    for expr in es {
        res = push_parallel(res, expr);
    }
    res
}

#[macro_export]
macro_rules! sequence {
    ($($x:expr),* $(,)?) => ($crate::ast::sequence_vec(vec![$($x),*]))
}
pub use sequence;

pub fn sequence_vec(es: Vec<RcExpr>) -> RcExpr {
    let mut res = unit();
    for expr in es {
        res = push_sequential(res, expr);
    }
    res
}

pub fn tif(cond: RcExpr, then_case: RcExpr, else_case: RcExpr) -> RcExpr {
    RcExpr::new(Expr::If(cond.clone(), then_case.clone(), else_case.clone()))
}

pub fn dowhile(inputs: RcExpr, pred_and_body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::DoWhile(inputs.clone(), pred_and_body.clone()))
}

pub fn function(name: &str, arg_ty: Type, ret_ty: Type, body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Function(
        name.to_string(),
        arg_ty,
        ret_ty,
        body.clone(),
    ))
}

pub fn ttrue() -> RcExpr {
    RcExpr::new(Expr::Const(crate::schema::Constant::Bool(true)))
}

pub fn tfalse() -> RcExpr {
    RcExpr::new(Expr::Const(crate::schema::Constant::Bool(false)))
}

pub fn int(i: i64) -> RcExpr {
    RcExpr::new(Expr::Const(crate::schema::Constant::Int(i)))
}

pub fn inlet(e: RcExpr) -> Assumption {
    Assumption::InLet(e)
}

pub fn inloop(e1: RcExpr, e2: RcExpr) -> Assumption {
    Assumption::InLoop(e1, e2)
}

pub fn assume(assumption: Assumption, body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Assume(assumption, body))
}
