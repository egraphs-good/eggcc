use std::iter;

/// A an enum of all the constructors
/// in the IR.
/// A constructor is either something
/// that creates an [`Expr`]
/// or it is a list constructor.
/// The arguments of the [`Expr`] should not be used-
/// it is just there to specify the type of the constructor.
#[derive(Clone, Debug, PartialEq)]
pub enum Constructor {
    Expr(Expr),
    Cons,
    Nil,
}

// The constructor fields must have purposes such that this is maintained:
// - A ctor has one or more CapturedExpr fields iff it has exactly one
//   CapturingId field. The CapturingId field corresponds to the context of the
//   CapturedExpr field(s).
//   * Note that this applies to let/loop ids, but not the id in an arg.
//   * Note also that a call's function reference has purpose Static
// Invariants of a valid term in the IR:
// - A ReferencingId must match the nearest enclosing BindingId
// - It must typecheck (see typechecker in interpreter.rs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Purpose {
    Static(Sort), // some int, bool, order that parameterizes constructor
    CapturingId,
    ReferencingId,
    SubExpr,      // subexpression, e.g. Add's summand
    SubListExpr,  // sublistexpr, e.g. Switch's branch lsit
    CapturedExpr, // a body's outputs
}

impl Purpose {
    pub(crate) fn to_sort(self) -> Sort {
        match self {
            Purpose::CapturingId => Sort::IdSort,
            Purpose::ReferencingId => Sort::IdSort,
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
    /// Iterate over all constructors
    pub(crate) fn iter() -> impl Iterator<Item = Constructor> {
        Expr::iter()
            .map(Constructor::Expr)
            .chain(iter::once(Constructor::Cons))
            .chain(iter::once(Constructor::Nil))
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Constructor::Expr(expr) => expr.name(),
            Constructor::Cons => "Cons",
            Constructor::Nil => "Nil",
        }
    }

    pub(crate) fn is_pure(&self) -> bool {
        use Constructor::*;
        match self {
            Expr(expr) => expr.is_pure(),
            Cons | Nil => true,
        }
    }

    pub(crate) fn fields(&self) -> Vec<Field> {
        use Purpose::{CapturedExpr, CapturingId, ReferencingId, Static, SubExpr, SubListExpr};
        let f = |purpose, name| Field { purpose, name };
        match self {
            Constructor::Expr(Expr::Function(..)) => {
                vec![
                    f(Static(Sort::IdSort), "id"),
                    f(Static(Sort::String), "name"),
                    f(Static(Sort::Type), "tyin"),
                    f(Static(Sort::Type), "tyout"),
                    f(SubExpr, "out"),
                ]
            }
            Constructor::Expr(Expr::Program(..)) => {
                vec![f(SubListExpr, "functions")]
            }
            Constructor::Expr(Expr::Num(..)) => {
                vec![f(ReferencingId, "id"), f(Static(Sort::I64), "n")]
            }
            Constructor::Expr(Expr::Boolean(..)) => {
                vec![f(ReferencingId, "id"), f(Static(Sort::Bool), "b")]
            }
            Constructor::Expr(Expr::BOp(..)) => vec![
                f(Static(Sort::BinPureOp), "op"),
                f(SubExpr, "x"),
                f(SubExpr, "y"),
            ],
            Constructor::Expr(Expr::UOp(..)) => {
                vec![f(Static(Sort::UnaryPureOp), "op"), f(SubExpr, "x")]
            }
            Constructor::Expr(Expr::Get(..)) => vec![f(SubExpr, "tup"), f(Static(Sort::I64), "i")],
            Constructor::Expr(Expr::Print(..)) => vec![f(SubExpr, "printee")],
            Constructor::Expr(Expr::Read(..)) => {
                vec![f(SubExpr, "addr"), f(Static(Sort::Type), "type")]
            }
            Constructor::Expr(Expr::Write(..)) => {
                vec![f(SubExpr, "addr"), f(SubExpr, "data")]
            }
            Constructor::Expr(Expr::All(..)) => vec![
                f(ReferencingId, "id"),
                f(Static(Sort::Order), "order"),
                f(SubListExpr, "exprs"),
            ],
            Constructor::Expr(Expr::Switch(..)) => {
                vec![f(SubExpr, "pred"), f(SubListExpr, "branches")]
            }
            Constructor::Expr(Expr::Branch(..)) => {
                vec![f(CapturingId, "id"), f(SubExpr, "expr")]
            }
            Constructor::Expr(Expr::Loop(..)) => vec![
                f(CapturingId, "id"),
                f(SubExpr, "in"),
                f(CapturedExpr, "pred-and-output"),
            ],
            Constructor::Expr(Expr::Let(..)) => vec![
                f(CapturingId, "id"),
                f(SubExpr, "in"),
                f(CapturedExpr, "out"),
            ],
            Constructor::Expr(Expr::Arg(..)) => vec![f(ReferencingId, "id")],
            Constructor::Expr(Expr::Call(..)) => {
                vec![
                    f(Static(Sort::I64), "id"),
                    f(Static(Sort::String), "func"),
                    f(SubExpr, "arg"),
                ]
            }
            Constructor::Cons => {
                vec![f(SubExpr, "hd"), f(SubListExpr, "tl")]
            }
            Constructor::Nil => vec![],
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
            Constructor::Expr(_) => ESort::Expr,
            Constructor::Cons => ESort::ListExpr,
            Constructor::Nil => ESort::ListExpr,
        }
    }

    pub(crate) fn creates_context(&self) -> bool {
        self.fields()
            .iter()
            .any(|field| field.purpose == Purpose::CapturingId)
    }
}

#[cfg(test)]
use std::collections::HashSet;

use strum::IntoEnumIterator;

use crate::expr::{ESort, Expr, Sort};

#[test]
fn no_duplicate_field_names() {
    for ctor in Constructor::iter() {
        let mut seen: HashSet<String> = HashSet::new();
        for field in ctor.fields() {
            assert!(!seen.contains(field.name));
            seen.insert(field.name.to_string());
        }
    }
}
