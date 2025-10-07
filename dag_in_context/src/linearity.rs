//! This file contains helpers for making the extracted
//! program use memory linearly.
//! In particular, it finds all the effectful e-nodes in an extracted term that are along the state edge path.

use std::{collections::HashSet, rc::Rc};

use crate::schema::{BinaryOp, Expr, RcExpr, TernaryOp, TreeProgram, Type};
use crate::typechecker::TypeChecker;
use egglog::Term;
use egraph_serialize::{ClassId, NodeId};
use indexmap::{IndexMap, IndexSet};

use crate::greedy_dag_extractor::{EgraphInfo, Extractor};

type EffectfulNodes = IndexMap<ClassId, IndexSet<*const Expr>>;

struct Linearity {
    effectful_nodes: EffectfulNodes,
    expr_to_term: IndexMap<*const Expr, Term>,
    n2c: IndexMap<NodeId, ClassId>,
}

impl<'a> Extractor<'a> {
    /// Finds all the effectful nodes along the state
    /// edge path (the path of the state edge from the argument to the return value).
    /// Input: a term representing the program
    /// Output: a map from root ids to the set of effectful nodes along the state edge path in this region
    pub fn find_effectful_nodes_in_function(
        &mut self,
        func: &RcExpr,
        egraph_info: &EgraphInfo,
    ) -> IndexMap<ClassId, IndexSet<NodeId>> {
        let mut expr_to_term = IndexMap::new();
        for (term, expr) in &self.term_to_expr {
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

        self.find_effectful_nodes_in_region(func.func_body().unwrap(), &mut linearity);

        let effectful_nodes: IndexMap<ClassId, IndexSet<NodeId>> = linearity
            .effectful_nodes
            .into_iter()
            .map(|(k, v)| {
                let v = v
                    .into_iter()
                    .map(|expr| self.term_node(linearity.expr_to_term.get(&expr).unwrap()))
                    .collect();
                (k, v)
            })
            .collect();

        // assert that we only find one node per eclass (otherwise the extractor is incorrect)
        for nodes in effectful_nodes.values() {
            let mut eclasses = IndexSet::new();
            for node in nodes {
                assert!(eclasses.insert(egraph_info.egraph.nid_to_cid(node)));
            }
        }

        effectful_nodes
    }

    fn class_of_expr(&self, linearity: &Linearity, expr: &RcExpr) -> ClassId {
        let term = linearity.expr_to_term.get(&Rc::as_ptr(expr)).unwrap();
        let nodeid = self.term_node(term);
        linearity.n2c.get(&nodeid).unwrap().clone()
    }

    /// Start finding effectful nodes from the root of the region
    /// If we've already visited this region, we should not visit again
    /// (otherwise, we may pick two paths to the same region, which is unsound)
    fn find_effectful_nodes_in_region(&mut self, expr: &RcExpr, linearity: &mut Linearity) {
        let rootid = self.class_of_expr(linearity, expr);
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
                TernaryOp::Select => {
                    panic!("Select is not effectful")
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
            Expr::Symbolic(_, _ty) => panic!("found symbolic"),
        }
    }
}

/// Check that a program is linear in its use of state.
#[allow(dead_code)]
pub fn check_program_is_linear(prog: &TreeProgram) -> Result<(), String> {
    for func in &prog.functions {
        check_function_is_linear(func, prog)?;
    }
    check_function_is_linear(&prog.entry, prog)
}

pub fn check_function_is_linear(fun: &RcExpr, prog_for_types: &TreeProgram) -> Result<(), String> {
    let mut reachables: IndexMap<*const Expr, IndexSet<*const Expr>> = Default::default();
    let mut raw_to_rc: IndexMap<*const Expr, RcExpr> = Default::default();
    let fun_body = fun.func_body().unwrap();
    fun_body.collect_reachable(fun_body, &mut reachables, &mut raw_to_rc);
    let mut tc = TypeChecker::new(prog_for_types, true);
    // Precompute effectfulness for every reachable expr exactly once to avoid mutable borrow conflicts.
    let mut effectful_cache: IndexMap<*const Expr, bool> = IndexMap::new();
    for (_region, exprs) in &reachables {
        for &ptr in exprs {
            if !effectful_cache.contains_key(&ptr) {
                let rcexpr = raw_to_rc.get(&ptr).unwrap();
                let ty: Type = tc.add_arg_types_to_expr(rcexpr.clone(), &None).0;
                effectful_cache.insert(ptr, ty.contains_state());
            }
        }
    }
    // Helper returning RcExpr if effectful (special casing Function to look at its body)
    let get_if_effectful = |expr_ptr: *const Expr| -> Option<&RcExpr> {
        let rcexpr = raw_to_rc.get(&expr_ptr).unwrap();
        if let Expr::Function(_n, _i, _o, body) = rcexpr.as_ref() {
            if effectful_cache
                .get(&Rc::as_ptr(body))
                .copied()
                .unwrap_or(false)
            {
                return Some(rcexpr);
            }
        } else if effectful_cache.get(&expr_ptr).copied().unwrap_or(false) {
            return Some(rcexpr);
        }
        None
    };
    for (region, exprs) in reachables {
        // consume map
        let mut dangling_effectful: HashSet<*const Expr> = HashSet::new();
        for &e in &exprs {
            if get_if_effectful(e).is_some() {
                dangling_effectful.insert(e);
            }
        }
        let mut effectful_parent: IndexMap<*const Expr, RcExpr> = Default::default();
        for expr_ptr in exprs {
            if let Some(expr) = get_if_effectful(expr_ptr) {
                if !matches!(expr.as_ref(), Expr::Arg(..)) {
                    let children = expr.children_same_scope();
                    let mut first: Option<RcExpr> = None;
                    let mut second = false;
                    for c in &children {
                        let cptr = Rc::as_ptr(c);
                        if effectful_cache.get(&cptr).copied().unwrap_or(false) {
                            if first.is_none() {
                                first = Some(c.clone());
                            } else {
                                second = true;
                                break;
                            }
                        }
                    }
                    let effectful_child = match first {
                        Some(c) if !second => c,
                        _ => panic!("Effectful operator child cardinality issue"),
                    };
                    let child_ptr = Rc::as_ptr(&effectful_child);
                    if !dangling_effectful.remove(&child_ptr) {
                        return Err(format!("Resulting program violated linearity! Effectful expression's state edge was referenced twice. Parent 1: {}\n\n Parent 2: {}\n\n Child referenced twice: {}", effectful_parent.get(&child_ptr).unwrap(), expr, effectful_child));
                    }
                    effectful_parent.insert(child_ptr, expr.clone());
                }
            }
        }
        if get_if_effectful(region).is_some() && !dangling_effectful.remove(&region) {
            panic!("The region operator is either consumed or not effectful.");
        }
        if !dangling_effectful.is_empty() {
            return Err(
                "Resulting program violated linearity! There are unconsumed effectful operators."
                    .to_string(),
            );
        }
    }
    Ok(())
}

impl Expr {
    /// Populates the reachable_from table, which is a map from the root of the region
    /// to the set of reachable nodes, stored as raw pointers.
    /// Additionally return a map from raw pointers to RcExpr.
    pub fn collect_reachable(
        self: &RcExpr,
        root: &RcExpr,
        reachable_from: &mut IndexMap<*const Expr, IndexSet<*const Expr>>,
        raw_to_rc: &mut IndexMap<*const Expr, RcExpr>,
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
