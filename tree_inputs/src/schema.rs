//! This module mirrors `schema.egg`.
//! No implementation or conversion should
//! be implemented in this file.

use std::rc::Rc;

pub enum BaseType {
    IntT,
    BoolT,
}

pub enum Type {
    PointerT(BaseType),
    /// Nested tuple types are not allowed.
    TupleT(Vec<BaseType>),
}

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

pub enum UnaryOp {
    Not,
    Print,
    Load,
}

pub enum Constant {
    Int(i64),
    Bool(bool),
}

pub enum Order {
    Parallel,
    Sequential,
}

pub type RcExpr = Rc<Expr>;

pub enum Assumption {
    InLet(RcExpr),
    InLoop(RcExpr, RcExpr),
}

pub enum Expr {
    Const(Constant),
    Bop(BinaryOp, RcExpr, RcExpr),
    Uop(UnaryOp, RcExpr),
    Get(RcExpr, i64),
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

pub struct TreeProgram {
    /// must be a function
    pub entry: RcExpr,
    /// a list of other functions
    pub functions: Vec<RcExpr>,
}
