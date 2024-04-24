use crate::{
    interpreter::Value,
    schema::{
        Assumption, BaseType, BinaryOp, Constant, Expr, RcExpr, TernaryOp, TreeProgram, Type,
        UnaryOp,
    },
};

pub fn base(t: BaseType) -> Type {
    Type::Base(t)
}

pub fn intt() -> BaseType {
    BaseType::IntT
}

pub fn boolt() -> BaseType {
    BaseType::BoolT
}

pub fn emptyt() -> Type {
    Type::TupleT(vec![])
}

pub fn statet() -> BaseType {
    BaseType::StateT
}

pub fn tuplet_vec(types: Vec<BaseType>) -> Type {
    Type::TupleT(types)
}

pub fn tuplev_vec(types: Vec<Value>) -> Value {
    Value::Tuple(types)
}

pub fn pointert(t: BaseType) -> BaseType {
    BaseType::PointerT(Box::new(t))
}

pub fn intv(i: i64) -> Value {
    Value::Const(Constant::Int(i))
}

pub fn falsev() -> Value {
    Value::Const(Constant::Bool(false))
}

pub fn truev() -> Value {
    Value::Const(Constant::Bool(true))
}

pub fn statev() -> Value {
    Value::StateV
}

pub fn val_bool(i: bool) -> Value {
    Value::Const(Constant::Bool(i))
}

pub fn val_empty() -> Value {
    Value::Tuple(vec![])
}

pub fn val_vec(vals: Vec<Value>) -> Value {
    Value::Tuple(vals)
}

/// Construct a tuple type from the child types
/// e.g. `tuple!(intt(), boolt())` becomes `Type::TupleT(vec![BaseType::IntT, BaseType::BoolT])`
#[macro_export]
macro_rules! tuplet {
    ($($x:expr),* $(,)?) => ($crate::ast::tuplet_vec(vec![$($x),*]))
}
pub use tuplet;

/// Construct a tuple value from the child values
#[macro_export]
macro_rules! tuplev {
    ($($x:expr),* $(,)?) => ($crate::ast::tuplev_vec(vec![$($x),*]))
}
pub use tuplev;

pub fn add(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Add, l, r))
}

pub fn sub(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Sub, l, r))
}

pub fn mul(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Mul, l, r))
}

pub fn div(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Div, l, r))
}

pub fn less_than(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::LessThan, l, r))
}

pub fn less_eq(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::LessEq, l, r))
}

pub fn greater_eq(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::GreaterEq, l, r))
}

pub fn greater_than(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::GreaterThan, l, r))
}

pub fn eq(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Eq, l, r))
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

pub fn alloc(id: i64, amount: RcExpr, state: RcExpr, value_ty: BaseType) -> RcExpr {
    RcExpr::new(Expr::Alloc(id, amount, state, value_ty))
}

pub fn free(ptr: RcExpr, state: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Free, ptr, state))
}

pub fn twrite(addr: RcExpr, val: RcExpr, state: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Top(TernaryOp::Write, addr, val, state))
}

pub fn tprint(e: RcExpr, state: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Print, e, state))
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
pub fn write(ptr: RcExpr, val: RcExpr, state: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Top(TernaryOp::Write, ptr, val, state))
}

pub fn load(e: RcExpr, state: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::Load, e, state))
}

pub fn ptradd(ptr: RcExpr, i: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Bop(BinaryOp::PtrAdd, ptr, i))
}

pub fn call(s: &str, e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Call(s.to_string(), e))
}
/// a macro that wraps the children in
/// a vec for program. Also ensures the program has correct argument types.
/// e.g. `program!(main, f1, f2, f3)` becomes `TreeProgram { entry: main, functions: vec![f1, f2, f3] }`
#[macro_export]
macro_rules! program {
    ($main:expr, $($x:expr),* $(,)?) => ($crate::ast::program_vec($main, vec![$($x),*]))
}
pub use program;

/// Ensures the program has correct argument types
/// by calling `with_arg_types`.
pub fn program_vec(entry: RcExpr, functions: Vec<RcExpr>) -> TreeProgram {
    TreeProgram { entry, functions }.with_arg_types()
}

/// Create a switch given a predicate and a list of cases
/// e.g. `switch!(cond; case1, case2, case3)` becomes `switch_vec(cond, vec![case1, case2, case3])`
#[macro_export]
macro_rules! switch {
    ($arg:expr, $input:expr; $($x:expr),* $(,)?) => ($crate::ast::switch_vec($arg, $input, vec![$($x),*]))
}
pub use switch;

pub fn switch_vec(cond: RcExpr, input: RcExpr, cases: Vec<RcExpr>) -> RcExpr {
    RcExpr::new(Expr::Switch(cond, input, cases))
}

pub fn empty() -> RcExpr {
    RcExpr::new(Expr::Empty(Type::Unknown))
}

pub fn single(e: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Single(e))
}

pub fn cons(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Concat(single(l), r))
}

pub fn push(l: RcExpr, r: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Concat(r, single(l)))
}

pub fn concat(tuple: RcExpr, tuple2: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Concat(tuple, tuple2))
}

/// Create a tuple of elements.
/// e.g. `parallel!(e1, e2, e3)` becomes `Concat(Single(e1), Concat(Single(e2), Single(e3)))`
#[macro_export]
macro_rules! parallel {
    ($($x:expr),* $(,)?) => ($crate::ast::parallel_vec(vec![$($x),*]))
}
pub use parallel;

pub fn parallel_vec<I: IntoIterator<Item = RcExpr>>(es: I) -> RcExpr
where
    <I as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    let mut iter = es.into_iter().rev();
    if let Some(e) = iter.next() {
        iter.fold(single(e), |acc, x| cons(x, acc))
    } else {
        empty()
    }
}

/// A helper for ensuring the list of expressions is non-empty.
/// This prevents missing adding context to a leaf node (e.g. empty).
pub fn parallel_vec_nonempty<I: IntoIterator<Item = RcExpr>>(es: I) -> RcExpr
where
    <I as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    let es_vec = es.into_iter().collect::<Vec<_>>();
    if es_vec.is_empty() {
        panic!("Expected non-empty list of expressions in parallel_vec_nonempty");
    } else {
        parallel_vec(es_vec)
    }
}

pub fn arg_ty(ty: Type) -> RcExpr {
    RcExpr::new(Expr::Arg(ty))
}

/// Returns an argument with an unknown type.
/// Use `with_arg_types` to fill in the correct type.
pub fn arg() -> RcExpr {
    RcExpr::new(Expr::Arg(Type::Unknown))
}

/// An argument with an integer type.
pub fn iarg() -> RcExpr {
    RcExpr::new(Expr::Arg(base(intt())))
}

pub fn barg() -> RcExpr {
    RcExpr::new(Expr::Arg(base(boolt())))
}

pub fn getat(index: usize) -> RcExpr {
    get(arg(), index)
}

pub fn tif(cond: RcExpr, input: RcExpr, then_case: RcExpr, else_case: RcExpr) -> RcExpr {
    RcExpr::new(Expr::If(cond, input, then_case, else_case))
}

pub fn dowhile(inputs: RcExpr, pred_and_body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::DoWhile(inputs, pred_and_body))
}

pub fn function(name: &str, arg_ty: Type, ret_ty: Type, body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::Function(name.to_string(), arg_ty, ret_ty, body))
}

pub fn ttrue() -> RcExpr {
    RcExpr::new(Expr::Const(
        crate::schema::Constant::Bool(true),
        Type::Unknown,
    ))
}

pub fn ttrue_ty(ty: Type) -> RcExpr {
    RcExpr::new(Expr::Const(crate::schema::Constant::Bool(true), ty))
}

pub fn tfalse() -> RcExpr {
    RcExpr::new(Expr::Const(
        crate::schema::Constant::Bool(false),
        Type::Unknown,
    ))
}

pub fn tfalse_ty(ty: Type) -> RcExpr {
    RcExpr::new(Expr::Const(crate::schema::Constant::Bool(false), ty))
}

pub fn int(i: i64) -> RcExpr {
    RcExpr::new(Expr::Const(crate::schema::Constant::Int(i), Type::Unknown))
}

pub fn int_ty(i: i64, ty: Type) -> RcExpr {
    RcExpr::new(Expr::Const(crate::schema::Constant::Int(i), ty))
}

pub fn inloop(input: RcExpr, pred_output: RcExpr) -> Assumption {
    Assumption::InLoop(input, pred_output)
}

pub fn inif(is_then: bool, pred: RcExpr, input: RcExpr) -> Assumption {
    Assumption::InIf(is_then, pred, input)
}

pub fn inswitch(branch: i64, pred: RcExpr, input: RcExpr) -> Assumption {
    Assumption::InSwitch(branch, pred, input)
}

pub fn noctx() -> Assumption {
    Assumption::NoContext
}

pub fn inctx(assumption: Assumption, body: RcExpr) -> RcExpr {
    RcExpr::new(Expr::InContext(assumption, body))
}
