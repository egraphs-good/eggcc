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
pub(crate) mod from_dag;
pub(crate) mod live_variables;
pub(crate) mod optimize_direct_jumps;
mod passthrough;
pub(crate) mod restructure;
pub(crate) mod rvsdg2svg;
pub(crate) mod simplify_branches;
pub(crate) mod to_cfg;
mod to_dag;

use std::fmt;

use bril_rs::{ConstOps, EffectOps, Literal, Type, ValueOps};

use dag_in_context::schema::BaseType;
use hashbrown::{HashMap, HashSet};
use thiserror::Error;

use crate::{
    cfg::{Identifier, SimpleCfgProgram},
    EggCCError,
};

use self::from_cfg::cfg_func_to_rvsdg;

#[cfg(test)]
pub(crate) mod tests;

/// Errors from the rvsdg module.
#[derive(Debug, Error)]
pub enum RvsdgError {
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

    // NB: We should be able to support these patterns, but it might be better
    // to desugar them away as part of the CFG parsing step.
    #[error("Multiple branches from loop tail to head ({pos:?})")]
    UnsupportedLoopTail { pos: Option<bril_rs::Position> },
}

pub(crate) type Result<T = ()> = std::result::Result<T, RvsdgError>;

pub(crate) type Id = usize;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum BasicExpr<Op> {
    /// A primitive operation.
    Op(ValueOps, Vec<Op>, Type),
    /// A function call. In general, there are two kinds of parameters
    /// to a function: value arguments, which have a Bril type, and state arguments,
    /// which includes PrintEdge.
    ///
    /// The `usize` fields denotes the number of return values this function call
    /// will produce. Among them at most one is a non-state value, which, if exists, is the
    /// first return value. The SVG rendering code also relies on this value to
    /// determine how many output ports to add to a function call.
    Call(String, Vec<Op>, usize, Option<Type>),
    /// A literal constant.
    Const(ConstOps, Literal, Type),
    /// A bril effect. These are a lot like an `Op`, but they only produce a
    /// "state edge" as output.
    ///
    /// Note: the only bril effects that can show up are print and
    /// memory-related (Print, Store, Free). Other effects (e.g. control flow)
    /// are handled separately.
    /// The state edge for these operators is the last input and last output.
    Effect(EffectOps, Vec<Op>),
}

impl<Op> BasicExpr<Op> {
    pub(crate) fn num_outputs(&self) -> usize {
        match self {
            BasicExpr::Op(ValueOps::Alloc | ValueOps::Load, _, _) => 2,
            BasicExpr::Op(_, _, _) => 1,
            BasicExpr::Call(_, _, n_outputs, _) => *n_outputs,
            BasicExpr::Const(_, _, _) => 1,
            BasicExpr::Effect(_, _) => 1,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn push_operand(&mut self, op: Op) {
        match self {
            BasicExpr::Op(_, operands, _) => operands.push(op),
            BasicExpr::Call(_, operands, _, _) => operands.push(op),
            BasicExpr::Const(_, _, _) => panic!("Cannot push operand to const"),
            BasicExpr::Effect(_, operands) => operands.push(op),
        }
    }

    pub(crate) fn map_operands(&mut self, f: &mut impl FnMut(&Op) -> Op) {
        match self {
            BasicExpr::Op(_, operands, _) => {
                for operand in operands {
                    *operand = f(operand);
                }
            }
            BasicExpr::Call(_, operands, _, _) => {
                for operand in operands {
                    *operand = f(operand);
                }
            }
            BasicExpr::Const(_, _, _) => {}
            BasicExpr::Effect(_, operands) => {
                for operand in operands {
                    *operand = f(operand);
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) enum Operand {
    /// A reference to an argument in the enclosing region.
    Arg(usize),
    /// Project a single output from a region in the RVSDG.
    Project(usize, Id),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum RvsdgBody {
    BasicOp(BasicExpr<Operand>),

    /// Conditional branch, with a boolean predicate.
    If {
        pred: Operand,
        inputs: Vec<Operand>,
        /// invariant: then_branch and else_branch have same length
        then_branch: Vec<Operand>,
        else_branch: Vec<Operand>,
    },

    /// Conditional branch, where the outputs chosen depend on the predicate.
    Gamma {
        pred: Operand,
        inputs: Vec<Operand>,
        /// invariant: all of the vecs in output have
        /// the same length.
        outputs: Vec<Vec<Operand>>,
    },

    /// A tail-controlled loop.
    Theta {
        pred: Operand,
        inputs: Vec<Operand>,
        outputs: Vec<Operand>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum RvsdgType {
    Bril(Type),
    PrintState,
}

fn type_to_treetype_base(ty: &Type) -> BaseType {
    match ty {
        Type::Int => BaseType::IntT,
        Type::Bool => BaseType::BoolT,
        Type::Float => BaseType::FloatT,
        Type::Char => todo!("Chars not supported yet"),
        Type::Pointer(inner) => BaseType::PointerT(Box::new(type_to_treetype_base(inner))),
    }
}

impl RvsdgType {
    /// Converts a bril type to a tree type.
    /// If the type is a print state, returns None.
    pub(crate) fn to_tree_type(&self) -> Option<BaseType> {
        match self {
            RvsdgType::Bril(ty) => Some(type_to_treetype_base(ty)),
            RvsdgType::PrintState => Some(BaseType::StateT),
        }
    }
}

/// Represents a single function as an RVSDG.
/// The function has arguments, a result, and nodes.
/// The nodes are stored in a vector, and variants of RvsdgBody refer
/// to nodes by their index in the vector.
#[derive(Clone)]
pub struct RvsdgFunction {
    /// The name of this function.
    pub(crate) name: String,

    /// The arguments to this function, which can be bril values or
    /// state edges.
    pub(crate) args: Vec<RvsdgType>,
    /// The backing heap for Rvsdg node ids within this function.
    /// Invariant: nodes refer only to nodes with a lower index.
    pub(crate) nodes: Vec<RvsdgBody>,

    /// A list of results pointing into this function.
    pub(crate) results: Vec<(RvsdgType, Operand)>,
}

impl fmt::Debug for RvsdgFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RvsdgFunction")
            .field("args", &self.args)
            .field("result", &self.results);
        let mut map = f.debug_map();
        for (i, node) in self.nodes.iter().enumerate() {
            map.entry(&i, node);
        }
        map.finish()?;
        write!(f, "}}")
    }
}

/// A Bril program represented as an Rvsdg.
/// For now, it's simply a vector of [RvsdgFunction]s.
/// In the future, we may want functions to be represented within
/// the RVSDG.
#[derive(Clone)]
pub struct RvsdgProgram {
    /// A list of functions in this program.
    /// The last function is the entry point (main function).
    pub(crate) functions: Vec<RvsdgFunction>,
}

pub(crate) fn cfg_to_rvsdg(
    cfg: &SimpleCfgProgram,
) -> std::result::Result<RvsdgProgram, EggCCError> {
    // Rvsdg translation also restructured the cfg
    // so make a copy for that.
    let mut cfg_restructured = cfg.clone().into_switch();
    let func_types = cfg_restructured.function_types();

    let mut functions = vec![];
    for func in cfg_restructured.functions.iter_mut() {
        functions.push(cfg_func_to_rvsdg(func, &func_types).map_err(EggCCError::RvsdgError)?);
    }
    let mut prog = RvsdgProgram { functions };

    // now run passthrough optimization to clean it up
    prog = prog.optimize_passthrough();
    Ok(prog)
}

impl RvsdgBody {
    pub(crate) fn map_operands(&mut self, f: &mut impl FnMut(&Operand) -> Operand) {
        match self {
            RvsdgBody::BasicOp(basic_expr) => {
                basic_expr.map_operands(f);
            }
            RvsdgBody::If {
                pred,
                inputs,
                then_branch,
                else_branch,
            } => {
                *pred = f(pred);
                for input in inputs {
                    *input = f(input);
                }
                for branch in then_branch {
                    *branch = f(branch);
                }
                for branch in else_branch {
                    *branch = f(branch);
                }
            }
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => {
                *pred = f(pred);
                for input in inputs {
                    *input = f(input);
                }
                for output in outputs.iter_mut().flatten() {
                    *output = f(output);
                }
            }
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            } => {
                *pred = f(pred);
                for input in inputs {
                    *input = f(input);
                }
                for output in outputs {
                    *output = f(output);
                }
            }
        }
    }

    pub(crate) fn map_same_region_operands(
        mut self,
        fun: &mut impl FnMut(&Operand) -> Operand,
    ) -> Self {
        match &mut self {
            RvsdgBody::BasicOp(basic_expr) => {
                basic_expr.map_operands(fun);
            }
            RvsdgBody::If {
                pred,
                inputs,
                then_branch: _,
                else_branch: _,
            } => {
                *pred = fun(pred);
                for input in inputs {
                    *input = fun(input);
                }
            }
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs: _,
            } => {
                *pred = fun(pred);
                for input in inputs {
                    *input = fun(input);
                }
            }
            RvsdgBody::Theta {
                pred: _,
                inputs,
                outputs: _,
            } => {
                for input in inputs {
                    *input = fun(input);
                }
            }
        }
        self
    }

    pub(crate) fn num_outputs(&self) -> usize {
        match self {
            RvsdgBody::BasicOp(basic_expr) => basic_expr.num_outputs(),
            RvsdgBody::If { then_branch, .. } => then_branch.len(),
            RvsdgBody::Gamma { outputs, .. } => outputs[0].len(),
            RvsdgBody::Theta { outputs, .. } => outputs.len(),
        }
    }
}

impl RvsdgFunction {
    pub(crate) fn uses_analysis(&self) -> HashMap<Id, HashSet<Id>> {
        let mut uses = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            let mut used = HashSet::new();
            node.clone().map_operands(&mut |op| {
                if let Operand::Project(_, region) = op {
                    used.insert(*region);
                }
                *op
            });
            for use_id in used {
                uses.entry(use_id).or_insert_with(HashSet::new).insert(i);
            }
        }
        uses
    }
}
