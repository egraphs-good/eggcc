//! Convert RVSDGs to PEGs. This is a shortcut to avoid duplicating the work of
//! analyzing the CFG as loops and ifs, as well as making it easier to do
//! interoperation between the two dataflow representations.
//!
//! # References
//!
//! * ["Equality Saturation: A New Approach to Optimization"](https://arxiv.org/abs/1012.1802)
//! by Tate, Stepp, Tatlock, and Lerner

// todo: remove this once it no longer does anything
#![allow(dead_code)]

pub(crate) mod simulate;

use crate::rvsdg::{Expr, Id, Operand, RvsdgBody, RvsdgFunction};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// An expression, expressed using PEGs.
#[derive(Debug, PartialEq)]
pub(crate) enum PegBody {
    /// A pure operation.
    PureOp(Expr<Id>),
    /// An argument of the enclosing function.
    Arg(usize),
    /// An if statement..
    Phi(Id, Id, Id),
    /// A stream that represents a loop.
    /// The usize is a label (frequently omitted from PEG diagrams).
    Theta(Id, Id, usize),
    /// Indexes into a stream.
    Eval(Id, Id, usize),
    /// Finds the index of the first value of false in a stream.
    Pass(Id, usize),
    /// Does nothing (but useful for creating cycles).
    Edge(Id),
}

/// A function, expressed using PEGs.
#[derive(Debug, PartialEq)]
pub(crate) struct PegFunction {
    /// The number of arguments to the function.
    pub(crate) n_args: usize,
    /// The backing heap for Peg nodes within this function.
    pub(crate) nodes: Vec<PegBody>,
    /// The (optional) result pointing into this function.
    pub(crate) result: Option<Id>,
}

impl PegFunction {
    #[allow(dead_code)]
    pub fn new(rvsdg: &RvsdgFunction) -> PegFunction {
        let mut nodes = Vec::new();
        let mut builder = PegBuilder {
            rvsdgs: &rvsdg.nodes,
            pegs: &mut nodes,
            memoize: &mut HashMap::new(),
        };
        let result = rvsdg.result.map(|op| builder.get_pegs(op, &[]));
        PegFunction {
            n_args: rvsdg.n_args,
            nodes,
            result,
        }
    }
}

struct PegBuilder<'a> {
    rvsdgs: &'a Vec<RvsdgBody>,
    pegs: &'a mut Vec<PegBody>,
    memoize: &'a mut HashMap<(usize, Id), usize>,
}

#[derive(Clone)]
enum Scope<'a> {
    Gamma(&'a [Operand]),
    Theta(Vec<Id>),
}

impl PegBuilder<'_> {
    fn get_pegs(&mut self, op: Operand, scope: &[Scope]) -> usize {
        match (op, scope.last()) {
            (Operand::Arg(arg), None) => {
                self.pegs.push(PegBody::Arg(arg));
                self.pegs.len() - 1
            }
            (Operand::Arg(arg), Some(s)) => match s {
                Scope::Gamma(inputs) => {
                    let mut inner_scope = scope.to_owned();
                    inner_scope.pop();
                    self.get_pegs(inputs[arg], &inner_scope)
                }
                Scope::Theta(thetas) => thetas[arg],
            },
            (Operand::Id(id), _) | (Operand::Project(_, id), _) => {
                let selected = match op {
                    Operand::Arg(_) => unreachable!(),
                    Operand::Id(_) => 0,
                    Operand::Project(i, _) => i,
                };
                if let Some(out) = self.memoize.get(&(selected, id)) {
                    return *out;
                }
                match &self.rvsdgs[id] {
                    RvsdgBody::PureOp(expr) => {
                        let expr = match expr {
                            Expr::Op(op, xs) => {
                                Expr::Op(*op, xs.iter().map(|x| self.get_pegs(*x, scope)).collect())
                            }
                            Expr::Call(f, xs) => Expr::Call(
                                f.clone(),
                                xs.iter().map(|x| self.get_pegs(*x, scope)).collect(),
                            ),
                            Expr::Const(o, t, l) => Expr::Const(*o, t.clone(), l.clone()),
                        };
                        assert_eq!(0, selected);
                        let out = self.pegs.len();
                        self.pegs.push(PegBody::PureOp(expr));
                        self.memoize.insert((selected, id), out);
                        out
                    }
                    RvsdgBody::Gamma {
                        pred,
                        inputs,
                        outputs,
                    } => {
                        assert_eq!(2, outputs.len());
                        let mut inner_scope = scope.to_owned();
                        inner_scope.push(Scope::Gamma(inputs));
                        let phis: Vec<PegBody> = outputs[0]
                            .iter()
                            .zip(&outputs[1])
                            .map(|(if_false, if_true)| {
                                PegBody::Phi(
                                    self.get_pegs(*pred, scope),
                                    self.get_pegs(*if_true, &inner_scope),
                                    self.get_pegs(*if_false, &inner_scope),
                                )
                            })
                            .collect();
                        let out = self.pegs.len() + selected;
                        for i in 0..phis.len() {
                            self.memoize.insert((i, id), self.pegs.len() + i);
                        }
                        self.pegs.extend(phis);
                        out
                    }
                    RvsdgBody::Theta {
                        pred,
                        inputs,
                        outputs,
                    } => {
                        // Generate a default PEG to be replaced later
                        let default = || PegBody::Arg(0);

                        let theta_start = self.pegs.len();
                        self.pegs.extend((0..outputs.len()).map(|_| default()));
                        let pass = self.pegs.len();
                        self.pegs.push(default());
                        let edges_start = self.pegs.len();
                        self.pegs.extend((0..outputs.len()).map(|_| default()));

                        for i in 0..outputs.len() {
                            self.memoize.insert((i, id), edges_start + i);
                        }

                        let mut inner_scope = scope.to_owned();
                        inner_scope.push(Scope::Theta((theta_start..pass).collect()));

                        for (i, (output, input)) in outputs.iter().zip(inputs).enumerate() {
                            self.pegs[theta_start + i] = PegBody::Theta(
                                self.get_pegs(*input, scope),
                                self.get_pegs(*output, &inner_scope),
                                id,
                            );
                        }

                        self.pegs[pass] = PegBody::Pass(self.get_pegs(*pred, &inner_scope), id);

                        // We need to unroll the loop once at the end because RVSDGs are do-while
                        let evals_start = self.pegs.len();
                        self.pegs.extend(
                            (0..outputs.len()).map(|i| PegBody::Eval(theta_start + i, pass, id)),
                        );
                        let mut eval_scope = scope.to_owned();
                        eval_scope.push(Scope::Theta(
                            (evals_start..evals_start + outputs.len()).collect(),
                        ));

                        for (i, output) in outputs.iter().enumerate() {
                            // We need a new builder here because memoize already contains
                            // inside-the-loop definitions for the output nodes.
                            let mut builder = PegBuilder {
                                rvsdgs: self.rvsdgs,
                                pegs: self.pegs,
                                memoize: &mut HashMap::new(),
                            };
                            self.pegs[edges_start + i] =
                                PegBody::Edge(builder.get_pegs(*output, &eval_scope));
                        }

                        edges_start + selected
                    }
                }
            }
        }
    }
}
