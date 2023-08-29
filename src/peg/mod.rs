//! Convert RVSDGs to PEGs. This is a shortcut to avoid duplicating the work of
//! analyzing the CFG as loops and ifs, as well as making it easier to do
//! interoperation between the two dataflow representations.
//!
//! # References
//!
//! * ["Equality Saturation: A New Approach to Optimization"](https://arxiv.org/abs/1012.1802)
//! by Tate, Stepp, Tatlock, and Lerner

use crate::rvsdg::{Expr, Id, Operand, RvsdgBody, RvsdgFunction};
use std::collections::HashMap;

/// An expression, expressed using PEGs.
pub(crate) enum PegBody {
    /// A pure operation.
    PureOp(Expr),
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
    memoize: &mut HashMap<Operand, usize>,
) -> usize {
    if let Some(out) = memoize.get(&op) {
        return *out;
    }
    match (op, scope.last()) {
        (Operand::Arg(arg), None) => {
            let out = pegs.len();
            pegs.push(PegBody::Arg(arg));
            memoize.insert(op, out);
            out
        }
        (Operand::Arg(arg), Some(id)) => match &rvsdgs[*id] {
            RvsdgBody::PureOp(_) => panic!("pure ops shouldn't contain regions"),
            RvsdgBody::Gamma { inputs, .. } => {
                let mut scope = scope.to_owned();
                scope.pop();
                get_pegs(inputs[arg], rvsdgs, &scope, pegs, memoize)
            }
            RvsdgBody::Theta { .. } => todo!(),
        },
        (Operand::Id(id), _) | (Operand::Project(_, id), _) => {
            let selected = match op {
                Operand::Arg(_) => unreachable!(),
                Operand::Id(_) => 0,
                Operand::Project(i, _) => i,
            };
            match &rvsdgs[id] {
                RvsdgBody::PureOp(expr) => {
                    assert_eq!(0, selected);
                    let out = pegs.len();
                    pegs.push(PegBody::PureOp(expr.clone()));
                    memoize.insert(op, out);
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
                        if i == 0 {
                            memoize.insert(Operand::Id(id), pegs.len());
                        }
                        memoize.insert(Operand::Project(i, id), pegs.len() + i);
                    }
                    pegs.extend(phis);
                    out
                }
                RvsdgBody::Theta { .. } => todo!(),
            }
        }
    }
}
