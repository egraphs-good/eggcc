use std::fmt::{Display, Formatter};
use strum_macros::EnumIter;

use crate::{
    ast::{boolt, intt},
    schema::{BinaryOp, Constant, Expr, RcExpr, TreeProgram, Type, UnaryOp},
};

/// Display for Constant implements a
/// rust-readable representation using
/// the sugar in `ast.rs`.
impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl BinaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use BinaryOp::*;
        match self {
            Add => "Add",
            Sub => "Sub",
            Mul => "Mul",
            LessThan => "LessThan",
            And => "And",
            Or => "Or",
            Write => "Write",
            PtrAdd => "PtrAdd",
        }
    }
}

impl UnaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use UnaryOp::*;
        match self {
            Not => "Not",
            Print => "Print",
            Load => "Load",
        }
    }
}

impl Expr {
    pub fn constructor(&self) -> Constructor {
        match self {
            Expr::Function(..) => Constructor::Function,
            Expr::Const(..) => Constructor::Const,
            Expr::Bop(..) => Constructor::Bop,
            Expr::Uop(..) => Constructor::Uop,
            Expr::Get(..) => Constructor::Get,
            Expr::Concat(..) => Constructor::Concat,
            Expr::Single(..) => Constructor::Single,
            Expr::Switch(..) => Constructor::Switch,
            Expr::If(..) => Constructor::If,
            Expr::DoWhile(..) => Constructor::DoWhile,
            Expr::Let(..) => Constructor::Let,
            Expr::Arg(..) => Constructor::Arg,
            Expr::Call(..) => Constructor::Call,
            Expr::Empty => Constructor::Empty,
            Expr::Alloc(..) => Constructor::Alloc,
            Expr::Assume(..) => Constructor::Assume,
        }
    }
    pub fn func_name(&self) -> Option<String> {
        match self {
            Expr::Function(name, _, _, _) => Some(name.clone()),
            _ => None,
        }
    }

    pub fn func_input_ty(&self) -> Option<Type> {
        match self {
            Expr::Function(_, ty, _, _) => Some(ty.clone()),
            _ => None,
        }
    }

    pub fn func_output_ty(&self) -> Option<Type> {
        match self {
            Expr::Function(_, _, ty, _) => Some(ty.clone()),
            _ => None,
        }
    }

    pub fn func_body(&self) -> Option<&RcExpr> {
        match self {
            Expr::Function(_, _, _, body) => Some(body),
            _ => None,
        }
    }

    /// Converts this expression to a program, and ensures arguments
    /// have the correct type by calling `with_arg_types`.
    pub fn to_program(self: &RcExpr, input_ty: Type, output_ty: Type) -> TreeProgram {
        match self.as_ref() {
            Expr::Function(..) => TreeProgram {
                entry: self.clone(),
                functions: vec![],
            },
            _ => TreeProgram {
                entry: RcExpr::new(Expr::Function(
                    "main".to_string(),
                    input_ty,
                    output_ty,
                    self.clone(),
                )),
                functions: vec![],
            },
        }
        .with_arg_types()
    }
}

impl TreeProgram {
    pub fn get_function(&self, name: &str) -> Option<&RcExpr> {
        if self.entry.func_name() == Some(name.to_string()) {
            return Some(&self.entry);
        }
        self.functions
            .iter()
            .find(|expr| expr.func_name() == Some(name.to_string()))
    }
}

use std::iter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Sort {
    Expr,
    ListExpr,
    Order,
    BinaryOp,
    UnaryOp,
    I64,
    Type,
    String,
    Constant,
    Assumption,
}

impl Sort {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Sort::Expr => "Expr",
            Sort::ListExpr => "ListExpr",
            Sort::Order => "Order",
            Sort::I64 => "i64",
            Sort::String => "String",
            Sort::Type => "Type",
            Sort::BinaryOp => "BinaryOp",
            Sort::UnaryOp => "UnaryOp",
            Sort::Constant => "Constant",
            Sort::Assumption => "Assumption",
        }
    }
}

#[derive(Debug, EnumIter, PartialEq)]
pub(crate) enum ESort {
    Expr,
    ListExpr,
}

impl std::fmt::Display for ESort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ESort {
    pub(crate) fn to_sort(&self) -> Sort {
        match self {
            ESort::Expr => Sort::Expr,
            ESort::ListExpr => Sort::ListExpr,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.to_sort().name()
    }
}

#[derive(Clone, Debug, EnumIter, PartialEq)]
pub enum Constructor {
    Function,
    Const,
    Bop,
    Uop,
    Get,
    Concat,
    Single,
    Switch,
    If,
    DoWhile,
    Let,
    Arg,
    Call,
    Empty,
    Cons,
    Nil,
    Alloc,
    Assume,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Purpose {
    Static(Sort), // some int, bool, order that parameterizes constructor
    SubExpr,      // subexpression, e.g. Add's summand
    SubListExpr,  // sublistexpr, e.g. Switch's branch lsit
    CapturedExpr, // a body's outputs
}

impl Purpose {
    pub(crate) fn to_sort(self) -> Sort {
        match self {
            Purpose::SubExpr => Sort::Expr,
            Purpose::CapturedExpr => Sort::Expr,
            Purpose::SubListExpr => Sort::ListExpr,
            Purpose::Static(sort) => sort,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Field {
    pub purpose: Purpose,
    pub name: &'static str,
}

impl Field {
    pub(crate) fn sort(&self) -> Sort {
        self.purpose.to_sort()
    }

    pub(crate) fn var(&self) -> String {
        format!("_{name}", name = self.name)
    }
}

impl Constructor {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Constructor::Function => "Function",
            Constructor::Const => "Const",
            Constructor::Bop => "Bop",
            Constructor::Uop => "Uop",
            Constructor::Get => "Get",
            Constructor::Concat => "Concat",
            Constructor::Single => "Single",
            Constructor::Switch => "Switch",
            Constructor::If => "If",
            Constructor::DoWhile => "DoWhile",
            Constructor::Let => "Let",
            Constructor::Arg => "Arg",
            Constructor::Call => "Call",
            Constructor::Empty => "Empty",
            Constructor::Alloc => "Alloc",
            Constructor::Assume => "Assume",
            Constructor::Cons => "Cons",
            Constructor::Nil => "Nil",
        }
    }

    pub(crate) fn fields(&self) -> Vec<Field> {
        use Purpose::{CapturedExpr, Static, SubExpr, SubListExpr};
        let f = |purpose, name| Field { purpose, name };
        match self {
            Constructor::Function => {
                vec![
                    f(Static(Sort::String), "name"),
                    f(Static(Sort::Type), "tyin"),
                    f(Static(Sort::Type), "tyout"),
                    f(SubExpr, "out"),
                ]
            }
            Constructor::Const => {
                vec![f(Static(Sort::Constant), "n")]
            }
            Constructor::Bop => vec![
                f(Static(Sort::BinaryOp), "op"),
                f(SubExpr, "x"),
                f(SubExpr, "y"),
            ],
            Constructor::Uop => {
                vec![f(Static(Sort::UnaryOp), "op"), f(SubExpr, "x")]
            }
            Constructor::Get => vec![f(SubExpr, "tup"), f(Static(Sort::I64), "i")],
            Constructor::Concat => {
                vec![
                    f(Static(Sort::Order), "order"),
                    f(SubExpr, "x"),
                    f(SubExpr, "y"),
                ]
            }
            Constructor::Single => {
                vec![f(SubExpr, "x")]
            }
            Constructor::Switch => {
                vec![f(SubExpr, "pred"), f(SubListExpr, "branches")]
            }
            Constructor::If => {
                vec![f(SubExpr, "pred"), f(SubExpr, "then"), f(SubExpr, "else")]
            }
            Constructor::DoWhile => {
                vec![f(SubExpr, "in"), f(CapturedExpr, "pred-and-output")]
            }
            Constructor::Let => vec![f(SubExpr, "in"), f(CapturedExpr, "out")],
            Constructor::Arg => vec![f(Static(Sort::Type), "ty")],
            Constructor::Call => {
                vec![f(Static(Sort::String), "func"), f(SubExpr, "arg")]
            }
            Constructor::Empty => vec![],
            Constructor::Cons => vec![f(SubExpr, "hd"), f(SubListExpr, "tl")],
            Constructor::Nil => vec![],
            Constructor::Alloc => vec![f(SubExpr, "e"), f(Static(Sort::Type), "ty")],
            Constructor::Assume => vec![f(Static(Sort::Assumption), "assumption"), f(SubExpr, "e")],
        }
    }

    pub(crate) fn filter_map_fields<F, T>(&self, f: F) -> Vec<T>
    where
        F: FnMut(&Field) -> Option<T>,
    {
        self.fields().iter().filter_map(f).collect::<Vec<_>>()
    }

    pub(crate) fn construct<F>(&self, f: F) -> String
    where
        F: FnMut(&Field) -> String,
    {
        let without_parens = iter::once(self.name().to_string())
            .chain(self.fields().iter().map(f))
            .collect::<Vec<_>>()
            .join(" ");
        format!("({without_parens})")
    }

    pub(crate) fn sort(&self) -> ESort {
        match self {
            Constructor::Function => ESort::Expr,
            Constructor::Const => ESort::Expr,
            Constructor::Bop => ESort::Expr,
            Constructor::Uop => ESort::Expr,
            Constructor::Get => ESort::Expr,
            Constructor::Concat => ESort::Expr,
            Constructor::Single => ESort::Expr,
            Constructor::Switch => ESort::Expr,
            Constructor::If => ESort::Expr,
            Constructor::DoWhile => ESort::Expr,
            Constructor::Let => ESort::Expr,
            Constructor::Arg => ESort::Expr,
            Constructor::Call => ESort::Expr,
            Constructor::Empty => ESort::Expr,
            Constructor::Alloc => ESort::Expr,
            Constructor::Assume => ESort::Expr,
            Constructor::Cons => ESort::ListExpr,
            Constructor::Nil => ESort::ListExpr,
        }
    }
}

impl BinaryOp {
    /// When a binary op has concrete input sorts, return them.
    pub(crate) fn types(&self) -> Option<(Type, Type, Type)> {
        match self {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => Some((intt(), intt(), intt())),
            BinaryOp::And | BinaryOp::Or => Some((boolt(), boolt(), boolt())),
            BinaryOp::LessThan => Some((intt(), intt(), boolt())),
            BinaryOp::Write => None,
            BinaryOp::PtrAdd => None,
        }
    }
}

impl UnaryOp {
    pub(crate) fn types(&self) -> Option<(Type, Type)> {
        match self {
            UnaryOp::Not => Some((boolt(), boolt())),
            UnaryOp::Print => None,
            UnaryOp::Load => None,
        }
    }
}
