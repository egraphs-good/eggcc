//! This file contains helpers for making the extracted
//! program use memory linearly.
//! In particular, it finds all the effectful e-nodes in an extracted term that are along the state edge path.

use std::{
    collections::{HashMap, HashSet},
    iter,
    rc::Rc,
};

use egglog::Term;
use egraph_serialize::{ClassId, NodeId};
use indexmap::{IndexMap, IndexSet};

use crate::{
    greedy_dag_extractor::{get_root, EgraphInfo, Extractor},
    schema::{Expr, *},
};

type EffectfulNodes = Vec<(ClassId, *const Expr)>;

struct Linearity {
    effectful_nodes: EffectfulNodes,
    expr_to_term: HashMap<*const Expr, Term>,
    n2c: HashMap<NodeId, ClassId>,
}

impl<'a> Extractor<'a> {
    /// Finds all the effectful nodes along the state
    /// edge path (the path of the state edge from the argument to the return value).
    /// Input: a term representing the program
    /// Output: a map from root ids to the set of effectful nodes along the state edge path in this region
    pub fn find_effectful_nodes_in_program(
        &mut self,
        prog: &TreeProgram,
        egraph_info: &EgraphInfo,
    ) -> HashMap<ClassId, HashSet<NodeId>> {
        let mut expr_to_term = HashMap::new();
        for (term, expr) in self.term_to_expr.as_ref().unwrap() {
            expr_to_term.insert(Rc::as_ptr(expr), term.clone());
        }
        let n2c = egraph_info
            .egraph
            .nodes
            .iter()
            .map(|(node_id, node)| (node_id.clone(), node.eclass.clone()))
            .collect();

        let prog_root_id = get_root(egraph_info.egraph); // should be the id of prog
        let prog_root_id = egraph_info.egraph.nid_to_cid(&prog_root_id);
        let mut linearity = Linearity {
            effectful_nodes: vec![],
            expr_to_term,
            n2c,
        };

        self.find_effectful_nodes_in_expr(&prog.entry, &mut linearity, prog_root_id);
        for function in &prog.functions {
            self.find_effectful_nodes_in_expr(function, &mut linearity, prog_root_id);
        }

        let mut effectful_classes: HashMap<ClassId, HashSet<NodeId>> = Default::default();
        for (rootid, expr) in linearity.effectful_nodes {
            let term = linearity.expr_to_term.get(&expr).unwrap();
            effectful_classes
                .entry(rootid.clone())
                .or_default()
                .insert(self.node_of(term));
        }

        effectful_classes
    }

    /// Finds all the effectful nodes along the state edge.
    /// When `recur_subregions` is true, it also finds effectful nodes in subregions.
    fn find_effectful_nodes_in_expr(
        &mut self,
        expr: &RcExpr,
        linearity: &mut Linearity,
        rootid: &ClassId,
    ) {
        let class_of_expr = |expr: &RcExpr, linearity: &Linearity, ext: &Extractor| {
            let term = linearity.expr_to_term.get(&Rc::as_ptr(expr)).unwrap();
            let nodeid = ext.node_of(term);
            linearity.n2c.get(&nodeid).unwrap().clone()
        };
        linearity
            .effectful_nodes
            .push((rootid.clone(), Rc::as_ptr(expr)));
        match expr.as_ref() {
            Expr::Top(op, _c1, _c2, c3) => match op {
                TernaryOp::Write => {
                    // c3 is the state edge
                    self.find_effectful_nodes_in_expr(c3, linearity, rootid)
                }
            },
            Expr::Bop(op, _c1, c2) => {
                match op {
                    BinaryOp::Load | BinaryOp::Print | BinaryOp::Free => {
                        // c2 is the state edge
                        self.find_effectful_nodes_in_expr(c2, linearity, rootid)
                    }
                    _ => {
                        panic!("BinaryOp {:?} is not effectful", op)
                    }
                }
            }
            Expr::Uop(op, _) => {
                panic!("UnaryOp {:?} is not effectful", op)
            }
            Expr::Get(child, _index) => self.find_effectful_nodes_in_expr(child, linearity, rootid),
            Expr::Alloc(_id, _amt, state, _ty) => {
                self.find_effectful_nodes_in_expr(state, linearity, rootid)
            }
            Expr::Call(_name, input) => self.find_effectful_nodes_in_expr(input, linearity, rootid),
            Expr::Empty(_, _ctx) => {
                panic!("Empty has no effect")
            }
            Expr::Single(expr) => self.find_effectful_nodes_in_expr(expr, linearity, rootid),
            Expr::Concat(c1, c2) => {
                let left_contains_state = self.is_effectful(c1);
                let right_contains_state = self.is_effectful(c2);
                assert!(left_contains_state || right_contains_state);
                assert!(!(left_contains_state && right_contains_state));
                if left_contains_state {
                    self.find_effectful_nodes_in_expr(c1, linearity, rootid)
                } else {
                    self.find_effectful_nodes_in_expr(c2, linearity, rootid)
                }
            }
            Expr::If(_pred, input, then_branch, else_branch) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity, rootid);
                let then_root_id = class_of_expr(then_branch, linearity, self);
                let else_root_id = class_of_expr(else_branch, linearity, self);
                self.find_effectful_nodes_in_expr(then_branch, linearity, &then_root_id);
                self.find_effectful_nodes_in_expr(else_branch, linearity, &else_root_id);
            }
            Expr::Switch(_pred, input, branches) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity, rootid);
                for branch in branches {
                    let branch_root_id = class_of_expr(branch, linearity, self);
                    self.find_effectful_nodes_in_expr(branch, linearity, &branch_root_id);
                }
            }
            Expr::DoWhile(input, body) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity, rootid);
                let body_root_id = class_of_expr(body, linearity, self);
                self.find_effectful_nodes_in_expr(body, linearity, &body_root_id);
            }
            Expr::Arg(ty, _ctx) => {
                assert!(ty.contains_state());
            }
            Expr::Function(_name, _inty, outty, body) => {
                if !outty.contains_state() {
                    panic!("Function output does not contain state");
                }
                let body_root_id = class_of_expr(body, linearity, self);
                self.find_effectful_nodes_in_expr(body, linearity, &body_root_id)
            }
            Expr::Const(_, _, _) => panic!("Const has no effect"),
        }
    }

    pub fn check_program_is_linear(&mut self, prog: &TreeProgram) -> Result<(), String> {
        for fun in iter::once(&prog.entry).chain(prog.functions.iter()) {
            let mut reachables = Default::default();
            fun.collect_reachable(fun, &mut reachables);

            // if the raw pointer is effectful, then return its RcExpr, otherwise None.
            // Gracefully handles `Function` which is not supported by Extractor::is_effectful.
            let get_if_effectful = |this: &mut Extractor<'a>, expr: *const Expr| {
                let rcexpr = unsafe {
                    Rc::increment_strong_count(expr);
                    Rc::from_raw(expr)
                };
                if let Expr::Function(_name, _inp, _out, body) = rcexpr.as_ref() {
                    if this.is_effectful(body) {
                        return Some(rcexpr);
                    }
                } else if this.is_effectful(&rcexpr) {
                    return Some(rcexpr);
                }
                None
            };

            for (region, exprs) in reachables {
                let mut dangling_effectful: HashSet<*const Expr> = exprs
                    .iter()
                    .filter_map(|&expr| {
                        // This is because from_raw creates a new Rc
                        // whose drop will decrement strong count.
                        if get_if_effectful(self, expr).is_some() {
                            Some(expr)
                        } else {
                            None
                        }
                    })
                    .collect();

                for expr in exprs {
                    let Some(expr) = get_if_effectful(self, expr) else {
                        continue;
                    };
                    // Arg is a leaf and does not have effectful children.
                    if !matches!(expr.as_ref(), Expr::Arg(..)) {
                        // We can view region nodes as a giant opaque operator
                        // and only need to consider children that are in the same scope
                        let children = expr.children_same_scope();
                        let mut effectful_child_iter =
                            children.iter().filter(|child| self.is_effectful(child));
                        let effectful_child = effectful_child_iter
                            .next()
                            .expect("Expect one effectful child from an effectful operator");
                        assert!(effectful_child_iter.next().is_none());
                        if !dangling_effectful.remove(&Rc::as_ptr(effectful_child)) {
                            return Err("An effectful operator is used twice".to_string());
                        }
                    }
                }
                if !dangling_effectful.remove(&region) {
                    return Err(
                        "The region operator is either consumed or not effectful.".to_string()
                    );
                }
                if !dangling_effectful.is_empty() {
                    return Err("There are unconsumed effectful operators".to_string());
                }
            }
        }
        Ok(())
    }
}

impl Expr {
    pub fn collect_reachable(
        self: &RcExpr,
        root: &RcExpr,
        reachable_from: &mut IndexMap<*const Expr, IndexSet<*const Expr>>,
    ) {
        if !reachable_from
            .entry(Rc::as_ptr(root))
            .or_default()
            .insert(Rc::as_ptr(self))
        {
            return;
        }

        match self.as_ref() {
            Expr::If(pred, input, t1, t2) => {
                pred.collect_reachable(root, reachable_from);
                input.collect_reachable(root, reachable_from);
                let root = t1;
                t1.collect_reachable(root, reachable_from);
                let root = t2;
                t2.collect_reachable(root, reachable_from);
            }
            Expr::Switch(pred, inputs, branches) => {
                pred.collect_reachable(root, reachable_from);
                inputs.collect_reachable(root, reachable_from);
                for branch in branches {
                    let root = branch;
                    branch.collect_reachable(root, reachable_from);
                }
            }
            Expr::DoWhile(input, body) => {
                input.collect_reachable(root, reachable_from);
                let root = body;
                body.collect_reachable(root, reachable_from);
            }
            _ => {
                for child in self.children_same_scope() {
                    child.collect_reachable(root, reachable_from);
                }
            }
        }
    }
}
