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
mod egglog_optimizer;
pub(crate) mod from_cfg;
pub(crate) mod from_egglog;
pub(crate) mod live_variables;
pub(crate) mod optimize_direct_jumps;
pub(crate) mod restructure;
pub(crate) mod rvsdg2svg;
pub(crate) mod to_cfg;
pub(crate) mod to_egglog;
mod tree_unique;

use std::fmt;

use bril_rs::{ConstOps, Literal, Type, ValueOps};
use egglog::{EGraph, SerializeConfig, TermDag};

use thiserror::Error;
use tree_optimizer::expr::TreeType;

use crate::{
    cfg::{Identifier, SimpleCfgProgram},
    util::FreshNameGen,
    EggCCError,
};

use self::{
    egglog_extensions::register_primitives,
    egglog_optimizer::{rvsdg_egglog_code, rvsdg_egglog_header_code, rvsdg_egglog_schedule},
    from_cfg::cfg_func_to_rvsdg,
};

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
    /// Following bril, we treat 'print' as a built-in primitive, rather than
    /// just another function. For the purposes of RVSDG translation, however,
    /// print is treated the same as any other function that has no ouptputs.
    /// The print edge is always passed as the last argument.
    Print(Vec<Op>),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) enum Operand {
    /// A reference to an argument in the enclosing region.
    Arg(usize),
    /// Another node in the RVSDG.
    Id(Id),
    /// Project a single output from a multi-output region.
    Project(usize, Id),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum RvsdgBody {
    BasicOp(BasicExpr<Operand>),

    /// Conditional branch, witha boolean predicate.
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RvsdgType {
    Bril(Type),
    PrintState,
}

impl RvsdgType {
    pub(crate) fn to_tree_type(&self) -> TreeType {
        match self {
            RvsdgType::Bril(t) => TreeType::Bril(t.clone()),
            RvsdgType::PrintState => TreeType::Tuple(vec![]),
        }
    }
}

/// Represents a single function as an RVSDG.
/// The function has arguments, a result, and nodes.
/// The nodes are stored in a vector, and variants of RvsdgBody refer
/// to nodes by their index in the vector.
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
pub struct RvsdgProgram {
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
    Ok(RvsdgProgram { functions })
}

impl RvsdgProgram {
    pub fn build_egglog_header_code(&self) -> String {
        rvsdg_egglog_header_code()
    }
    pub fn build_egglog_code(&self) -> (String, Vec<String>) {
        let mut fresh_names = FreshNameGen::new();
        let mut func_names = vec![];

        let mut res_string = vec![rvsdg_egglog_code()];

        for function in &self.functions {
            let name = fresh_names.fresh();
            func_names.push(name.clone());
            let expr = function.to_egglog_expr().to_sexp().pretty();
            res_string.push(format!("(let {} {})", name, expr));
        }

        res_string.push(rvsdg_egglog_schedule());

        (res_string.join("\n").to_string(), func_names)
    }

    pub fn serialized_egraph(&self) -> std::result::Result<egraph_serialize::EGraph, EggCCError> {
        let egglog_header_code = self.build_egglog_header_code();
        let mut egraph = EGraph::default();
        let _results = egraph
            .parse_and_run_program(&egglog_header_code)
            .map_err(EggCCError::EggLog);
        register_primitives(&mut egraph);
        let (egglog_code, function_names) = self.build_egglog_code();
        let _results = egraph
            .parse_and_run_program(egglog_code.as_str())
            .map_err(EggCCError::EggLog)?;

        let mut root_eclasses = vec![];
        for name in function_names {
            let (_sort, value) = egraph
                .eval_expr(&egglog::ast::Expr::Var(name.into()), None, true)
                .unwrap();
            root_eclasses.push(value);
        }

        let config = SerializeConfig {
            max_functions: None,
            max_calls_per_function: None,
            include_temporary_functions: false,
            split_primitive_outputs: true,
            root_eclasses,
        };

        Ok(egraph.serialize(config))
    }

    pub fn optimize(&self) -> std::result::Result<Self, EggCCError> {
        let egglog_header_code = self.build_egglog_header_code();
        let mut egraph = EGraph::default();
        let _results = egraph
            .parse_and_run_program(&egglog_header_code)
            .map_err(EggCCError::EggLog);
        register_primitives(&mut egraph);
        let (egglog_code, function_names) = self.build_egglog_code();
        let _results = egraph
            .parse_and_run_program(egglog_code.as_str())
            .map_err(EggCCError::EggLog)?;

        let mut functions = vec![];
        let mut termdag = TermDag::default();
        for name in function_names {
            let (sort, value) = egraph
                .eval_expr(&egglog::ast::Expr::Var(name.into()), None, true)
                .unwrap();
            let costset = egraph.extract(value, &mut termdag, &sort);
            functions.push(RvsdgFunction::egglog_term_to_function(
                costset.term,
                &termdag,
            ));
        }

        Ok(RvsdgProgram { functions })
    }
}

mod egglog_extensions {
    use std::{collections::BTreeSet, sync::Arc};

    use egglog::{
        constraint::AllEqualTypeConstraint,
        sort::{FromSort, I64Sort, IntoSort, SetSort, Sort, VecSort},
        EGraph, PrimitiveLike, Value,
    };

    pub(crate) fn register_primitives(egraph: &mut EGraph) {
        let i64: Arc<I64Sort> = egraph.get_sort().unwrap();
        let vec_i64: Arc<VecSort> = egraph
            .get_sort_by(|s: &Arc<VecSort>| s.element_name() == i64.name())
            .unwrap();
        let set_i64: Arc<SetSort> = egraph
            .get_sort_by(|s: &Arc<SetSort>| s.element_name() == i64.name())
            .unwrap();
        egraph.add_primitive(SetToVec {
            set: set_i64,
            vec: vec_i64,
        })
    }
    // Converts a set into a vec
    pub(crate) struct SetToVec {
        pub(crate) set: Arc<SetSort>,
        pub(crate) vec: Arc<VecSort>,
    }

    impl PrimitiveLike for SetToVec {
        fn name(&self) -> egglog::ast::Symbol {
            "set->vec".into()
        }

        fn get_type_constraints(&self) -> Box<dyn egglog::constraint::TypeConstraint> {
            AllEqualTypeConstraint::new(self.name())
                .with_all_arguments_sort(self.set.clone())
                .with_exact_length(2)
                .with_output_sort(self.vec.clone())
                .into_box()
            // SimpleTypeConstraint::new(self.name(), vec![self.set.clone(), self.vec.clone()]).into_box()
        }

        fn apply(
            &self,
            values: &[egglog::Value],
            _egraph: &egglog::EGraph,
        ) -> Option<egglog::Value> {
            let set = BTreeSet::<Value>::load(&self.set, &values[0]);
            let vec: Vec<Value> = set.into_iter().collect();
            vec.store(&self.vec)
        }
    }
}
