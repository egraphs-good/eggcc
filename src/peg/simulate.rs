//! This module lets you interpret a PEG.

use crate::peg::{PegBody, PegFunction};
use crate::rvsdg::Expr;
use bril_rs::{ConstOps, Literal, ValueOps};
use std::collections::HashMap;

#[derive(Default)]
struct Indices(HashMap<usize, usize>);

impl Indices {
    fn get(&self, label: usize) -> usize {
        *self.0.get(&label).unwrap_or(&0)
    }

    fn set(&self, label: usize, value: usize) -> Indices {
        let mut out = Indices(self.0.clone());
        out.0.insert(label, value);
        out
    }
}

impl PegFunction {
    pub fn simulate(&self, args: &[Literal]) -> Option<Literal> {
        assert_eq!(self.n_args, args.len());
        self.result
            .map(|body| self.nodes[body].simulate(args, &self.nodes, &Indices::default()))
    }
}

impl PegBody {
    fn simulate(&self, args: &[Literal], nodes: &[PegBody], indices: &Indices) -> Literal {
        match self {
            PegBody::BasicOp(expr) => match expr {
                Expr::Op(op, xs, _ty) => {
                    let xs: Vec<_> = xs
                        .iter()
                        .map(|x| nodes[*x].simulate(args, nodes, indices))
                        .collect();
                    match op {
                        ValueOps::Add => Literal::Int(int(xs[0].clone()) + int(xs[1].clone())),
                        ValueOps::Mul => Literal::Int(int(xs[0].clone()) * int(xs[1].clone())),
                        ValueOps::Lt => Literal::Bool(int(xs[0].clone()) < int(xs[1].clone())),
                        op => todo!("implement {op}"),
                    }
                }
                Expr::Call(..) => panic!("can't simulate inter-function calls"),
                Expr::Print(..) => panic!("can't simulate print"),
                Expr::Const(ConstOps::Const, literal, _) => literal.clone(),
            },
            PegBody::Arg(arg) => args[*arg].clone(),
            PegBody::Phi(c, x, y) => {
                let c = nodes[*c].simulate(args, nodes, indices);
                let x = nodes[*x].simulate(args, nodes, indices);
                let y = nodes[*y].simulate(args, nodes, indices);
                if bool(c) {
                    x
                } else {
                    y
                }
            }
            PegBody::Theta(a, b, l) => {
                let c = indices.get(*l);
                if c == 0 {
                    nodes[*a].simulate(args, nodes, indices)
                } else {
                    nodes[*b].simulate(args, nodes, &indices.set(*l, c - 1))
                }
            }
            PegBody::Eval(s, i, l) => {
                let i = nodes[*i].simulate(args, nodes, indices);
                nodes[*s].simulate(args, nodes, &indices.set(*l, int(i).try_into().unwrap()))
            }
            PegBody::Pass(s, l) => {
                let mut i = 0;
                loop {
                    if !bool(nodes[*s].simulate(args, nodes, &indices.set(*l, i))) {
                        return Literal::Int(i.try_into().unwrap());
                    }
                    i += 1;
                }
            }
            PegBody::Edge(i) => nodes[*i].simulate(args, nodes, indices),
        }
    }
}

fn int(literal: Literal) -> i64 {
    match literal {
        Literal::Int(x) => x,
        _ => panic!("expected int, found {literal}"),
    }
}

fn bool(literal: Literal) -> bool {
    match literal {
        Literal::Bool(x) => x,
        // todo!: the Type shouldn't be necessary, but RVSDG gives the wrong type for
        // Annotation::AssignCond, and I couldn't figure out what entry_map was doing
        Literal::Int(x) => x != 0,
        _ => panic!("expected bool, found {literal}"),
    }
}
