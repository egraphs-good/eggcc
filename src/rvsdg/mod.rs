#![allow(dead_code)] // TODO: remove this once wired in
//! Convert bril programs to RVSDGs.
//!
//! Bril functions are written in terms of basic blocks and jumps/gotos. RVSDGs
//! only support intra-function control flow in the form of switch statements
//! and do-while loops (gamma and theta nodes, respectively). Transforming the
//! native Bril representation to RVSDGs requires the following steps:
//!
//! * Parse to CFG: read the bril program into a graph data-structure where
//! basic blocks are nodes and edges are jumps. (This happens in the `cfg`
//! module).
//!
//! * Restructure the CFG: Bril programs support irreducible CFGs, but the CFGs
//! corresponding to RVSDGs are all reducible. Before we convert the CFG to an
//! RVSDG, we must convert the unstructured CFG to a structured one.
//!
//! * RVSDG conversion: Once we have a structured CFG we can convert the
//! program (still written in terms of gotos) to the structured format for
//! RVSDGs. Part of this conversion process is the discovery of what the
//! "inputs" and "outputs" are for different RVSDG nodes; the main subroutine we
//! use there is a live variable analysis.
//!
//! # References
//!
//! * ["RVSDG: An Intermediate Representation for Optimizing Compilers"](https://arxiv.org/abs/1912.05036)
//! by Reissmann, Meyer, Bahmann, and Själander
//! * ["Perfect Reconstructability of Control Flow from Demand Dependence Graphs"](https://dl.acm.org/doi/10.1145/2693261)
//! by Bahmann, Reissmann,  Jahre, and Meyer
//!
//! In addition to those papers, the Jamey Sharp's
//! [optir](https://github.com/jameysharp/optir) project is a major inspiration.
pub(crate) mod from_cfg;
pub(crate) mod live_variables;
pub(crate) mod restructure;
pub(crate) mod rvsdg2svg;

use std::fmt;

use bril_rs::{ConstOps, Literal, Type, ValueOps};
use thiserror::Error;

use crate::cfg::Identifier;

#[cfg(test)]
mod tests;

/// Errors from the rvsdg module.
#[derive(Debug, Error)]
pub(crate) enum RvsdgError {
    #[error("Unsupported operation: {op:?}, {pos:?}")]
    UnsupportedOperation {
        op: bril_rs::ValueOps,
        pos: Option<bril_rs::Position>,
    },

    #[error("Unsupported effect: {op:?}, {pos:?}")]
    UnsupportedEffect {
        op: bril_rs::EffectOps,
        pos: Option<bril_rs::Position>,
    },

    #[error("Scope error: undefined id {id:?}, {pos:?}")]
    UndefinedId {
        id: Identifier,
        pos: Option<bril_rs::Position>,
    },

    // NB: We should  be able to suppor these patterns, but it might be better
    // to desugar them away as part of the CFG parsing step.
    #[error("Multiple branches from loop tail to head ({pos:?})")]
    UnsupportedLoopTail { pos: Option<bril_rs::Position> },
}

pub(crate) type Result<T = ()> = std::result::Result<T, RvsdgError>;

#[derive(Debug)]
pub(crate) enum Annotation {
    AssignCond { dst: Identifier, cond: u32 },
    AssignRet { src: Identifier },
}

pub(crate) type Id = usize;

#[derive(Debug)]
pub(crate) enum Expr {
    Op(ValueOps, Vec<Operand>),
    Call(Identifier, Vec<Operand>),
    Const(ConstOps, Type, Literal),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Operand {
    /// A reference to an argument in the enclosing region.
    Arg(usize),
    /// Another node in the RVSDG.
    Id(Id),
    /// Project a single output from a multi-output region.
    Project(usize, Id),
}

#[derive(Debug)]
pub(crate) enum RvsdgBody {
    PureOp(Expr),
    Gamma {
        pred: Operand,
        inputs: Vec<Operand>,
        outputs: Vec<Vec<Operand>>,
    },
    Theta {
        pred: Operand,
        inputs: Vec<Operand>,
        outputs: Vec<Operand>,
    },
}

pub(crate) struct RvsdgFunction {
    /// The number of input arguments to the function.
    pub(crate) n_args: usize,
    /// The backing heap for Rvsdg node ids within this function.
    pub(crate) nodes: Vec<RvsdgBody>,
    /// The (optional) result pointing into this function.
    ///
    /// NB: until effects are supported, the only way to ensure a computation is
    /// marked as used is to populate a result of some kind.
    pub(crate) result: Option<Operand>,
}

impl fmt::Debug for RvsdgFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RvsdgFunction")
            .field("n_args", &self.n_args)
            .field("result", &self.result);
        let mut map = f.debug_map();
        for (i, node) in self.nodes.iter().enumerate() {
            map.entry(&i, node);
        }
        map.finish()?;
        write!(f, "}}")
    }
}
