//! This module mirrors `schema.egg`.
//! No implementation or conversion should
//! be implemented in this file.

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    IntT,
    BoolT,
    PointerT(Box<Type>),
    /// Nested tuple types are not allowed.
    TupleT(Vec<Rc<Type>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    LessThan,
    And,
    Or,
    Write,
    PtrAdd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Print,
    Load,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    Parallel,
    Sequential,
}

pub type RcExpr = Rc<Expr>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assumption {
    InLet(RcExpr),
    InLoop(RcExpr, RcExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Const(Constant),
    Bop(BinaryOp, RcExpr, RcExpr),
    Uop(UnaryOp, RcExpr),
    Get(RcExpr, usize),
    Alloc(RcExpr, Type),
    Call(String, RcExpr),
    Empty,
    Single(RcExpr),
    Extend(Order, RcExpr, RcExpr),
    Switch(RcExpr, Vec<RcExpr>),
    If(RcExpr, RcExpr, RcExpr),
    Let(RcExpr, RcExpr),
    DoWhile(RcExpr, RcExpr),
    Arg,
    Assume(Assumption, RcExpr),
    Function(String, Type, Type, RcExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeProgram {
    /// must be a function
    pub entry: RcExpr,
    /// a list of other functions
    pub functions: Vec<RcExpr>,
}
