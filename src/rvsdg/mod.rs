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
use egglog::{ast::Symbol, EGraph};
use ordered_float::OrderedFloat;
use thiserror::Error;

use crate::{
    cfg::{CfgProgram, Identifier},
    conversions::egglog_op_to_bril,
    EggCCError,
};

use self::from_cfg::cfg_func_to_rvsdg;

#[cfg(test)]
mod tests;

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

#[derive(Debug, PartialEq)]
pub(crate) enum Expr<Op> {
    /// A primitive operation.
    Op(ValueOps, Vec<Op>, Type),
    /// A function call. The last parameter is the number of outputs to the
    /// function. Functions always have an "extra" output that is used for the
    /// 'state edge' flowing out of the function.
    ///
    /// Essentially all of the code here does not use this value at all. The
    /// exception is the SVG rendering code, which relies on this value to
    /// determine how many output ports to add to a function call.
    Call(Identifier, Vec<Op>, usize, Option<Type>),
    /// A literal constant.
    Const(ConstOps, Literal, Type),
    /// Following bril, we treat 'print' as a built-in primitive, rather than
    /// just another function. For the purposes of RVSDG translation, however,
    /// print is treated the same as any other function that has no ouptputs.
    /// The print edge is always passed as the last argument.
    Print(Vec<Op>),
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
    BasicOp(Expr<Operand>),

    /// Conditional branch, where the outputs chosen depend on the predicate.
    Gamma {
        /// always has type bool
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

#[derive(Debug)]
pub(crate) enum RvsdgType {
    Bril(Type),
    PrintState,
}

/// Represents a single function as an RVSDG.
/// The function has arguments, a result, and nodes.
/// The nodes are stored in a vector, and variants of RvsdgBody refer
/// to nodes by their index in the vector.
pub struct RvsdgFunction {
    /// The name of this function.
    pub(crate) name: String,

    /// The number of input arguments to the function.
    ///
    /// Functions all take `n_args + 1` arguments, where the last argument is a
    /// "state edge" used to preserve ordering constraints to (potentially)
    /// impure function calls.
    /// TODO remove n_args when the egglog encoding supports args
    pub(crate) n_args: usize,
    /// The arguments to this function, which can be bril values or
    /// state edges.
    pub(crate) args: Vec<RvsdgType>,
    /// The backing heap for Rvsdg node ids within this function.
    pub(crate) nodes: Vec<RvsdgBody>,
    /// The (optional) result pointing into this function.
    ///
    /// NB: until effects are supported, the only way to ensure a computation is
    /// marked as used is to populate a result of some kind.
    pub(crate) result: Option<Operand>,

    /// The output port corersponding to the state edge of the function.
    pub(crate) state: Operand,
}

impl fmt::Debug for RvsdgFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RvsdgFunction")
            .field("args", &self.args)
            .field("result", &self.result)
            .field("state", &self.state);
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

pub(crate) fn cfg_to_rvsdg(cfg: &CfgProgram) -> std::result::Result<RvsdgProgram, EggCCError> {
    // Rvsdg translation also restructured the cfg
    // so make a copy for that.
    let mut cfg_restructured = cfg.clone();
    let func_types = cfg_restructured.function_types();

    let mut functions = vec![];
    for func in cfg_restructured.functions.iter_mut() {
        functions.push(cfg_func_to_rvsdg(func, &func_types).map_err(EggCCError::RvsdgError)?);
    }
    Ok(RvsdgProgram { functions })
}

impl RvsdgFunction {
    fn expr_from_ty(ty: &Type) -> egglog::ast::Expr {
        use egglog::ast::Expr::*;
        match ty {
            Type::Int => Call("IntT".into(), vec![]),
            Type::Bool => Call("BoolT".into(), vec![]),
            Type::Float => Call("FloatT".into(), vec![]),
            Type::Char => Call("CharT".into(), vec![]),
            Type::Pointer(ty) => Call("PointerT".into(), vec![Self::expr_from_ty(ty.as_ref())]),
        }
    }

    fn expr_from_rvsdg_ty(ty: &RvsdgType) -> egglog::ast::Expr {
        use egglog::ast::Expr::*;
        match ty {
            RvsdgType::Bril(ty) => Call("Bril".into(), vec![RvsdgFunction::expr_from_ty(ty)]),
            RvsdgType::PrintState => Call("PrintState".into(), vec![]),
        }
    }

    fn expr_to_egglog_expr(&self, expr: &Expr<Operand>) -> egglog::ast::Expr {
        use egglog::ast::{Expr::*, Literal::*};
        let f = |operands: &Vec<Operand>, ty: Option<Type>| {
            let mut res = Vec::with_capacity(operands.len() + ty.is_some() as usize);
            if let Some(ty) = ty {
                res.push(Self::expr_from_ty(&ty));
            }
            res.extend(operands.iter().map(|op| self.operand_to_egglog_expr(op)));
            res
        };

        match expr {
            Expr::Op(op, operands, ty) => {
                Call(op.to_string().into(), f(operands, Some(ty.clone())))
            }
            // TODO I'm pretty sure this conversion isn't right
            Expr::Call(ident, operands, _, ty) => {
                Call(ident.to_string().into(), f(operands, ty.clone()))
            }
            Expr::Print(operands) => Call("PRINT".into(), f(operands, None)),
            Expr::Const(ConstOps::Const, lit, ty) => {
                let lit = match (ty, lit) {
                    (Type::Int, Literal::Int(n)) => Call("Num".into(), vec![Lit(Int(*n))]),
                    (Type::Bool, Literal::Bool(b)) => {
                        Call("Bool".into(), vec![Lit(Int(*b as i64))])
                    }
                    (Type::Float, Literal::Float(f)) => Call(
                        "Float".into(),
                        vec![Lit(F64(OrderedFloat::<f64>::from(*f)))],
                    ),
                    (Type::Char, Literal::Char(c)) => {
                        Call("Char".into(), vec![Lit(String(c.to_string().into()))])
                    }
                    (Type::Pointer(ty), Literal::Int(p)) => Call(
                        "Ptr".into(),
                        vec![Self::expr_from_ty(ty.as_ref()), Lit(Int(*p))],
                    ),
                    _ => panic!("type mismatch"),
                };
                Call(
                    "Const".into(),
                    vec![Self::expr_from_ty(ty), Call("const".into(), vec![]), lit],
                )
            }
        }
    }

    fn body_to_egglog_expr(&self, body: &RvsdgBody) -> egglog::ast::Expr {
        use egglog::ast::Expr::*;
        match body {
            RvsdgBody::BasicOp(expr) => Call("PureOp".into(), vec![self.expr_to_egglog_expr(expr)]),
            RvsdgBody::Gamma {
                pred,
                inputs,
                outputs,
            } => {
                let pred = self.operand_to_egglog_expr(pred);
                let inputs = inputs
                    .iter()
                    .map(|input| self.operand_to_egglog_expr(input));
                let inputs = Call("vec-of".into(), inputs.collect());
                let outputs = outputs.iter().map(|region| {
                    let region = region
                        .iter()
                        .map(|output| self.operand_to_egglog_expr(output));
                    Call("VO".into(), vec![Call("vec-of".into(), region.collect())])
                });
                let outputs = Call("vec-of".into(), outputs.collect());
                Call("Gamma".into(), vec![pred, inputs, outputs])
            }
            RvsdgBody::Theta {
                pred,
                inputs,
                outputs,
            } => {
                let pred = self.operand_to_egglog_expr(pred);
                let inputs = inputs
                    .iter()
                    .map(|input| self.operand_to_egglog_expr(input));
                let inputs = Call("vec-of".into(), inputs.collect());
                let outputs = outputs
                    .iter()
                    .map(|output| self.operand_to_egglog_expr(output));
                let outputs = Call("vec-of".into(), outputs.collect());
                Call("Theta".into(), vec![pred, inputs, outputs])
            }
        }
    }

    fn operand_to_egglog_expr(&self, op: &Operand) -> egglog::ast::Expr {
        use egglog::ast::{Expr::*, Literal::*};
        match op {
            Operand::Arg(p) => Call("Arg".into(), vec![Lit(Int(i64::try_from(*p).unwrap()))]),
            Operand::Id(id) => Call(
                "Node".into(),
                vec![self.body_to_egglog_expr(&self.nodes[*id])],
            ),
            Operand::Project(i, id) => {
                let body = self.body_to_egglog_expr(&self.nodes[*id]);
                Call(
                    "Project".into(),
                    vec![Lit(Int(i64::try_from(*i).unwrap())), body],
                )
            }
        }
    }

    pub fn to_egglog_expr(&self) -> egglog::ast::Expr {
        use egglog::ast::{Expr::*, Literal::*};
        let name: Symbol = self.name.clone().into();

        let sig = Call(
            "vec-of".into(),
            self.args
                .iter()
                .map(RvsdgFunction::expr_from_rvsdg_ty)
                .collect(),
        );
        let output = {
            let state = self.operand_to_egglog_expr(&self.state);
            if let Some(result) = &self.result {
                let value = self.operand_to_egglog_expr(result);
                Call("StateAndValue".into(), vec![state, value])
            } else {
                Call("StateOnly".into(), vec![state])
            }
        };
        Call("Func".into(), vec![Lit(String(name)), sig, output])
    }

    fn egglog_expr_to_operand(op: &egglog::ast::Expr, bodies: &mut Vec<RvsdgBody>) -> Operand {
        use egglog::ast::{Expr::*, Literal::*};
        if let Call(func, args) = op {
            match (func.as_str(), &args.as_slice()) {
                ("Arg", [Lit(Int(n))]) => Operand::Arg(*n as usize),
                ("Node", [body]) => Operand::Id(Self::egglog_expr_to_body(body, bodies)),
                ("Project", [Lit(Int(n)), body]) => {
                    Operand::Project(*n as usize, Self::egglog_expr_to_body(body, bodies))
                }
                _ => panic!("expect an operand, got {op}"),
            }
        } else {
            panic!("expect an operand, got {op}")
        }
    }

    fn egglog_expr_to_body(body: &egglog::ast::Expr, bodies: &mut Vec<RvsdgBody>) -> Id {
        use egglog::ast::Expr::*;
        if let Call(func, args) = body {
            let body = match (func.as_str(), &args.as_slice()) {
                ("PureOp", [expr]) => RvsdgBody::BasicOp(Self::egglog_expr_to_expr(expr, bodies)),
                ("Gamma", [pred, inputs, outputs]) => {
                    let pred = Self::egglog_expr_to_operand(pred, bodies);
                    let inputs = vec_map(inputs, |e| Self::egglog_expr_to_operand(e, bodies));
                    let outputs = vec_map(outputs, |es| {
                        if let Call(func, args) = es {
                            assert_eq!(func.as_str(), "VO");
                            assert_eq!(args.len(), 1);
                            let es = &args[0];
                            vec_map(es, |e| Self::egglog_expr_to_operand(e, bodies))
                        } else {
                            panic!("expect VecOperandWrapper")
                        }
                    });
                    RvsdgBody::Gamma {
                        pred,
                        inputs,
                        outputs,
                    }
                }
                ("Theta", [pred, inputs, outputs]) => {
                    let pred = Self::egglog_expr_to_operand(pred, bodies);
                    let inputs = vec_map(inputs, |e| Self::egglog_expr_to_operand(e, bodies));
                    let outputs = vec_map(outputs, |e| Self::egglog_expr_to_operand(e, bodies));
                    RvsdgBody::Theta {
                        pred,
                        inputs,
                        outputs,
                    }
                }
                _ => panic!("expect an operand, got {body}"),
            };
            bodies.push(body);
            bodies.len() - 1
        } else {
            panic!("expect an operand, got {body}")
        }
    }

    fn egglog_expr_to_expr(expr: &egglog::ast::Expr, bodies: &mut Vec<RvsdgBody>) -> Expr<Operand> {
        use egglog::ast::Literal;
        if let egglog::ast::Expr::Call(func, args) = expr {
            match (func.as_str(), &args.as_slice()) {
                ("Call", [ty, egglog::ast::Expr::Lit(Literal::String(ident)), args]) => {
                    let args = vec_map(args, |e| Self::egglog_expr_to_operand(e, bodies));
                    // TODO: this is imprecise, we don't know if the number of outputs is 1 or 2.
                    Expr::Call(
                        Identifier::Name(ident.to_string()),
                        args,
                        1,
                        Some(Self::egglog_expr_to_ty(ty)),
                    )
                }
                ("Const", [ty, _const_op, lit]) => Expr::Const(
                    // todo remove the const op from the encoding because it is always ConstOps::Const
                    ConstOps::Const,
                    Self::egglog_expr_to_literal(lit),
                    Self::egglog_expr_to_ty(ty),
                ),
                (binop, [ty, opr1, opr2]) => {
                    let opr1 = Self::egglog_expr_to_operand(opr1, bodies);
                    let opr2 = Self::egglog_expr_to_operand(opr2, bodies);
                    Expr::Op(
                        egglog_op_to_bril(binop.into()),
                        vec![opr1, opr2],
                        Self::egglog_expr_to_ty(ty),
                    )
                }
                _ => panic!("expect an operand, got {expr}"),
            }
        } else {
            panic!("expect an operand, got {expr}")
        }
    }

    fn egglog_expr_to_ty(ty: &egglog::ast::Expr) -> Type {
        use egglog::ast::Expr::*;
        if let Call(func, args) = ty {
            match (func.as_str(), &args.as_slice()) {
                ("IntT", []) => Type::Int,
                ("BoolT", []) => Type::Bool,
                ("FloatT", []) => Type::Float,
                ("CharT", []) => Type::Char,
                ("PointerT", [inner]) => Type::Pointer(Box::new(Self::egglog_expr_to_ty(inner))),
                _ => panic!("expect a list, got {ty}"),
            }
        } else {
            panic!("expect a list, got {ty}")
        }
    }

    fn egglog_expr_to_rvsdg_ty(ty: &egglog::ast::Expr) -> RvsdgType {
        use egglog::ast::Expr::*;
        if let Call(func, args) = ty {
            match (func.as_str(), &args.as_slice()) {
                ("PrintState", []) => RvsdgType::PrintState,
                ("Bril", [ty]) => RvsdgType::Bril(Self::egglog_expr_to_ty(ty)),
                _ => panic!("expect an expression, got {ty}"),
            }
        } else {
            panic!("expect an expression, got {ty}")
        }
    }

    fn egglog_expr_to_literal(lit: &egglog::ast::Expr) -> Literal {
        use egglog::ast::{Expr::*, Literal::*};
        if let Call(func, args) = lit {
            match (func.as_str(), &args.as_slice()) {
                ("Num", [Lit(Int(n))]) => Literal::Int(*n),
                ("Float", [Lit(F64(n))]) => Literal::Float(f64::from(*n)),
                ("Char", [Lit(String(s))]) => {
                    assert_eq!(s.as_str().len(), 1);
                    Literal::Char(s.as_str().chars().next().unwrap())
                }
                _ => panic!("expect a list, got {lit}"),
            }
        } else {
            panic!("expect a list, got {lit}")
        }
    }

    pub fn egglog_expr_to_function(expr: &egglog::ast::Expr) -> RvsdgFunction {
        use egglog::ast::{Expr::*, Literal::*};
        if let Call(func, args) = expr {
            match (func.as_str(), &args.as_slice()) {
                ("Func", [Lit(String(name)), sig, Call(func_output, func_args)]) => {
                    let args: Vec<RvsdgType> = vec_map(sig, Self::egglog_expr_to_rvsdg_ty);
                    let n_args = args.len() - 1;

                    let mut nodes = vec![];
                    let (state, result) = match (func_output.as_str(), &func_args.as_slice()) {
                        ("StateOnly", [state]) => {
                            (Self::egglog_expr_to_operand(state, &mut nodes), None)
                        }
                        ("StateAndValue", [state, result]) => {
                            let state = Self::egglog_expr_to_operand(state, &mut nodes);
                            let result = Self::egglog_expr_to_operand(result, &mut nodes);
                            (state, Some(result))
                        }
                        _ => panic!("expect a function, got {expr}"),
                    };
                    RvsdgFunction {
                        name: name.to_string(),
                        n_args,
                        args,
                        nodes,
                        result,
                        state,
                    }
                }
                _ => panic!("expect a function, got {expr}"),
            }
        } else {
            panic!("expect a function, got {expr}")
        }
    }
}

fn vec_map<T>(inputs: &egglog::ast::Expr, mut f: impl FnMut(&egglog::ast::Expr) -> T) -> Vec<T> {
    use egglog::ast::Expr::*;
    let mut inputs: &egglog::ast::Expr = inputs;
    let mut results = vec![];
    if let Call(func, args) = inputs {
        if func.as_str() == "vec-of" {
            return args.iter().map(&mut f).collect();
        }
    }
    loop {
        if let Call(func, args) = inputs {
            match (func.as_str(), &args.as_slice()) {
                ("vec-push", [head, tail]) => {
                    results.push(f(head));
                    inputs = tail;
                }
                ("vec-empty", []) => {
                    break;
                }
                _ => panic!("expect a list, got {inputs}"),
            }
        } else {
            panic!("expect a list, got {inputs}")
        }
    }
    results.reverse();
    results
}

pub fn new_rvsdg_egraph() -> EGraph {
    let mut egraph = EGraph::default();
    let schema = std::fs::read_to_string("src/rvsdg/schema.egg").unwrap();
    egraph.parse_and_run_program(schema.as_str()).unwrap();
    egraph
}
