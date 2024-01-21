#![allow(dead_code)]

use std::iter::{self, once};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Sort {
    Expr,
    ListExpr,
    SExpr, // "shared expr"
    ListSExpr,
    Order,
    IdSort,
    I64,
    Bool,
}

impl Sort {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Sort::Expr => "Expr",
            Sort::ListExpr => "ListExpr",
            Sort::SExpr => "SExpr",
            Sort::ListSExpr => "ListSExpr",
            Sort::Order => "Order",
            Sort::IdSort => "IdSort",
            Sort::I64 => "i64",
            Sort::Bool => "bool",
        }
    }
}

// Subset of sorts that refer to expressions
#[derive(Debug, EnumIter, PartialEq)]
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

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
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
            Constructor::Num => vec![f(ReferencingId, "id"), f(Static(Sort::I64), "n")],
            Constructor::Boolean => vec![f(ReferencingId, "id"), f(Static(Sort::Bool), "b")],
            Constructor::UnitExpr => vec![f(ReferencingId, "id")],
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

    pub(crate) fn creates_context(&self) -> bool {
        self.fields()
            .iter()
            .any(|field| field.purpose == Purpose::CapturingId)
    }
}

#[cfg(test)]
use std::collections::HashSet;

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

// SHARED EXPPS

// Subset of sorts that refer to shared expressions
#[derive(Debug, EnumIter, PartialEq)]
pub(crate) enum SESort {
    SExpr,
    ListSExpr,
}
impl SESort {
    pub(crate) fn to_sort(&self) -> Sort {
        match self {
            SESort::SExpr => Sort::SExpr,
            SESort::ListSExpr => Sort::ListSExpr,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.to_sort().name()
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub(crate) enum SConstructor {
    SNum,
    SBoolean,
    SUnitExpr,
    SAdd,
    SSub,
    SMul,
    SLessThan,
    SAnd,
    SOr,
    SNot,
    SGet,
    SPrint,
    SRead,
    SWrite,
    SAll,
    SSwitch,
    SLoop,
    SLet,
    SArg,
    SCall,
    SCons,
    SNil,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SPurpose {
    Static(Sort), // some int, bool, order that parameterizes constructor
    SubExpr,      // subexpression, e.g. Add's summand
    SubListExpr,  // sublistexpr, e.g. Switch's branch lsit
    CapturedExpr, // a body's outputs
}

impl SPurpose {
    pub(crate) fn to_sort(self) -> Sort {
        match self {
            SPurpose::SubExpr => Sort::Expr,
            SPurpose::CapturedExpr => Sort::Expr,
            SPurpose::SubListExpr => Sort::ListExpr,
            SPurpose::Static(sort) => sort,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SField {
    pub purpose: SPurpose,
    pub name: &'static str,
}

impl SField {
    pub(crate) fn sort(&self) -> Sort {
        self.purpose.to_sort()
    }

    pub(crate) fn var(&self) -> String {
        format!("_{name}", name = self.name)
    }
}

impl SConstructor {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            SConstructor::SNum => "SNum",
            SConstructor::SBoolean => "SBoolean",
            SConstructor::SUnitExpr => "SUnitExpr",
            SConstructor::SAdd => "SAdd",
            SConstructor::SSub => "SSub",
            SConstructor::SMul => "SMul",
            SConstructor::SLessThan => "SLessThan",
            SConstructor::SAnd => "SAnd",
            SConstructor::SOr => "SOr",
            SConstructor::SNot => "SNot",
            SConstructor::SGet => "SGet",
            SConstructor::SPrint => "SPrint",
            SConstructor::SRead => "SRead",
            SConstructor::SWrite => "SWrite",
            SConstructor::SAll => "SAll",
            SConstructor::SSwitch => "SSwitch",
            SConstructor::SLoop => "SLoop",
            SConstructor::SLet => "SLet",
            SConstructor::SArg => "SArg",
            SConstructor::SCall => "SCall",
            SConstructor::SCons => "SCons",
            SConstructor::SNil => "SNil",
        }
    }

    pub(crate) fn fields(&self) -> Vec<SField> {
        use SPurpose::{CapturedExpr, Static, SubExpr, SubListExpr};
        let f = |purpose, name| SField { purpose, name };
        match self {
            SConstructor::SNum => vec![f(Static(Sort::I64), "n")],
            SConstructor::SBoolean => vec![f(Static(Sort::Bool), "b")],
            SConstructor::SUnitExpr => vec![],
            SConstructor::SAdd => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            SConstructor::SSub => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            SConstructor::SMul => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            SConstructor::SLessThan => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            SConstructor::SAnd => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            SConstructor::SOr => vec![f(SubExpr, "x"), f(SubExpr, "y")],
            SConstructor::SNot => vec![f(SubExpr, "x")],
            SConstructor::SGet => vec![f(SubExpr, "tup"), f(Static(Sort::I64), "i")],
            SConstructor::SPrint => vec![f(SubExpr, "printee")],
            SConstructor::SRead => vec![f(SubExpr, "addr")],
            SConstructor::SWrite => vec![f(SubExpr, "addr"), f(SubExpr, "data")],
            SConstructor::SAll => vec![f(Static(Sort::Order), "order"), f(SubListExpr, "exprs")],
            SConstructor::SSwitch => vec![f(SubExpr, "pred"), f(SubListExpr, "branches")],
            SConstructor::SLoop => vec![f(SubExpr, "in"), f(CapturedExpr, "pred-and-output")],
            SConstructor::SLet => vec![f(SubExpr, "in"), f(CapturedExpr, "out")],
            SConstructor::SArg => vec![],
            SConstructor::SCall => vec![f(Static(Sort::I64), "f"), f(SubExpr, "arg")],
            SConstructor::SCons => vec![f(SubExpr, "hd"), f(SubListExpr, "tl")],
            SConstructor::SNil => vec![],
        }
    }

    pub(crate) fn map_fields<F, T>(&self, f: F) -> Vec<T>
    where
        F: FnMut(&SField) -> T,
    {
        self.fields().iter().map(f).collect::<Vec<_>>()
    }

    pub(crate) fn filter_map_fields<F, T>(&self, f: F) -> Vec<T>
    where
        F: FnMut(&SField) -> Option<T>,
    {
        self.fields().iter().filter_map(f).collect::<Vec<_>>()
    }

    pub(crate) fn construct<F>(&self, f: F) -> String
    where
        F: FnMut(&SField) -> String,
    {
        let without_parens = iter::once(self.name().to_string())
            .chain(self.fields().iter().map(f))
            .collect::<Vec<_>>()
            .join(" ");
        format!("({without_parens})")
    }

    pub(crate) fn sort(&self) -> SESort {
        match self {
            SConstructor::SNum => SESort::SExpr,
            SConstructor::SBoolean => SESort::SExpr,
            SConstructor::SUnitExpr => SESort::SExpr,
            SConstructor::SAdd => SESort::SExpr,
            SConstructor::SSub => SESort::SExpr,
            SConstructor::SMul => SESort::SExpr,
            SConstructor::SLessThan => SESort::SExpr,
            SConstructor::SAnd => SESort::SExpr,
            SConstructor::SOr => SESort::SExpr,
            SConstructor::SNot => SESort::SExpr,
            SConstructor::SGet => SESort::SExpr,
            SConstructor::SPrint => SESort::SExpr,
            SConstructor::SRead => SESort::SExpr,
            SConstructor::SWrite => SESort::SExpr,
            SConstructor::SAll => SESort::SExpr,
            SConstructor::SSwitch => SESort::SExpr,
            SConstructor::SLoop => SESort::SExpr,
            SConstructor::SLet => SESort::SExpr,
            SConstructor::SArg => SESort::SExpr,
            SConstructor::SCall => SESort::SExpr,
            SConstructor::SCons => SESort::ListSExpr,
            SConstructor::SNil => SESort::ListSExpr,
        }
    }

    pub(crate) fn creates_context(&self) -> bool {
        self.fields()
            .iter()
            .any(|field| field.purpose == SPurpose::CapturedExpr)
    }
}

#[test]
fn no_duplicate_sfield_names() {
    for ctor in SConstructor::iter() {
        let mut seen: HashSet<String> = HashSet::new();
        for field in ctor.fields() {
            assert!(!seen.contains(field.name));
            seen.insert(field.name.to_string());
        }
    }
}

pub(crate) fn schema_shared() -> String {
    once("(datatype SExpr) (datatype ListSExpr)".to_string())
        .chain(SConstructor::iter().map(|ctor| {
            format!(
                "(function {name} ({field_sorts}) {sort})",
                name = ctor.name(),
                sort = ctor.sort().name(),
                field_sorts = ctor.map_fields(|field| field.sort().name()).join(" ")
            )
        }))
        .collect::<Vec<_>>()
        .join("\n")
}
