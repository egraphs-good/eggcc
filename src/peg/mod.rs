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

use crate::rvsdg::{Expr, Id, Operand, RvsdgBody, RvsdgFunction, RvsdgProgram};
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
    /// This is a layer of indirection for convenience (useful for creating cycles).
    /// This should not be encoded into egglog, and we should probably have a compiler
    /// pass to remove it from PegFunctions.
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
    /// A list of RVSDGs in a function.
    rvsdgs: &'a Vec<RvsdgBody>,
    /// An output parameter that is the list of PEGs for the function.
    pegs: &'a mut Vec<PegBody>,
    /// A cache of previously computed RVSDGs.
    memoize: &'a mut HashMap<(usize, Id), usize>,
}

/// A region that `get_pegs` is inside of.
#[derive(Clone)]
enum Scope<'a> {
    /// A list of the input operands to a gamma node.
    Gamma(&'a [Operand]),
    /// A list of PEG-theta nodes that are the analogs to the nested input ports.
    Theta(Vec<Id>),
}

impl PegBuilder<'_> {
    /// Get the PEG corresponding to `op`.
    fn get_pegs(&mut self, op: Operand, scope: &[Scope]) -> usize {
        match (op, scope.last()) {
            // If we aren't in a region, the arg refers to the function argument
            (Operand::Arg(arg), None) => {
                self.pegs.push(PegBody::Arg(arg));
                self.pegs.len() - 1
            }
            // If we're under a gamma, the arg refers to an input to the gamma node
            (Operand::Arg(arg), Some(Scope::Gamma(inputs))) => {
                let mut inner_scope = scope.to_owned();
                inner_scope.pop();
                self.get_pegs(inputs[arg], &inner_scope)
            }
            // If we're under a theta, the arg refers to an input to the RVSDG-theta
            // region, which corresponds to a PEG-theta node
            (Operand::Arg(arg), Some(Scope::Theta(thetas))) => thetas[arg],
            // Otherwise, we refer to a node directly
            (Operand::Id(id), _) | (Operand::Project(_, id), _) => {
                // The output port that `op` refers to
                let selected = match op {
                    Operand::Arg(_) => unreachable!(),
                    Operand::Id(_) => 0,
                    Operand::Project(i, _) => i,
                };
                // If we've already computed this node, exit early
                if let Some(out) = self.memoize.get(&(selected, id)) {
                    return *out;
                }
                match &self.rvsdgs[id] {
                    // To translate a PureOp, translate all its arguments, then change ops to ids
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
                    // To translate a Gamma, we translate the inputs lazily, and compute
                    // the predicate once (it's memoized), then each output of the Gamma
                    // becomes its own Phi node
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
                    // To translate a Theta node, we translate its implicit loops to
                    // PEG-Theta nodes, where its inputs are the initial values and
                    // its inner expressions are the loop values
                    // Then the outputs are evals with a shared pass node
                    RvsdgBody::Theta {
                        pred,
                        inputs,
                        outputs,
                    } => {
                        // Generate a default PEG to be replaced later
                        let default = || PegBody::Arg(0);

                        // Reserve slots in the list to fill in later
                        // This breaks cycles by giving us known indices
                        let theta_start = self.pegs.len();
                        self.pegs.extend((0..outputs.len()).map(|_| default()));
                        let pass = self.pegs.len();
                        self.pegs.push(default());
                        let edges_start = self.pegs.len();
                        self.pegs.extend((0..outputs.len()).map(|_| default()));

                        for i in 0..outputs.len() {
                            self.memoize.insert((i, id), edges_start + i);
                        }

                        // To compute the thetas, the args should refer to the thetas also
                        let mut inner_scope = scope.to_owned();
                        inner_scope.push(Scope::Theta((theta_start..pass).collect()));

                        for (i, (output, input)) in outputs.iter().zip(inputs).enumerate() {
                            self.pegs[theta_start + i] = PegBody::Theta(
                                self.get_pegs(*input, scope),
                                self.get_pegs(*output, &inner_scope),
                                id,
                            );
                        }

                        // The pass condition is very similar
                        self.pegs[pass] = PegBody::Pass(self.get_pegs(*pred, &inner_scope), id);

                        // We need to unroll the loop once at the end because RVSDGs are do-while
                        let evals_start = self.pegs.len();
                        self.pegs.extend(
                            (0..outputs.len()).map(|i| PegBody::Eval(theta_start + i, pass, id)),
                        );
                        let mut eval_scope = scope.to_owned();

                        // The args for the unrolling are the evals, not the thetas
                        eval_scope.push(Scope::Theta(
                            (evals_start..evals_start + outputs.len()).collect(),
                        ));

                        for (i, output) in outputs.iter().enumerate() {
                            // We need a new builder here because self.memoize already contains
                            // inside-the-loop definitions for the output nodes
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

pub struct PegProgram {
    pub(crate) functions: Vec<PegFunction>,
}

pub(crate) fn peg_to_rvsdg(
    RvsdgProgram { functions }: &RvsdgProgram,
) -> Result<PegProgram, crate::EggCCError> {
    Ok(PegProgram {
        functions: functions.iter().map(PegFunction::new).collect(),
    })
}
