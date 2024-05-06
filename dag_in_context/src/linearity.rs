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
    greedy_dag_extractor::{EgraphInfo, Extractor},
    schema::{Expr, *},
};

type EffectfulNodes = IndexMap<ClassId, IndexSet<*const Expr>>;

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

        let mut linearity = Linearity {
            effectful_nodes: Default::default(),
            expr_to_term,
            n2c,
        };

        self.find_effectful_nodes_in_region(prog.entry.func_body().unwrap(), &mut linearity);
        for function in &prog.functions {
            // root of the function is the body of the function
            self.find_effectful_nodes_in_region(function.func_body().unwrap(), &mut linearity);
        }

        let effectful_nodes: HashMap<ClassId, HashSet<NodeId>> = linearity
            .effectful_nodes
            .into_iter()
            .map(|(k, v)| {
                let v = v
                    .into_iter()
                    .flat_map(|expr| self.term_nodes(linearity.expr_to_term.get(&expr).unwrap()))
                    .collect();
                (k, v)
            })
            .collect();

        // assert that we only find one node per eclass
        for nodes in effectful_nodes.values() {
            let mut eclasses = HashSet::new();
            for node in nodes {
                assert!(eclasses.insert(egraph_info.egraph.nid_to_cid(node)));
            }
        }

        effectful_nodes
    }

    fn classes_of_expr(&self, linearity: &Linearity, expr: &RcExpr) -> HashSet<ClassId> {
        let term = linearity.expr_to_term.get(&Rc::as_ptr(expr)).unwrap();
        let nodeids = self.term_nodes(term);
        nodeids
            .iter()
            .map(|nodeid| linearity.n2c.get(nodeid).unwrap().clone())
            .collect()
    }

    /// Start finding effectful nodes from the root of the region
    /// If we've already visited this region, we should not visit again
    /// (otherwise, we may pick two paths to the same region, which is unsound)
    fn find_effectful_nodes_in_region(&mut self, expr: &RcExpr, linearity: &mut Linearity) {
        let rootids = self.classes_of_expr(linearity, expr);
        for rootid in rootids {
            // if we have already visited this region, we should not visit again
            if linearity.effectful_nodes.contains_key(&rootid) {
                return;
            }
            let mut res = Default::default();
            self.find_effectful_nodes_in_expr(expr, linearity, &mut res);

            if linearity.effectful_nodes.contains_key(&rootid) {
                panic!("The same region was visited twice before being set by `find_effectful_nodes_in_region`");
            }

            linearity.effectful_nodes.insert(rootid.clone(), res);
        }
    }

    /// Finds all the effectful nodes along the state edge
    /// in the same region
    fn find_effectful_nodes_in_expr(
        &mut self,
        expr: &RcExpr,
        linearity: &mut Linearity,
        res: &mut IndexSet<*const Expr>,
    ) {
        if !res.insert(Rc::as_ptr(expr)) {
            panic!("The same expression was visited twice by `find_effectful_nodes_in_expr`");
        }
        match expr.as_ref() {
            Expr::Top(op, _c1, _c2, c3) => match op {
                TernaryOp::Write => {
                    // c3 is the state edge
                    self.find_effectful_nodes_in_expr(c3, linearity, res)
                }
            },
            Expr::Bop(op, _c1, c2) => {
                match op {
                    BinaryOp::Load | BinaryOp::Print | BinaryOp::Free => {
                        // c2 is the state edge
                        self.find_effectful_nodes_in_expr(c2, linearity, res)
                    }
                    _ => {
                        panic!("BinaryOp {:?} is not effectful", op)
                    }
                }
            }
            Expr::Uop(op, _) => {
                panic!("UnaryOp {:?} is not effectful", op)
            }
            Expr::Get(child, _index) => self.find_effectful_nodes_in_expr(child, linearity, res),
            Expr::Alloc(_id, _amt, state, _ty) => {
                self.find_effectful_nodes_in_expr(state, linearity, res)
            }
            Expr::Call(_name, input) => self.find_effectful_nodes_in_expr(input, linearity, res),
            Expr::Empty(_, _ctx) => {
                panic!("Empty has no effect")
            }
            Expr::Single(expr) => self.find_effectful_nodes_in_expr(expr, linearity, res),
            Expr::Concat(c1, c2) => {
                let left_contains_state = self.is_effectful(c1);
                let right_contains_state = self.is_effectful(c2);
                assert!(left_contains_state || right_contains_state);
                assert!(!(left_contains_state && right_contains_state));
                if left_contains_state {
                    self.find_effectful_nodes_in_expr(c1, linearity, res)
                } else {
                    self.find_effectful_nodes_in_expr(c2, linearity, res)
                }
            }
            Expr::If(_pred, input, then_branch, else_branch) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity, res);
                self.find_effectful_nodes_in_region(then_branch, linearity);
                self.find_effectful_nodes_in_region(else_branch, linearity);
            }
            Expr::Switch(_pred, input, branches) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity, res);
                for branch in branches {
                    self.find_effectful_nodes_in_region(branch, linearity);
                }
            }
            Expr::DoWhile(input, body) => {
                let input_contains_state = self.is_effectful(input);
                assert!(input_contains_state);

                self.find_effectful_nodes_in_expr(input, linearity, res);
                self.find_effectful_nodes_in_region(body, linearity);
            }
            Expr::Arg(ty, _ctx) => {
                assert!(ty.contains_state());
            }
            Expr::Function(_name, _inty, outty, body) => {
                if !outty.contains_state() {
                    panic!("Function output does not contain state");
                }
                self.find_effectful_nodes_in_region(body, linearity)
            }
            Expr::Const(_, _, _) => panic!("Const has no effect"),
        }
    }

    pub fn check_program_is_linear(&mut self, prog: &TreeProgram) -> Result<(), String> {
        for fun in iter::once(&prog.entry).chain(prog.functions.iter()) {
            let mut reachables = Default::default();
            let mut raw_to_rc = Default::default();
            let fun_body = fun.func_body().unwrap();
            fun_body.collect_reachable(fun_body, &mut reachables, &mut raw_to_rc);

            // if the raw pointer is effectful, then return its RcExpr, otherwise None.
            // Gracefully handles `Function` which is not supported by Extractor::is_effectful.
            let get_if_effectful = |this: &mut Extractor<'a>, expr: *const Expr| {
                let rcexpr = raw_to_rc.get(&expr).unwrap();
                if let Expr::Function(_name, _inp, _out, body) = rcexpr.as_ref() {
                    if this.is_effectful(body) {
                        return Some(rcexpr);
                    }
                } else if this.is_effectful(rcexpr) {
                    return Some(rcexpr);
                }
                None
            };

            for (region, exprs) in reachables {
                // get all effectful operators in the region,
                // and check if they are used exactly once.
                let mut dangling_effectful: HashSet<*const Expr> = exprs
                    .iter()
                    .filter_map(|&expr| {
                        if get_if_effectful(self, expr).is_some() {
                            Some(expr)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut effectful_parent: HashMap<*const Expr, RcExpr> = Default::default();

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
                            return Err(
                                format!("Resulting program violated linearity! Effectful expression's state edge was referenced twice. Parent 1: {}\n\n Parent 2: {}\n\n Child referenced twice: {}", effectful_parent.get(&Rc::as_ptr(effectful_child)).unwrap(), expr, effectful_child),
                            );
                        }
                        effectful_parent.insert(Rc::as_ptr(effectful_child), expr.clone());
                    }
                }
                if get_if_effectful(self, region).is_some() && !dangling_effectful.remove(&region) {
                    panic!("The region operator is either consumed or not effectful.");
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
    /// Populates the reachable_from table, which is a map from the root of the region
    /// to the set of reachable nodes, stored as raw pointers.
    /// Additionally return a map from raw pointers to RcExpr.
    pub fn collect_reachable(
        self: &RcExpr,
        root: &RcExpr,
        reachable_from: &mut IndexMap<*const Expr, IndexSet<*const Expr>>,
        raw_to_rc: &mut HashMap<*const Expr, RcExpr>,
    ) {
        raw_to_rc
            .entry(Rc::as_ptr(self))
            .or_insert_with(|| self.clone());
        if !reachable_from
            .entry(Rc::as_ptr(root))
            .or_default()
            .insert(Rc::as_ptr(self))
        {
            return;
        }

        match self.as_ref() {
            Expr::If(pred, input, t1, t2) => {
                pred.collect_reachable(root, reachable_from, raw_to_rc);
                input.collect_reachable(root, reachable_from, raw_to_rc);
                let root = t1;
                t1.collect_reachable(root, reachable_from, raw_to_rc);
                let root = t2;
                t2.collect_reachable(root, reachable_from, raw_to_rc);
            }
            Expr::Switch(pred, inputs, branches) => {
                pred.collect_reachable(root, reachable_from, raw_to_rc);
                inputs.collect_reachable(root, reachable_from, raw_to_rc);
                for branch in branches {
                    let root = branch;
                    branch.collect_reachable(root, reachable_from, raw_to_rc);
                }
            }
            Expr::DoWhile(input, body) => {
                input.collect_reachable(root, reachable_from, raw_to_rc);
                let root = body;
                body.collect_reachable(root, reachable_from, raw_to_rc);
            }
            _ => {
                for child in self.children_same_scope() {
                    child.collect_reachable(root, reachable_from, raw_to_rc);
                }
            }
        }
    }
}
