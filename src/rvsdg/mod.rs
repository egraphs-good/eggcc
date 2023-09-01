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
use ordered_float::OrderedFloat;
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

impl RvsdgFunction {
    fn expr_to_egglog_expr(&self, expr: &Expr) -> egglog::ast::Expr {
        use egglog::ast::{Expr::*, Literal::*};
        let f = |operands: &Vec<Operand>| {
            operands
                .iter()
                .map(|op| self.operand_to_egglog_expr(op))
                .collect()
        };
        fn from_ty(ty: &Type) -> egglog::ast::Expr {
            match ty {
                Type::Int => Call("IntT".into(), vec![]),
                Type::Bool => Call("BoolT".into(), vec![]),
                Type::Float => Call("FloatT".into(), vec![]),
                Type::Char => Call("CharT".into(), vec![]),
                Type::Pointer(ty) => Call("PointerT".into(), vec![from_ty(ty.as_ref())]),
            }
        }
        match expr {
            Expr::Op(op, operands) => Call(op.to_string().into(), f(operands)),
            Expr::Call(ident, operands) => Call(ident.to_string().into(), f(operands)),
            Expr::Const(ConstOps::Const, ty, lit) => {
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
                    (Type::Pointer(ty), Literal::Int(p)) => {
                        Call("Ptr".into(), vec![from_ty(ty.as_ref()), Lit(Int(*p))])
                    }
                    _ => panic!("type mismatch"),
                };
                Call(
                    "Const".into(),
                    vec![Call("const".into(), vec![]), from_ty(ty), lit],
                )
            }
        }
    }

    fn body_to_egglog_expr(&self, body: &RvsdgBody) -> egglog::ast::Expr {
        use egglog::ast::Expr::*;
        match body {
            RvsdgBody::PureOp(expr) => self.expr_to_egglog_expr(expr),
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
                    Call("vec-of".into(), region.collect())
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
            Operand::Arg(p) => {
                assert!(*p < self.n_args);
                Call("Arg".into(), vec![Lit(Int(i64::try_from(*p).unwrap()))])
            }
            Operand::Id(id) => self.body_to_egglog_expr(&self.nodes[*id]),
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
        // There might be multiple results in the future,
        // e.g., one for return value and one for effect
        if let Some(result) = &self.result {
            self.operand_to_egglog_expr(result)
        } else {
            panic!("A function with no output is a noop")
        }
    }

    fn egglog_expr_to_operand(op: &egglog::ast::Expr, bodies: &mut Vec<RvsdgBody>) -> Operand {
        use egglog::ast::{Expr::*, Literal::*};
        if let Call(func, args) = op {
            match (func.as_str(), &args.as_slice()) {
                ("Arg", [Lit(Int(n))]) => Operand::Arg(*n as usize),
                ("Node", [body]) => Operand::Arg(Self::egglog_expr_to_body(body, bodies)),
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
                ("PureOp", [expr]) => RvsdgBody::PureOp(Self::egglog_expr_to_expr(expr, bodies)),
                ("Gamma", [pred, inputs, outputs]) => {
                    let pred = Self::egglog_expr_to_operand(pred, bodies);
                    let inputs = vec_map(inputs, |e| Self::egglog_expr_to_operand(e, bodies));
                    let outputs = vec_map(outputs, |es| {
                        vec_map(es, |e| Self::egglog_expr_to_operand(e, bodies))
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

    fn egglog_expr_to_expr(expr: &egglog::ast::Expr, bodies: &mut Vec<RvsdgBody>) -> Expr {
        use egglog::ast::Literal;
        if let egglog::ast::Expr::Call(func, args) = expr {
            match (func.as_str(), &args.as_slice()) {
                ("Call", [egglog::ast::Expr::Lit(Literal::String(ident)), args]) => {
                    let args = vec_map(args, |e| Self::egglog_expr_to_operand(e, bodies));
                    Expr::Call(Identifier::Name(ident.to_string()), args)
                }
                ("Const", [_const_op, ty, lit]) => Expr::Const(
                    ConstOps::Const,
                    Self::egglog_expr_to_ty(ty),
                    Self::egglog_expr_to_literal(lit),
                ),
                (binop, [opr1, opr2]) => {
                    let opr1 = Self::egglog_expr_to_operand(opr1, bodies);
                    let opr2 = Self::egglog_expr_to_operand(opr2, bodies);
                    Expr::Call(binop.into(), vec![opr1, opr2])
                }
                _ => panic!("expect an operand, got {expr}"),
            }
        } else {
            panic!("expect an operand, got {expr}")
        }
    }
    fn egglog_expr_to_ty(ty: &egglog::ast::Expr) -> Type {
        use egglog::ast::{Expr::*, Literal::*};
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
}

fn vec_map<T>(inputs: &egglog::ast::Expr, mut f: impl FnMut(&egglog::ast::Expr) -> T) -> Vec<T> {
    use egglog::ast::Expr::*;
    let mut inputs: &egglog::ast::Expr = inputs;
    let mut results = vec![];
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
