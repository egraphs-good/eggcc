use std::iter;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Sort {
    Expr,
    ListExpr,
    Order,
    I64,
    Bool,
}

impl Sort {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Sort::Expr => "Expr",
            Sort::ListExpr => "ListExpr",
            Sort::Order => "Order",
            Sort::I64 => "i64",
            Sort::Bool => "bool",
        }
    }
}

// Subset of sorts that refer to expressions
#[derive(EnumIter)]
pub(crate) enum ESort {
    Expr,
    ListExpr,
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

#[derive(Clone, Copy, EnumIter, PartialEq)]
pub(crate) enum Constructor {
    Num,
    Boolean,
    UnitExpr,
    Add,
    Sub,
    Mul,
    LessThan,
    And,
    Or,
    Not,
    Get,
    Print,
    Read,
    Write,
    All,
    Switch,
    Loop,
    Let,
    Arg,
    Call,
    Cons,
    Nil,
}

// The constructor fields must purposes such that this is maintained:
// - A ctor has one or more CapturedExpr fields iff it has exactly one
//   CapturingId field. The CapturingId field corresponds to the context of the
//   CapturedExpr field(s).
//   * Note that this applies to let/loop ids, but not the id in an arg.
//   * Note also that a call's function reference has purpose Static
// Invariants of a valid term in the IR:
// - A ReferencingId must match the nearest enclosing BindingId
// - It must typecheck (see typechecker in interpreter.rs).
pub(crate) enum Purpose {
    Static(Sort), // some int, bool, order that parameterizes constructor
    CapturingId,
    ReferencingId,
    SubExpr,      // subexpression, e.g. Add's summand
    SubListExpr,  // sublistexpr, e.g. Switch's branch lsit
    CapturedExpr, // a body's outputs
}

impl Purpose {
    pub(crate) fn to_sort(&self) -> Sort {
        match self {
            Purpose::CapturingId => Sort::I64,
            Purpose::ReferencingId => Sort::I64,
            Purpose::SubExpr => Sort::Expr,
            Purpose::CapturedExpr => Sort::Expr,
            Purpose::SubListExpr => Sort::ListExpr,
            Purpose::Static(sort) => *sort,
        }
    }
}

pub(crate) struct Field {
    pub purpose: Purpose,
    pub name: &'static str,
}

impl Field {
    pub(crate) fn sort(&self) -> Sort {
        self.purpose.to_sort()
    }
}

impl Constructor {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Constructor::Num => "Num",
            Constructor::Boolean => "Boolean",
            Constructor::UnitExpr => "UnitExpr",
            Constructor::Add => "Add",
            Constructor::Sub => "Sub",
            Constructor::Mul => "Mul",
            Constructor::LessThan => "LessThan",
            Constructor::And => "And",
            Constructor::Or => "Or",
            Constructor::Not => "Not",
            Constructor::Get => "Get",
            Constructor::Print => "Print",
            Constructor::Read => "Read",
            Constructor::Write => "Write",
            Constructor::All => "All",
            Constructor::Switch => "Switch",
            Constructor::Loop => "Loop",
            Constructor::Let => "Let",
            Constructor::Arg => "Arg",
            Constructor::Call => "Call",
            Constructor::Cons => "Cons",
            Constructor::Nil => "Nil",
        }
    }

    pub(crate) fn fields(&self) -> Vec<Field> {
        use Purpose::{CapturedExpr, CapturingId, ReferencingId, Static, SubExpr, SubListExpr};
        let f = |purpose, name| Field { purpose, name };
        match self {
            Constructor::Num => vec![f(Static(Sort::I64), "n")],
            Constructor::Boolean => vec![f(Static(Sort::Bool), "b")],
            Constructor::UnitExpr => vec![],
            Constructor::Add => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            Constructor::Sub => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            Constructor::Mul => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            Constructor::LessThan => {
                vec![f(SubExpr, "x"), f(SubExpr, "y")]
            }
            Constructor::And => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            Constructor::Or => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            Constructor::Not => vec![f(SubExpr, "x")],
            Constructor::Get => vec![f(SubExpr, "tup"), f(Static(Sort::I64), "i")],
            Constructor::Print => vec![f(SubExpr, "printee")],
            Constructor::Read => vec![f(SubExpr, "addr")],
            Constructor::Write => vec![f(SubExpr, "addr"), f(SubExpr, "data")],
            Constructor::All => vec![f(Static(Sort::Order), "order"), f(SubListExpr, "exprs")],
            Constructor::Switch => vec![f(SubExpr, "pred"), f(SubListExpr, "branches")],
            Constructor::Loop => vec![
                f(CapturingId, "id"),
                f(SubExpr, "in"),
                f(CapturedExpr, "pred-and-output"),
            ],
            Constructor::Let => vec![
                f(CapturingId, "id"),
                f(SubExpr, "in"),
                f(CapturedExpr, "out"),
            ],
            Constructor::Arg => vec![f(ReferencingId, "id")],
            Constructor::Call => vec![f(Static(Sort::I64), "f"), f(SubExpr, "arg")],
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
        F: FnMut(&Field) -> &str,
    {
        let without_parens = iter::once(self.name())
            .chain(self.fields().iter().map(f))
            .collect::<Vec<_>>()
            .join(" ");
        format!("({without_parens})")
    }

    pub(crate) fn sort(&self) -> ESort {
        match self {
            Constructor::Num => ESort::Expr,
            Constructor::Boolean => ESort::Expr,
            Constructor::UnitExpr => ESort::Expr,
            Constructor::Add => ESort::Expr,
            Constructor::Sub => ESort::Expr,
            Constructor::Mul => ESort::Expr,
            Constructor::LessThan => ESort::Expr,
            Constructor::And => ESort::Expr,
            Constructor::Or => ESort::Expr,
            Constructor::Not => ESort::Expr,
            Constructor::Get => ESort::Expr,
            Constructor::Print => ESort::Expr,
            Constructor::Read => ESort::Expr,
            Constructor::Write => ESort::Expr,
            Constructor::All => ESort::Expr,
            Constructor::Switch => ESort::Expr,
            Constructor::Loop => ESort::Expr,
            Constructor::Let => ESort::Expr,
            Constructor::Arg => ESort::Expr,
            Constructor::Call => ESort::Expr,
            Constructor::Cons => ESort::ListExpr,
            Constructor::Nil => ESort::ListExpr,
        }
    }
}

#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use strum::IntoEnumIterator;

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
