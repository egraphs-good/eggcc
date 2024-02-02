//! This module mirrors `schema.egg`.
//! No implementation or conversion should
//! be implemented in this file.

use std::rc::Rc;

struct Program {
    entry: Expr,          // must be a function
    functions: Vec<Expr>, // a list of other functions
}

enum Ctx {
    Global,
}

enum Type {
    IntT,
    BoolT,
    FuncT(Rc<Type>, Rc<Type>),
    TupleT(Vec<Rc<Type>>),
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
    LessThan,
    And,
    Or,
    Write,
}

enum UnaryOp {
    Not,
    Print,
}

enum Constant {
    Int(i64),
    Bool(bool),
}

enum Order {
    Parallel,
    Sequential,
}

enum Expr {
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
