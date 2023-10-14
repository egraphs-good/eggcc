//! This module lets you interpret a PEG.

use crate::peg::{PegBody, PegProgram};
use crate::rvsdg::BasicExpr;
use bril_rs::{ConstOps, Literal, ValueOps};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash)]
enum Indices {
    Root,
    Node {
        key: usize,
        value: usize,
        parent: Rc<Indices>,
    },
}

fn indices_get(indices: &Indices, label: usize) -> usize {
    match indices {
        Indices::Root => 0,
        Indices::Node { key, value, .. } if *key == label => *value,
        Indices::Node { parent, .. } => indices_get(parent, label),
    }
}

fn indices_set(indices: Rc<Indices>, key: usize, value: usize) -> Rc<Indices> {
    Rc::new(Indices::Node {
        key,
        value,
        parent: indices,
    })
}

impl PegProgram {
    pub fn simulate(&self, args: &[Literal]) -> String {
        let main = self
            .functions
            .iter()
            .position(|f| f.name == "main")
            .unwrap();
        let mut s = Simulator {
            args: args.to_vec(),
            indices: Rc::new(Indices::Root),
            func: usize::MAX, // garbage value
            program: self,
            stdout: Rc::default(),
            memoizer: Rc::default(),
        };
        let output = s.simulate_func(main);
        assert!(output.is_none());
        Rc::try_unwrap(s.stdout).unwrap().into_inner()
    }
}

#[derive(Clone)]
struct Simulator<'a> {
    args: Vec<Literal>,
    indices: Rc<Indices>,
    func: usize,
    program: &'a PegProgram,
    stdout: Rc<RefCell<String>>,
    memoizer: Rc<RefCell<Memoizer>>,
}

/// A Map from (func, body, index) to the output of that body for that index.
type Memoizer = HashMap<(usize, usize, Rc<Indices>), Literal>;

impl Simulator<'_> {
    fn simulate_func(&mut self, func_index: usize) -> Option<Literal> {
        self.func = func_index;
        let func = &self.program.functions[func_index];
        assert_eq!(func.n_args, self.args.len());
        assert!(func
            .state
            .and_then(|state| self.simulate_body(state))
            .is_none());
        func.result.and_then(|body| self.simulate_body(body))
    }

    // Returns None if the output is a print edge
    fn simulate_body(&self, body_index: usize) -> Option<Literal> {
        if let Some(out) =
            self.memoizer
                .borrow()
                .get(&(self.func, body_index, self.indices.clone()))
        {
            return Some(out.clone());
        }
        let out = match &self.program.functions[self.func].nodes[body_index] {
            PegBody::BasicOp(expr) => match expr {
                BasicExpr::Op(op, xs, _ty) => {
                    let xs: Vec<_> = xs.iter().map(|x| self.simulate_body(*x)).collect();
                    match op {
                        ValueOps::Add => {
                            Some(Literal::Int(int(xs[0].clone()) + int(xs[1].clone())))
                        }
                        ValueOps::Sub => {
                            Some(Literal::Int(int(xs[0].clone()) - int(xs[1].clone())))
                        }
                        ValueOps::Mul => {
                            Some(Literal::Int(int(xs[0].clone()) * int(xs[1].clone())))
                        }
                        ValueOps::Div => {
                            Some(Literal::Int(int(xs[0].clone()) / int(xs[1].clone())))
                        }
                        ValueOps::Fadd => {
                            Some(Literal::Float(float(xs[0].clone()) + float(xs[1].clone())))
                        }
                        ValueOps::Fsub => {
                            Some(Literal::Float(float(xs[0].clone()) - float(xs[1].clone())))
                        }
                        ValueOps::Fmul => {
                            Some(Literal::Float(float(xs[0].clone()) * float(xs[1].clone())))
                        }
                        ValueOps::Fdiv => {
                            Some(Literal::Float(float(xs[0].clone()) / float(xs[1].clone())))
                        }
                        ValueOps::Lt => {
                            Some(Literal::Bool(int(xs[0].clone()) < int(xs[1].clone())))
                        }
                        op => todo!("implement {op}"),
                    }
                }
                BasicExpr::Call(f, xs, _, _, pure) => {
                    let args: Vec<_> = xs.iter().flat_map(|x| self.simulate_body(*x)).collect();
                    // Pure functions should not have non-value arguments
                    assert!(!pure || xs.len() == args.len());
                    let mut s = Simulator {
                        args,
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
                BasicExpr::Print(xs) => {
                    for x in xs {
                        let value = self.simulate_body(*x);
                        // if not a print edge
                        if let Some(value) = value {
                            self.stdout.borrow_mut().push_str(&format!("{}\n", value));
                        }
                    }
                    None
                }
                BasicExpr::Const(ConstOps::Const, literal, _) => Some(literal.clone()),
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
                if bool(self.simulate_body(*c)) {
                    self.simulate_body(*x)
                } else {
                    self.simulate_body(*y)
                }
            }
            PegBody::Theta(a, b, l) => {
                let c = indices_get(&self.indices, *l);
                if c == 0 {
                    self.simulate_body(*a)
                } else {
                    let s = Simulator {
                        indices: indices_set(self.indices.clone(), *l, c - 1),
                        ..self.clone()
                    };
                    s.simulate_body(*b)
                }
            }
            PegBody::Eval(q, i, l) => {
                let i = self.simulate_body(*i);
                let s = Simulator {
                    indices: indices_set(self.indices.clone(), *l, int(i).try_into().unwrap()),
                    ..self.clone()
                };
                s.simulate_body(*q)
            }
            PegBody::Pass(q, l) => {
                let mut i = 0;
                loop {
                    let s = Simulator {
                        indices: indices_set(self.indices.clone(), *l, i),
                        ..self.clone()
                    };
                    if !bool(s.simulate_body(*q)) {
                        return Some(Literal::Int(i.try_into().unwrap()));
                    }
                    i += 1;
                }
            }
            PegBody::Edge(i) => self.simulate_body(*i),
        };
        if let Some(out) = out.clone() {
            let old = self
                .memoizer
                .borrow_mut()
                .insert((self.func, body_index, self.indices.clone()), out);
            assert!(old.is_none());
        }
        out
    }
}

fn int(literal: Option<Literal>) -> i64 {
    match literal.unwrap() {
        Literal::Int(x) => x,
        literal => panic!("expected int, found {literal}"),
    }
}

fn float(literal: Option<Literal>) -> f64 {
    match literal.unwrap() {
        Literal::Float(x) => x,
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
