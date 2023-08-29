//! Convert RVSDGs to PEGs. This is a shortcut to avoid duplicating the work of
//! analyzing the CFG as loops and ifs, as well as making it easier to do
//! interoperation between the two dataflow representations.
//!
//! # References
//!
//! * ["Equality Saturation: A New Approach to Optimization"](https://arxiv.org/abs/1012.1802)
//! by Tate, Stepp, Tatlock, and Lerner

use crate::rvsdg::{Expr, Id};

/// An expression, expressed using PEGs.
pub(crate) enum Peg {
    /// A pure operation.
    PureOp(Expr<Id>),
    /// An argument of the enclosing function.
    Arg(usize),
    /// An if statement..
    Phi(Id, Id, Id),
    /// A stream that represents a loop.
    Theta(Id, Id),
}

/// A function, expressed using PEGs.
pub(crate) struct PegFunction {
    /// The number of arguments to the function.
    pub(crate) n_args: usize,
    /// The backing heap for Peg nodes within this function.
    pub(crate) nodes: Vec<Peg>,
    /// The (optional) result pointing into this function.
    pub(crate) result: Option<Id>,
}
