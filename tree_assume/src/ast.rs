use crate::schema::{
    Assumption, BaseType, BinaryOp, Expr, Order, RcExpr, TreeProgram, Type, UnaryOp,
};

pub fn intt() -> Type {
    Type::Base(BaseType::IntT)
}

pub fn boolt() -> Type {
    Type::Base(BaseType::BoolT)
}

pub fn tuplet_vec(types: Vec<BaseType>) -> Type {
    Type::TupleT(types)
}

#[macro_export]
macro_rules! tuplet {
    ($($x:expr),* $(,)?) => ($crate::ast::tuplet_vec(vec![$(std::rc::Rc::new($x)),*]))
}
pub use tuplet;

pub fn add(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Add, l, r))
}

pub fn sub(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Sub, l, r))
}

pub fn mul(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Mul, l, r))
}

pub fn less_than(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::LessThan, l, r))
}

pub fn and(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::And, l, r))
}

pub fn or(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Or, l, r))
}

pub fn not(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Uop(UnaryOp::Not, e))
}

pub fn twrite(addr: RcExpr, val: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Write, addr, val))
}

pub fn tprint(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Uop(UnaryOp::Print, e))
}

pub fn get(e: RcExpr, i: usize) -> RcExpr {
    RcExpr::new(Expr::Get(e, i))
}

pub fn first(e: RcExpr) -> RcExpr {
    get(e, 0)
}

pub fn second(e: RcExpr) -> RcExpr {
    get(e, 1)
}
pub fn write(ptr: RcExpr, val: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Write, ptr, val))
}

pub fn load(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Uop(UnaryOp::Load, e))
}

pub fn ptradd(ptr: RcExpr, i: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::PtrAdd, ptr, i))
}

pub fn alloc(e: RcExpr, ty: Type) -> RcExpr {
    RcExpr::new(Expr::Alloc(e, ty))
}

pub fn call(s: &str, e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Call(s.to_string(), e))
}
/// a macro that wraps the children in
/// a vec for program
#[macro_export]
macro_rules! program {
    ($main:expr, $($x:expr),* $(,)?) => ($crate::ast::program_vec($main, vec![$($x),*]))
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
    RcExpr::new(Expr::Switch(cond, cases))
}

pub fn empty() -> RcExpr {
    RcExpr::new(Expr::Empty)
}

pub fn single(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Single(e))
}

pub fn cons_par(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Parallel, r, single(l)))
}

pub fn push_par(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Parallel, single(l), r))
}

pub fn push_seq(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Sequential, single(l), r))
}

pub fn extend_par(tuple: RcExpr, onto: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Parallel, tuple, onto))
}

pub fn extend_seq(tuple: RcExpr, onto: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Extend(Order::Sequential, tuple, onto))
}

#[macro_export]
macro_rules! parallel {
    ($($x:expr),* $(,)?) => ($crate::ast::parallel_vec(vec![$($x),*]))
}
pub use parallel;

pub fn parallel_vec(es: Vec<RcExpr>) -> RcExpr {
    let mut res = empty();
    for expr in es {
        res = push_par(expr, res);
    }
    res
}

pub fn tlet(lhs: RcExpr, rhs: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Let(lhs, rhs))
}

pub fn arg() -> RcExpr {
    RcExpr::new(Expr::Arg)
}

pub fn getat(index: usize) -> RcExpr {
    get(arg(), index)
}

pub fn tif(cond: RcExpr, then_case: RcExpr, else_case: RcExpr) -> RcExpr {
    RcExpr::new(Expr::If(cond, then_case, else_case))
}

pub fn dowhile(inputs: RcExpr, pred_and_body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::DoWhile(inputs, pred_and_body))
}

pub fn function(name: &str, arg_ty: Type, ret_ty: Type, body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Function(name.to_string(), arg_ty, ret_ty, body))
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
