//! This module mirrors `schema.egg`.
//! No implementation or conversion should
//! be implemented in this file.
//! Also see schema.egg for documentation

use std::rc::Rc;
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    IntT,
    BoolT,
    PointerT(Box<BaseType>),
    StateT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Base(BaseType),
    /// Nested tuple types are not allowed.
    TupleT(Vec<BaseType>),
    /// Before `with_arg_types`, users of this IR can leave unknown types
    /// in arguments.
    /// When all types are present except for unknowns in arguments,
    /// `with_arg_types` succeeds and the unknowns are replaced with the correct types.
    /// `to_egglog` calls `with_arg_types`, so there are never any
    /// unknown types in the egraph.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum TernaryOp {
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    LessThan,
    GreaterThan,
    LessEq,
    GreaterEq,
    And,
    Or,
    PtrAdd,
    Load,
    Print,
    Free,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum UnaryOp {
    Not,
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
    FakeState,
    Const(Constant, Type),
    Top(TernaryOp, RcExpr, RcExpr, RcExpr),
    Bop(BinaryOp, RcExpr, RcExpr),
    Uop(UnaryOp, RcExpr),
    Get(RcExpr, usize),
    Alloc(RcExpr, RcExpr, Type),
    Call(String, RcExpr),
    Empty(Type),
    Single(RcExpr),
    Concat(Order, RcExpr, RcExpr),
    Switch(RcExpr, Vec<RcExpr>),
    If(RcExpr, RcExpr, RcExpr),
    Let(RcExpr, RcExpr),
    DoWhile(RcExpr, RcExpr),
    Arg(Type),
    InContext(Assumption, RcExpr),
    Function(String, Type, Type, RcExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeProgram {
    /// must be a function
    pub entry: RcExpr,
    /// a list of other functions
    pub functions: Vec<RcExpr>,
}
