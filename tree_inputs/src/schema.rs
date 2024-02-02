//! This module mirrors `schema.egg`.
//! No implementation or conversion should
//! be implemented in this file.

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub entry: RcExpr,          // must be a function
    pub functions: Vec<RcExpr>, // a list of other functions
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ctx {
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    IntT,
    BoolT,
    FuncT(Rc<Type>, Rc<Type>),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Print,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Const(Ctx, Constant),
    Bop(BinaryOp, Rc<Expr>, Rc<Expr>),
    Uop(UnaryOp, Rc<Expr>),
    Get(Rc<Expr>, i64),
    Read(Rc<Expr>, Type),
    Call(String, Rc<Expr>),
    All(Ctx, Order, Vec<Rc<Expr>>),
    Switch(Rc<Expr>, Vec<Rc<Expr>>),
    If(Rc<Expr>, Rc<Expr>, Rc<Expr>),
    Input(Rc<Expr>),
    Arg(Type),
    Let(Rc<Expr>),
    DoWhile(Rc<Expr>, Rc<Expr>, Rc<Expr>),
    Function(String, Type, Type, Rc<Expr>),
}

pub type RcExpr = Rc<Expr>;
