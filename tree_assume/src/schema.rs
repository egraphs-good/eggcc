//! This module mirrors `schema.egg`.
//! No implementation or conversion should
//! be implemented in this file.

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    IntT,
    BoolT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Base(BaseType),
    PointerT(BaseType),
    /// Nested tuple types are not allowed.
    TupleT(Vec<Type>),
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
    Reversed,
}

/// A reference counted expression.
/// We want sharing between sub-expressions, so we
/// use Rc instead of Box.
pub type RcExpr = Rc<Expr>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assumption {
    InLet(RcExpr),
    InLoop(RcExpr, RcExpr),
    InFunc(String),
    InIf(bool, RcExpr),
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
    Concat(Order, RcExpr, RcExpr),
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
