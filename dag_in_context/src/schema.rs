//! This module mirrors `schema.egg`.
//! No implementation or conversion should
//! be implemented in this file.
//! Also see schema.egg for documentation

use ordered_float::OrderedFloat;
use std::rc::Rc;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Display)]
pub enum BaseType {
    IntT,
    FloatT,
    BoolT,
    PointerT(Box<BaseType>),
    StateT,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, PartialOrd, Ord)]
pub enum TernaryOp {
    Write,
    Select,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, PartialOrd, Ord)]
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
    Smax,
    Smin,
    Shl,
    Shr,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FEq,
    FLessThan,
    FGreaterThan,
    FLessEq,
    FGreaterEq,
    Fmax,
    Fmin,
    And,
    Or,
    PtrAdd,
    Load,
    Print,
    Free,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, PartialOrd, Ord)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    Int(i64),
    Bool(bool),
    Float(OrderedFloat<f64>),
}

/// A reference counted expression.
/// We want sharing between sub-expressions, so we use Rc instead of Box.
/// Invariant: Every shared sub-expression is re-used by the same Rc<Expr> (pointer equality).
/// This is important for the correctness of the interpreter, which makes this assumption.
/// NOTE: Please do not hash this. Hash a *const Expr instead. The hash function for RcExpr
/// is very slow due to sharing of subexpressions.
pub type RcExpr = Rc<Expr>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Assumption {
    InLoop(RcExpr, RcExpr),
    InFunc(String),
    InIf(bool, RcExpr, RcExpr),
    InSwitch(i64, RcExpr, RcExpr),
    WildCard(String),
}

impl Assumption {
    pub fn dummy() -> Assumption {
        Assumption::InFunc("dummy".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Const(Constant, Type, Assumption),
    Top(TernaryOp, RcExpr, RcExpr, RcExpr),
    Bop(BinaryOp, RcExpr, RcExpr),
    Uop(UnaryOp, RcExpr),
    Get(RcExpr, usize),
    Alloc(i64, RcExpr, RcExpr, BaseType),
    Call(String, RcExpr),
    Empty(Type, Assumption),
    Single(RcExpr),
    Concat(RcExpr, RcExpr),
    If(RcExpr, RcExpr, RcExpr, RcExpr),
    Switch(RcExpr, RcExpr, Vec<RcExpr>),
    DoWhile(RcExpr, RcExpr),
    Arg(Type, Assumption),
    Function(String, Type, Type, RcExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeProgram {
    /// must be a function
    pub entry: RcExpr,
    /// a list of other functions
    pub functions: Vec<RcExpr>,
}
