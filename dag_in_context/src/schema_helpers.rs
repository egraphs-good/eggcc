use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};
use strum_macros::EnumIter;

use crate::{
    ast::{base, boolt, intt},
    schema::{
        Assumption, BaseType, BinaryOp, Constant, Expr, RcExpr, TernaryOp, TreeProgram, Type,
        UnaryOp,
    },
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
        let rcexpr = RcExpr::new(self.clone());
        let (term, termdag) = rcexpr.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Display for Assumption {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl TernaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use TernaryOp::*;
        match self {
            Write => "Write",
        }
    }
}

impl BinaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use BinaryOp::*;
        match self {
            Add => "Add",
            Sub => "Sub",
            Mul => "Mul",
            Div => "Div",
            Eq => "Eq",
            GreaterThan => "GreaterThan",
            LessThan => "LessThan",
            GreaterEq => "GreaterEq",
            LessEq => "LessEq",
            And => "And",
            Or => "Or",
            Load => "Load",
            Free => "Free",
            Print => "Print",
            PtrAdd => "PtrAdd",
        }
    }
}

impl UnaryOp {
    pub(crate) fn name(&self) -> &'static str {
        use UnaryOp::*;
        match self {
            Not => "Not",
        }
    }
}

impl Display for TreeProgram {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
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
            Expr::Arg(..) => Constructor::Arg,
            Expr::Call(..) => Constructor::Call,
            Expr::Empty(..) => Constructor::Empty,
            Expr::Alloc(..) => Constructor::Alloc,
            Expr::Top(..) => Constructor::Top,
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

    pub fn func_to_program(&self) -> TreeProgram {
        match self {
            Expr::Function(name, input_ty, output_ty, body) => TreeProgram {
                entry: RcExpr::new(Expr::Function(
                    name.clone(),
                    input_ty.clone(),
                    output_ty.clone(),
                    body.clone(),
                )),
                functions: vec![],
            },
            _ => panic!("Expected function"),
        }
        .with_arg_types()
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

    /// Get the children of this expression that are still in the same scope
    /// For context nodes, doesn't include the context (which is an assumption)
    pub fn children_same_scope(self: &RcExpr) -> Vec<RcExpr> {
        match self.as_ref() {
            Expr::Function(_, _, _, body) => vec![body.clone()],
            Expr::Const(..) => vec![],
            Expr::Top(_, x, y, z) => vec![x.clone(), y.clone(), z.clone()],
            Expr::Bop(_, x, y) => vec![x.clone(), y.clone()],
            Expr::Uop(_, x) => vec![x.clone()],
            Expr::Get(x, _) => vec![x.clone()],
            Expr::Alloc(_, x, y, _) => vec![x.clone(), y.clone()],
            Expr::Call(_, x) => vec![x.clone()],
            Expr::Empty(_, _) => vec![],
            Expr::Single(x) => vec![x.clone()],
            Expr::Concat(x, y) => vec![x.clone(), y.clone()],
            Expr::Switch(x, inputs, _branches) => {
                let children = vec![x.clone(), inputs.clone()];
                children
            }
            Expr::If(x, inputs, _y, _z) => {
                let children = vec![x.clone(), inputs.clone()];
                children
            }
            Expr::DoWhile(inputs, _body) => vec![inputs.clone()],
            Expr::Arg(_, _) => vec![],
        }
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

    pub fn pretty(&self) -> String {
        let (term, termdag) = self.to_egglog();
        let expr = termdag.term_to_expr(&term);
        expr.to_sexp().pretty()
    }
}

use std::iter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Sort {
    Expr,
    ListExpr,
    BinaryOp,
    UnaryOp,
    TernaryOp,
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
            Sort::I64 => "i64",
            Sort::String => "String",
            Sort::Type => "Type",
            Sort::BinaryOp => "BinaryOp",
            Sort::UnaryOp => "UnaryOp",
            Sort::TernaryOp => "TernaryOp",
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
    Top,
    Bop,
    Uop,
    Get,
    Concat,
    Single,
    Switch,
    If,
    DoWhile,
    Arg,
    Call,
    Empty,
    Cons,
    Nil,
    Alloc,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Purpose {
    Static(Sort),        // some int, bool, order that parameterizes constructor
    SubExpr,             // subexpression, e.g. Add's summand
    CapturedSubListExpr, // a swtich's branches
    CapturedExpr,        // a body's outputs
}

impl Purpose {
    pub(crate) fn to_sort(self) -> Sort {
        match self {
            Purpose::SubExpr => Sort::Expr,
            Purpose::CapturedExpr => Sort::Expr,
            Purpose::CapturedSubListExpr => Sort::ListExpr,
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
            Constructor::Arg => "Arg",
            Constructor::Call => "Call",
            Constructor::Empty => "Empty",
            Constructor::Alloc => "Alloc",
            Constructor::Cons => "Cons",
            Constructor::Nil => "Nil",
            Constructor::Top => "Top",
        }
    }

    pub(crate) fn fields(&self) -> Vec<Field> {
        use Purpose::{CapturedExpr, CapturedSubListExpr, Static, SubExpr};
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
                vec![
                    f(Static(Sort::Constant), "n"),
                    f(Static(Sort::Type), "ty"),
                    f(SubExpr(Sort::Assumption), "ctx"),
                ]
            }
            Constructor::Top => vec![
                f(Static(Sort::TernaryOp), "op"),
                f(SubExpr, "x"),
                f(SubExpr, "y"),
                f(SubExpr, "z"),
            ],
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
                vec![f(SubExpr, "x"), f(SubExpr, "y")]
            }
            Constructor::Single => {
                vec![f(SubExpr, "x")]
            }
            Constructor::Switch => {
                vec![
                    f(SubExpr, "pred"),
                    f(SubExpr, "inputs"),
                    f(CapturedSubListExpr, "branches"),
                ]
            }
            Constructor::If => {
                vec![
                    f(SubExpr, "pred"),
                    f(SubExpr, "input"),
                    f(CapturedExpr, "then"),
                    f(CapturedExpr, "else"),
                ]
            }
            Constructor::DoWhile => {
                vec![f(SubExpr, "in"), f(CapturedExpr, "pred-and-output")]
            }
            Constructor::Arg => vec![
                f(Static(Sort::Type), "ty"),
                f(Static(Sort::Assumption), "ctx"),
            ],
            Constructor::Call => {
                vec![f(Static(Sort::String), "func"), f(SubExpr, "arg")]
            }
            Constructor::Empty => vec![
                f(Static(Sort::Type), "ty"),
                f(Static(Sort::Assumption), "ctx"),
            ],
            Constructor::Cons => vec![f(SubExpr, "hd"), f(CapturedSubListExpr, "tl")],
            Constructor::Nil => vec![],
            Constructor::Alloc => vec![
                f(Static(Sort::I64), "id"),
                f(SubExpr, "e"),
                f(SubExpr, "state"),
                f(Static(Sort::Type), "ty"),
            ],
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
            Constructor::Top => ESort::Expr,
            Constructor::Bop => ESort::Expr,
            Constructor::Uop => ESort::Expr,
            Constructor::Get => ESort::Expr,
            Constructor::Concat => ESort::Expr,
            Constructor::Single => ESort::Expr,
            Constructor::Switch => ESort::Expr,
            Constructor::If => ESort::Expr,
            Constructor::DoWhile => ESort::Expr,
            Constructor::Arg => ESort::Expr,
            Constructor::Call => ESort::Expr,
            Constructor::Empty => ESort::Expr,
            Constructor::Alloc => ESort::Expr,
            Constructor::Cons => ESort::ListExpr,
            Constructor::Nil => ESort::ListExpr,
        }
    }
}

impl BinaryOp {
    /// When a binary op has concrete input sorts, return them.
    pub fn types(&self) -> Option<(Type, Type, Type)> {
        match self {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                Some((base(intt()), base(intt()), base(intt())))
            }
            BinaryOp::And | BinaryOp::Or => Some((base(boolt()), base(boolt()), base(boolt()))),
            BinaryOp::LessThan
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterEq
            | BinaryOp::LessEq
            | BinaryOp::Eq => Some((base(intt()), base(intt()), base(boolt()))),
            BinaryOp::Load => None,
            BinaryOp::Free => None,
            BinaryOp::Print => None,
            BinaryOp::PtrAdd => None,
        }
    }
}

impl UnaryOp {
    pub(crate) fn types(&self) -> Option<(Type, Type)> {
        match self {
            UnaryOp::Not => Some((base(boolt()), base(boolt()))),
        }
    }
}

/// used to hash an assumption
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssumptionRef {
    InLoop(*const Expr, *const Expr),
    NoContext,
    InIf(bool, *const Expr, *const Expr),
    InSwitch(i64, *const Expr, *const Expr),
    WildCard(String),
}

impl Assumption {
    pub fn to_ref(&self) -> AssumptionRef {
        match self {
            Assumption::InLoop(inputs, pred_and_body) => {
                AssumptionRef::InLoop(Rc::as_ptr(inputs), Rc::as_ptr(pred_and_body))
            }
            Assumption::NoContext => AssumptionRef::NoContext,
            Assumption::InIf(b, pred, input) => {
                AssumptionRef::InIf(*b, Rc::as_ptr(pred), Rc::as_ptr(input))
            }
            Assumption::InSwitch(branch, pred, input) => {
                AssumptionRef::InSwitch(*branch, Rc::as_ptr(pred), Rc::as_ptr(input))
            }
            Assumption::WildCard(str) => AssumptionRef::WildCard(str.clone()),
        }
    }
}

impl BaseType {
    pub fn contains_state(&self) -> bool {
        match self {
            BaseType::IntT => false,
            BaseType::BoolT => false,
            BaseType::PointerT(inner) => {
                assert!(!inner.contains_state(), "Pointers can't contain state");
                false
            }
            BaseType::StateT => true,
        }
    }
}

impl Type {
    pub fn contains_state(&self) -> bool {
        match self {
            Type::Base(basety) => basety.contains_state(),
            Type::TupleT(types) => types.iter().any(|ty| ty.contains_state()),
            Type::Unknown => panic!("Unknown type"),
        }
    }
}
