//! This file contains helpers for making the extracted
//! program use memory linearly.
//! In particular, it finds all the effectful e-nodes in an extracted term that are along the state edge path.

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use egglog::Term;
use egraph_serialize::NodeId;

use crate::{
    greedy_dag_extractor::Extractor,
    schema::{Expr, *},
};

type EffectfulNodes = Vec<*const Expr>;

struct Linearity {
    effectful_nodes: EffectfulNodes,
    recur_subregions: bool,
}

impl<'a> Extractor<'a> {
    /// Finds all the effectful nodes along the state
    /// edge path (the path of the state edge from the argument to the return value).
    /// Input: a term representing the program
    /// Output: a vector of terms representing the effectful nodes along the state edge path
    pub fn find_effectful_nodes_in_program(&mut self, prog: &TreeProgram) -> HashSet<NodeId> {
        let mut expr_to_term = HashMap::new();
        for (term, expr) in self.term_to_expr.as_ref().unwrap() {
            expr_to_term.insert(Rc::as_ptr(expr), term.clone());
        }

        let mut linearity = Linearity {
            effectful_nodes: vec![],
            recur_subregions: true,
        };

        self.find_effectful_nodes_in_expr(&prog.entry, &mut linearity);
        for function in &prog.functions {
            self.find_effectful_nodes_in_expr(function, &mut linearity);
        }

        let mut effectful_classes = HashSet::new();
        for expr in linearity.effectful_nodes {
            let term = expr_to_term.get(&expr).unwrap();
            effectful_classes.insert(self.node_of(term));
        }

        effectful_classes
    }

    #[allow(dead_code)]
    pub fn find_effectful_nodes_in_region(&mut self, term: &Term) -> HashSet<NodeId> {
        let expr = self.term_to_expr(term);
        let mut expr_to_term = HashMap::new();
        for (term, expr) in self.term_to_expr.as_ref().unwrap() {
            expr_to_term.insert(Rc::as_ptr(expr), term.clone());
        }

        let mut linearity = Linearity {
            effectful_nodes: vec![],
            recur_subregions: true,
        };

        self.find_effectful_nodes_in_expr(&expr, &mut linearity);

        let mut effectful_classes = HashSet::new();
        for expr in linearity.effectful_nodes {
            let term = expr_to_term.get(&expr).unwrap();
            effectful_classes.insert(self.node_of(term));
        }

        effectful_classes
    }

    /// Finds all the effectful nodes along the state edge.
    /// When `recur_subregions` is true, it also finds effectful nodes in subregions.
    fn find_effectful_nodes_in_expr(&mut self, expr: &RcExpr, linearity: &mut Linearity) {
        linearity.effectful_nodes.push(Rc::as_ptr(expr));
        match expr.as_ref() {
            Expr::Top(op, _c1, _c2, c3) => match op {
                TernaryOp::Write => {
                    // c3 is the state edge
                    self.find_effectful_nodes_in_expr(c3, linearity)
                }
            },
            Expr::Bop(op, _c1, c2) => {
                match op {
                    BinaryOp::Load | BinaryOp::Print | BinaryOp::Free => {
                        // c2 is the state edge
                        self.find_effectful_nodes_in_expr(c2, linearity)
                    }
                    _ => {
                        panic!("BinaryOp {:?} is not effectful", op)
                    }
                }
            }
            Expr::Uop(op, _) => {
                panic!("UnaryOp {:?} is not effectful", op)
            }
            Expr::Get(child, _index) => self.find_effectful_nodes_in_expr(child, linearity),
            Expr::Alloc(_id, _amt, state, _ty) => {
                self.find_effectful_nodes_in_expr(state, linearity)
            }
            Expr::Call(_name, input) => self.find_effectful_nodes_in_expr(input, linearity),
            Expr::Empty(_) => {
                panic!("Empty has no effect")
            }
            Expr::Single(expr) => self.find_effectful_nodes_in_expr(expr, linearity),
            Expr::Concat(c1, c2) => {
                let left_contains_state = self.is_effectful(c1);
                let right_contains_state = self.is_effectful(c2);
                assert!(left_contains_state || right_contains_state);
                assert!(!(left_contains_state && right_contains_state));
                if left_contains_state {
                    self.find_effectful_nodes_in_expr(c1, linearity)
                } else {
                    self.find_effectful_nodes_in_expr(c2, linearity)
                }
            }
            Expr::If(_pred, input, then_branch, else_branch) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity);
                if linearity.recur_subregions {
                    self.find_effectful_nodes_in_expr(then_branch, linearity);
                    self.find_effectful_nodes_in_expr(else_branch, linearity);
                }
            }
            Expr::Switch(_pred, input, branches) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity);
                if linearity.recur_subregions {
                    for branch in branches {
                        self.find_effectful_nodes_in_expr(branch, linearity);
                    }
                }
            }
            Expr::DoWhile(input, body) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity);
                if linearity.recur_subregions {
                    self.find_effectful_nodes_in_expr(body, linearity);
                }
            }
            Expr::Arg(ty) => {
                assert!(ty.contains_state());
            }
            Expr::InContext(_ctx, body) => self.find_effectful_nodes_in_expr(body, linearity),
            Expr::Function(_name, _inty, outty, body) => {
                if !outty.contains_state() {
                    panic!("Function output does not contain state");
                }
                self.find_effectful_nodes_in_expr(body, linearity)
            }
            Expr::Const(_, _) => panic!("Const has no effect"),
        }
    }
}
