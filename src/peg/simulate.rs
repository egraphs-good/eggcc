//! This module lets you interpret a PEG.

use crate::cfg::Identifier;
use crate::peg::{PegBody, PegFunction, PegProgram};
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

impl PegProgram {
    pub fn simulate(&self, args: &[Literal]) -> String {
        let mut stdout = String::new();
        let main = self.functions.iter().find(|f| f.name == "main").unwrap();
        let output = main.simulate(args, self, &mut stdout);
        assert!(output.is_none());
        stdout
    }
}

impl PegFunction {
    pub fn simulate(
        &self,
        args: &[Literal],
        program: &PegProgram,
        stdout: &mut String,
    ) -> Option<Literal> {
        assert_eq!(self.n_args, args.len());
        assert!(self.nodes[self.state]
            .simulate(args, &self.nodes, &Indices::default(), program, stdout)
            .is_none());
        self.result.and_then(|body| {
            self.nodes[body].simulate(args, &self.nodes, &Indices::default(), program, stdout)
        })
    }
}

impl PegBody {
    // Returns None if the output is a print edge
    fn simulate(
        &self,
        args: &[Literal],
        nodes: &[PegBody],
        indices: &Indices,
        program: &PegProgram,
        stdout: &mut String,
    ) -> Option<Literal> {
        match self {
            PegBody::BasicOp(expr) => match expr {
                Expr::Op(op, xs, _ty) => {
                    let xs: Vec<_> = xs
                        .iter()
                        .map(|x| nodes[*x].simulate(args, nodes, indices, program, stdout))
                        .collect();
                    match op {
                        ValueOps::Add => {
                            Some(Literal::Int(int(xs[0].clone()) + int(xs[1].clone())))
                        }
                        ValueOps::Mul => {
                            Some(Literal::Int(int(xs[0].clone()) * int(xs[1].clone())))
                        }
                        ValueOps::Div => {
                            Some(Literal::Int(int(xs[0].clone()) / int(xs[1].clone())))
                        }
                        ValueOps::Lt => {
                            Some(Literal::Bool(int(xs[0].clone()) < int(xs[1].clone())))
                        }
                        op => todo!("implement {op}"),
                    }
                }
                Expr::Call(f, xs, _, _) => {
                    let Identifier::Name(f) = f else {
                        panic!("function call identifier should be a name");
                    };
                    let xs: Vec<_> = xs
                        .iter()
                        .map(|x| nodes[*x].simulate(args, nodes, indices, program, stdout))
                        .map(Option::unwrap)
                        .collect();
                    program
                        .functions
                        .iter()
                        .find(|func| func.name == *f)
                        .unwrap()
                        .simulate(&xs, program, stdout)
                }
                Expr::Print(xs) => {
                    for x in xs {
                        let value = nodes[*x].simulate(args, nodes, indices, program, stdout);
                        // if not a print edge
                        if let Some(value) = value {
                            stdout.push_str(&format!("{}\n", value));
                        }
                    }
                    None
                }
                Expr::Const(ConstOps::Const, literal, _) => Some(literal.clone()),
            },
            PegBody::Arg(arg) => {
                if *arg == args.len() {
                    // this is a print edge
                    None
                } else {
                    Some(args[*arg].clone())
                }
            }
            PegBody::Phi(c, x, y) => {
                let c = nodes[*c].simulate(args, nodes, indices, program, stdout);
                let x = nodes[*x].simulate(args, nodes, indices, program, stdout);
                let y = nodes[*y].simulate(args, nodes, indices, program, stdout);
                if bool(c) {
                    x
                } else {
                    y
                }
            }
            PegBody::Theta(a, b, l) => {
                let c = indices.get(*l);
                if c == 0 {
                    nodes[*a].simulate(args, nodes, indices, program, stdout)
                } else {
                    nodes[*b].simulate(args, nodes, &indices.set(*l, c - 1), program, stdout)
                }
            }
            PegBody::Eval(s, i, l) => {
                let i = nodes[*i].simulate(args, nodes, indices, program, stdout);
                nodes[*s].simulate(
                    args,
                    nodes,
                    &indices.set(*l, int(i).try_into().unwrap()),
                    program,
                    stdout,
                )
            }
            PegBody::Pass(s, l) => {
                let mut i = 0;
                loop {
                    if !bool(nodes[*s].simulate(args, nodes, &indices.set(*l, i), program, stdout))
                    {
                        return Some(Literal::Int(i.try_into().unwrap()));
                    }
                    i += 1;
                }
            }
            PegBody::Edge(i) => nodes[*i].simulate(args, nodes, indices, program, stdout),
        }
    }
}

fn int(literal: Option<Literal>) -> i64 {
    match literal.unwrap() {
        Literal::Int(x) => x,
        literal => panic!("expected int, found {literal}"),
    }
}

fn bool(literal: Option<Literal>) -> bool {
    match literal.unwrap() {
        Literal::Bool(x) => x,
        // todo!: the Type shouldn't be necessary, but RVSDG gives the wrong type for
        // Annotation::AssignCond, and I couldn't figure out what entry_map was doing
        Literal::Int(x) => x != 0,
        literal => panic!("expected bool, found {literal}"),
    }
}
