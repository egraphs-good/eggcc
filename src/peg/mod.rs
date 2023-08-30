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
        let mut memoize = HashMap::new();
        let result = rvsdg
            .result
            .map(|op| get_pegs(op, &rvsdg.nodes, &[], &mut nodes, &mut memoize));
        PegFunction {
            n_args: rvsdg.n_args,
            nodes,
            result,
        }
    }
}

fn get_pegs(
    op: Operand,
    rvsdgs: &Vec<RvsdgBody>,
    scope: &[Id],
    pegs: &mut Vec<PegBody>,
    memoize: &mut HashMap<(usize, Id), usize>,
) -> usize {
    match (op, scope.last()) {
        (Operand::Arg(arg), None) => {
            pegs.push(PegBody::Arg(arg));
            pegs.len() - 1
        }
        (Operand::Arg(arg), Some(id)) => match &rvsdgs[*id] {
            RvsdgBody::PureOp(_) => panic!("pure ops shouldn't contain regions"),
            RvsdgBody::Gamma { inputs, .. } => {
                let mut scope = scope.to_owned();
                scope.pop();
                get_pegs(inputs[arg], rvsdgs, &scope, pegs, memoize)
            }
            RvsdgBody::Theta { outputs, .. } => {
                let mut scope = scope.to_owned();
                scope.pop();
                // Layout in `pegs`: thetas, evals, pass internals, pass, theta internals
                let start_of_evals = get_pegs(Operand::Id(*id), rvsdgs, &scope, pegs, memoize);
                let start_of_thetas = start_of_evals - outputs.len();
                start_of_thetas + arg
            }
        },
        (Operand::Id(id), _) | (Operand::Project(_, id), _) => {
            let selected = match op {
                Operand::Arg(_) => unreachable!(),
                Operand::Id(_) => 0,
                Operand::Project(i, _) => i,
            };
            if let Some(out) = memoize.get(&(selected, id)) {
                return *out;
            }
            match &rvsdgs[id] {
                RvsdgBody::PureOp(expr) => {
                    let expr = match expr {
                        Expr::Op(op, xs) => Expr::Op(
                            *op,
                            xs.iter()
                                .map(|x| get_pegs(*x, rvsdgs, scope, pegs, memoize))
                                .collect(),
                        ),
                        Expr::Call(f, xs) => Expr::Call(
                            f.clone(),
                            xs.iter()
                                .map(|x| get_pegs(*x, rvsdgs, scope, pegs, memoize))
                                .collect(),
                        ),
                        Expr::Const(o, t, l) => Expr::Const(*o, t.clone(), l.clone()),
                    };
                    assert_eq!(0, selected);
                    let out = pegs.len();
                    pegs.push(PegBody::PureOp(expr));
                    memoize.insert((selected, id), out);
                    out
                }
                RvsdgBody::Gamma { pred, outputs, .. } => {
                    assert_eq!(2, outputs.len());
                    let mut scope = scope.to_owned();
                    scope.push(id);
                    let phis: Vec<PegBody> = outputs[0]
                        .iter()
                        .zip(&outputs[1])
                        .map(|(if_false, if_true)| {
                            PegBody::Phi(
                                get_pegs(*pred, rvsdgs, &scope, pegs, memoize),
                                get_pegs(*if_true, rvsdgs, &scope, pegs, memoize),
                                get_pegs(*if_false, rvsdgs, &scope, pegs, memoize),
                            )
                        })
                        .collect();
                    let out = pegs.len() + selected;
                    for i in 0..phis.len() {
                        memoize.insert((i, id), pegs.len() + i);
                    }
                    pegs.extend(phis);
                    out
                }
                RvsdgBody::Theta {
                    pred,
                    inputs,
                    outputs,
                } => {
                    // Generate a default PEG to be replaced later
                    let default = || PegBody::Arg(0);

                    // Layout in `pegs`: thetas, evals, pass internals, pass, theta internals
                    let theta_start = pegs.len();
                    pegs.extend((0..outputs.len()).map(|_| default()));
                    let evals_start = pegs.len();
                    let pass = pegs.len() + outputs.len();

                    pegs.extend(
                        (0..outputs.len()).map(|i| PegBody::Eval(theta_start + i, pass, id)),
                    );
                    pegs.push(default());

                    for i in 0..outputs.len() {
                        memoize.insert((i, id), evals_start + i);
                    }

                    let mut scope = scope.to_owned();
                    scope.push(id);

                    for (i, (output, input)) in outputs.iter().zip(inputs).enumerate() {
                        pegs[theta_start + i] = PegBody::Theta(
                            get_pegs(*input, rvsdgs, &scope, pegs, memoize),
                            get_pegs(*output, rvsdgs, &scope, pegs, memoize),
                            id,
                        );
                    }

                    pegs[pass] = PegBody::Pass(get_pegs(*pred, rvsdgs, &scope, pegs, memoize), id);

                    evals_start + selected
                }
            }
        }
    }
}
