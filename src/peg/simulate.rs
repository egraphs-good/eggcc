//! This module lets you interpret a PEG.

use crate::cfg::Identifier;
use crate::peg::{PegBody, PegProgram};
use crate::rvsdg::Expr;
use bril_rs::{ConstOps, Literal, ValueOps};
use std::cell::RefCell;
use std::rc::Rc;

enum Indices<'a> {
    Root,
    Node {
        key: usize,
        value: usize,
        parent: &'a Indices<'a>,
    },
}

impl Indices<'_> {
    fn get(&self, label: usize) -> usize {
        match self {
            Indices::Root => 0,
            Indices::Node { key, value, .. } if *key == label => *value,
            Indices::Node { parent, .. } => parent.get(label),
        }
    }

    fn set(&self, key: usize, value: usize) -> Indices {
        Indices::Node {
            key,
            value,
            parent: self,
        }
    }
}

impl PegProgram {
    pub fn simulate(&self, args: &[Literal]) -> String {
        let main = self
            .functions
            .iter()
            .position(|f| f.name == "main")
            .unwrap();
        let mut s = Simulator {
            args,
            indices: &Indices::Root,
            func: usize::MAX, // garbage value
            program: self,
            stdout: Rc::new(RefCell::new(String::new())),
        };
        let output = s.simulate_func(main);
        assert!(output.is_none());
        Rc::try_unwrap(s.stdout).unwrap().into_inner()
    }
}

#[derive(Clone)]
struct Simulator<'a> {
    args: &'a [Literal],
    indices: &'a Indices<'a>,
    func: usize,
    program: &'a PegProgram,
    stdout: Rc<RefCell<String>>,
}

impl Simulator<'_> {
    fn simulate_func(&mut self, i: usize) -> Option<Literal> {
        self.func = i;
        let func = &self.program.functions[i];
        assert_eq!(func.n_args, self.args.len());
        assert!(self.simulate_body(func.state).is_none());
        func.result.and_then(|body| self.simulate_body(body))
    }

    // Returns None if the output is a print edge
    fn simulate_body(&self, i: usize) -> Option<Literal> {
        match &self.program.functions[self.func].nodes[i] {
            PegBody::BasicOp(expr) => match expr {
                Expr::Op(op, xs, _ty) => {
                    let xs: Vec<_> = xs.iter().map(|x| self.simulate_body(*x)).collect();
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
                        .map(|x| self.simulate_body(*x))
                        .map(Option::unwrap)
                        .collect();
                    let mut s = Simulator {
                        args: &xs,
                        ..self.clone()
                    };
                    s.simulate_func(
                        s.program
                            .functions
                            .iter()
                            .position(|func| func.name == *f)
                            .unwrap(),
                    )
                }
                Expr::Print(xs) => {
                    for x in xs {
                        let value = self.simulate_body(*x);
                        // if not a print edge
                        if let Some(value) = value {
                            self.stdout.borrow_mut().push_str(&format!("{}\n", value));
                        }
                    }
                    None
                }
                Expr::Const(ConstOps::Const, literal, _) => Some(literal.clone()),
            },
            PegBody::Arg(arg) => {
                if *arg == self.args.len() {
                    // this is a print edge
                    None
                } else {
                    Some(self.args[*arg].clone())
                }
            }
            PegBody::Phi(c, x, y) => {
                let c = self.simulate_body(*c);
                let x = self.simulate_body(*x);
                let y = self.simulate_body(*y);
                if bool(c) {
                    x
                } else {
                    y
                }
            }
            PegBody::Theta(a, b, l) => {
                let c = self.indices.get(*l);
                if c == 0 {
                    self.simulate_body(*a)
                } else {
                    let s = Simulator {
                        indices: &self.indices.set(*l, c - 1),
                        ..self.clone()
                    };
                    s.simulate_body(*b)
                }
            }
            PegBody::Eval(q, i, l) => {
                let i = self.simulate_body(*i);
                let s = Simulator {
                    indices: &self.indices.set(*l, int(i).try_into().unwrap()),
                    ..self.clone()
                };
                s.simulate_body(*q)
            }
            PegBody::Pass(q, l) => {
                let mut i = 0;
                loop {
                    let s = Simulator {
                        indices: &self.indices.set(*l, i),
                        ..self.clone()
                    };
                    if !bool(s.simulate_body(*q)) {
                        return Some(Literal::Int(i.try_into().unwrap()));
                    }
                    i += 1;
                }
            }
            PegBody::Edge(i) => self.simulate_body(*i),
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
