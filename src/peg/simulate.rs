//! This module lets you interpret a PEG.

use std::rc::Rc;
use std::cell::RefCell;
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
        let main = self.functions.iter().find(|f| f.name == "main").unwrap();
        let mut s = Simulator {
            args,
            nodes: &main.nodes,
            indices: &Indices::default(),
            program: self,
            stdout: Rc::new(RefCell::new(String::new())),
        };
        let output = main.simulate(&mut s);
        assert!(output.is_none());
        Rc::try_unwrap(s.stdout).unwrap().into_inner()
    }
}

impl PegFunction {
    fn simulate(
        &self,
        s: &mut Simulator,
    ) -> Option<Literal> {
        assert_eq!(self.n_args, s.args.len());
        assert!(self.nodes[self.state].simulate(&s).is_none());
        self.result.and_then(|body| self.nodes[body].simulate(&s))
    }
}

#[derive(Clone)]
struct Simulator<'a> {
    args: &'a [Literal],
    nodes: &'a [PegBody],
    indices: &'a Indices,
    program: &'a PegProgram,
    stdout: Rc<RefCell<String>>,
}

impl PegBody {
    // Returns None if the output is a print edge
    fn simulate(
        &self,
        s: &Simulator,
    ) -> Option<Literal> {
        match self {
            PegBody::BasicOp(expr) => match expr {
                Expr::Op(op, xs, _ty) => {
                    let xs: Vec<_> = xs
                        .iter()
                        .map(|x| s.nodes[*x].simulate(s))
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
                        .map(|x| s.nodes[*x].simulate(s))
                        .map(Option::unwrap)
                        .collect();
                    let mut s = Simulator {
                        args: &xs, ..s.clone()
                    };
                    s.program
                        .functions
                        .iter()
                        .find(|func| func.name == *f)
                        .unwrap()
                        .simulate(&mut s)
                }
                Expr::Print(xs) => {
                    for x in xs {
                        let value = s.nodes[*x].simulate(s);
                        // if not a print edge
                        if let Some(value) = value {
                            s.stdout.borrow_mut().push_str(&format!("{}\n", value));
                        }
                    }
                    None
                }
                Expr::Const(ConstOps::Const, literal, _) => Some(literal.clone()),
            },
            PegBody::Arg(arg) => {
                if *arg == s.args.len() {
                    // this is a print edge
                    None
                } else {
                    Some(s.args[*arg].clone())
                }
            }
            PegBody::Phi(c, x, y) => {
                let c = s.nodes[*c].simulate(s);
                let x = s.nodes[*x].simulate(s);
                let y = s.nodes[*y].simulate(s);
                if bool(c) {
                    x
                } else {
                    y
                }
            }
            PegBody::Theta(a, b, l) => {
                let c = s.indices.get(*l);
                if c == 0 {
                    s.nodes[*a].simulate(s)
                } else {
                    let mut s = Simulator {
                        indices: &s.indices.set(*l, c - 1),
                        ..s.clone()
                    };
                    s.nodes[*b].simulate(&mut s)
                }
            }
            PegBody::Eval(q, i, l) => {
                let i = s.nodes[*i].simulate(s);
                let mut s = Simulator {
                    indices: &s.indices.set(*l, int(i).try_into().unwrap()),
                    ..s.clone()
                };
                s.nodes[*q].simulate(&mut s)
            }
            PegBody::Pass(q, l) => {
                let mut i = 0;
                loop {
                    let mut s = Simulator {
                        indices: &s.indices.set(*l, i),
                        ..s.clone()
                    };
                    if !bool(s.nodes[*q].simulate(&mut s))
                    {
                        return Some(Literal::Int(i.try_into().unwrap()));
                    }
                    i += 1;
                }
            }
            PegBody::Edge(i) => s.nodes[*i].simulate(s),
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
