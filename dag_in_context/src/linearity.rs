//! This file contains helpers for making the extracted
//! program use memory linearly.

use std::{collections::HashMap, rc::Rc};

use egglog::{match_term_app, Term, TermDag};
use egraph_serialize::{ClassId, EGraph};

use crate::{
    from_egglog::FromEgglog,
    greedy_dag_extractor::Extractor,
    schema::{Expr, Type, *},
    typechecker::TypeCache,
};

type EffectfulNodes = Vec<*const Expr>;

struct Linearity {
    expr_types: TypeCache,
    effectful_nodes: EffectfulNodes,
}

impl Linearity {
    fn expr_has_state_edge(&self, expr: &RcExpr) -> bool {
        self.expr_types
            .get(&Rc::as_ptr(expr))
            .unwrap()
            .contains_state()
    }
}

impl<'a> Extractor<'a> {
    /// Finds all the effectful nodes along the state
    /// edge path (the path of the state edge from the argument to the return value).
    /// Input: a term representing the program
    /// Output: a vector of terms representing the effectful nodes along the state edge path
    pub fn find_effectful_nodes_in_program(&mut self, term: &Term) {
        let mut converter = FromEgglog {
            termdag: self.termdag,
            conversion_cache: HashMap::new(),
        };
        let prog = converter.program_from_egglog(term.clone());
        let mut expr_to_term = HashMap::new();
        for (term, expr) in converter.conversion_cache {
            expr_to_term.insert(Rc::as_ptr(&expr), term);
        }

        let type_cache = prog.typecheck();
        let mut linearity = Linearity {
            expr_types: type_cache,
            effectful_nodes: vec![],
        };

        self.find_effectful_nodes_in_expr(&prog.entry, &mut linearity);
        for function in prog.functions {
            self.find_effectful_nodes_in_expr(&function, &mut linearity);
        }

        todo!()
    }

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
                let left_contains_state = linearity.expr_has_state_edge(c1);
                let right_contains_state = linearity.expr_has_state_edge(c2);
                assert!(left_contains_state || right_contains_state);
                assert!(!(left_contains_state && right_contains_state));
                if left_contains_state {
                    self.find_effectful_nodes_in_expr(c1, linearity)
                } else {
                    self.find_effectful_nodes_in_expr(c2, linearity)
                }
            }
            Expr::If(_pred, input, then_branch, else_branch) => {
                let input_contains_state = linearity.expr_has_state_edge(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity);
                self.find_effectful_nodes_in_expr(then_branch, linearity);
                self.find_effectful_nodes_in_expr(else_branch, linearity);
            }
            Expr::Switch(_pred, input, branches) => {
                let input_contains_state = linearity.expr_has_state_edge(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity);
                for branch in branches {
                    self.find_effectful_nodes_in_expr(branch, linearity);
                }
            }
            Expr::DoWhile(input, body) => {
                let input_contains_state = linearity.expr_has_state_edge(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity);
                self.find_effectful_nodes_in_expr(body, linearity);
            }
            Expr::Arg(ty) => {
                assert!(ty.contains_state());
            }
            Expr::InContext(_ctx, body) => self.find_effectful_nodes_in_expr(body, linearity),
            Expr::Function(_, _, _, body) => self.find_effectful_nodes_in_expr(body, linearity),
            Expr::Const(_, _) => panic!("Const has no effect"),
        }
    }
}
