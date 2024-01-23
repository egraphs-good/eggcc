pub(crate) mod body_contains;
pub(crate) mod conditional_invariant_code_motion;
pub(crate) mod deep_copy;
pub(crate) mod function_inlining;
pub mod interpreter;
pub(crate) mod ir;
pub(crate) mod is_valid;
pub(crate) mod purity_analysis;
pub(crate) mod simple;
pub(crate) mod subst;
pub(crate) mod switch_rewrites;
pub(crate) mod util;

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Parallel,
    Sequential,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Id(i64);

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    Boolean(bool),
    Unit,
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Get(Box<Expr>, usize),
    Print(Box<Expr>),
    Read(Box<Expr>),
    Write(Box<Expr>, Box<Expr>),
    All(Order, Vec<Expr>),
    Switch(Box<Expr>, Vec<Expr>),
    Loop(Id, Box<Expr>, Box<Expr>),
    Let(Id, Box<Expr>, Box<Expr>),
    Arg(Id),
    Function(Id, Box<Expr>),
    Call(Id, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(i64),
    Boolean(bool),
    Unit,
    Tuple(Vec<Value>),
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Num,
    Boolean,
    Unit,
    Tuple(Vec<Type>),
}

pub enum TypeError {
    ExpectedType(Expr, Type, Type),
    ExpectedTupleType(Expr, Type),
    ExpectedLoopOutputType(Expr, Type),
    NoArg(Expr),
}
