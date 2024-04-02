//! This file is a helper for translation from the dag IR to RVSDGs.
//! It contains the `RegionGraph` struct, which is used to create a dependency graph
//! for a region (a loop or a function body).
//! Using the graph, we can compute a dominance frontier for if and switch statement
//! branches.
//! Using the dominance frontier, we decide which nodes need to be computed
//! in the resulting region for a branch.

use std::rc::Rc;

use dag_in_context::schema::{Expr, RcExpr};
use hashbrown::{HashMap, HashSet};
use petgraph::{
    algo::dominators::{self, Dominators},
    graph::{DiGraph, NodeIndex},
};

pub(crate) struct RegionGraph {
    graph: DiGraph<(), ()>,
    expr_to_node: HashMap<*const Expr, NodeIndex>,
    node_to_expr: HashMap<NodeIndex, Rc<Expr>>,
    /// For each branch node, we add root node for the branch to the graph.
    /// This handles the edge case where two branches share a common subexpression, and so
    /// the dominance frontier contains just that subexpression.
    expr_branch_node: HashMap<(*const Expr, usize), NodeIndex>,
    /// Dominators for the graph. This is None until the graph is fully constructed.
    dominators: Option<Dominators<NodeIndex>>,
}

/// In the DAG IR, there are two nodes that create new "regions"
/// by binding an argument: Function and DoWhile.
/// `expr`` should be the body of the function or the loop body.
/// This function creates a dependency graph of all the computations for a given region
/// (it doesn't traverse into nested regions).
pub(crate) fn region_graph(expr: &RcExpr) -> RegionGraph {
    let mut dfs_stack = vec![expr.clone()];
    let mut processed = HashSet::<*const Expr>::new();
    let mut rgraph = RegionGraph {
        graph: DiGraph::new(),
        expr_to_node: HashMap::new(),
        node_to_expr: HashMap::new(),
        expr_branch_node: HashMap::new(),
        // dummy dominators, will be replaced later
        dominators: None,
    };
    while let Some(expr) = dfs_stack.pop() {
        if !processed.insert(Rc::as_ptr(&expr)) {
            continue;
        }
        // for `If` or `Switch`` statements, we need to create branch nodes
        match expr.as_ref() {
            Expr::If(inputs, then_branch, else_branch) => {
                let then_root_node = rgraph.graph.add_node(());
                rgraph
                    .expr_branch_node
                    .insert((Rc::as_ptr(&expr), 0), then_root_node);
                let else_root_node = rgraph.graph.add_node(());
                rgraph
                    .expr_branch_node
                    .insert((Rc::as_ptr(&expr), 1), else_root_node);
                let expr_node = rgraph.node(&expr);
                let inputs_node = rgraph.node(inputs);
                let then_node = rgraph.node(then_branch);
                let else_node = rgraph.node(else_branch);

                // direct edge to inputs
                rgraph.graph.add_edge(expr_node, inputs_node, ());
                // edges to newly made branch nodes
                rgraph.graph.add_edge(expr_node, then_root_node, ());
                rgraph.graph.add_edge(expr_node, else_root_node, ());
                // branch nodes point to actual branch expressions
                rgraph.graph.add_edge(then_root_node, then_node, ());
                rgraph.graph.add_edge(else_root_node, else_node, ());

                dfs_stack.push(else_branch.clone());
                dfs_stack.push(then_branch.clone());
                dfs_stack.push(inputs.clone());
            }
            Expr::Switch(inputs, branches) => {
                let expr_node = rgraph.node(&expr);
                for (i, branch) in branches.iter().enumerate() {
                    let branch_root = rgraph.graph.add_node(());
                    rgraph
                        .expr_branch_node
                        .insert((Rc::as_ptr(&expr), i), branch_root);
                    let branch_node = rgraph.node(branch);
                    rgraph.graph.add_edge(expr_node, branch_root, ());
                    rgraph.graph.add_edge(branch_root, branch_node, ());
                    dfs_stack.push(branch.clone());
                }
                let inputs_node = rgraph.node(inputs);
                rgraph.graph.add_edge(expr_node, inputs_node, ());
                dfs_stack.push(inputs.clone());
            }
            _ => {
                // for loops, don't recur into subregions
                let children = expr.children_same_scope();

                let expr_node = rgraph.node(&expr);
                for child in children {
                    let child_node = rgraph.node(&child);
                    rgraph.graph.add_edge(expr_node, child_node, ());
                    dfs_stack.push(child);
                }
            }
        }
    }

    let root = rgraph.node(expr);
    rgraph.dominators = Some(dominators::simple_fast(&rgraph.graph, root));
    rgraph
}

impl RegionGraph {
    /// Make a new node, or return an existing one.
    pub(crate) fn node(&mut self, expr: &RcExpr) -> NodeIndex {
        match self.expr_to_node.get(&Rc::as_ptr(expr)) {
            Some(node) => *node,
            None => {
                let new_node = self.graph.add_node(());
                self.expr_to_node.insert(Rc::as_ptr(expr), new_node);
                self.node_to_expr.insert(new_node, expr.clone());
                new_node
            }
        }
    }

    /// Return the expressions dominated by this branch.
    /// Expressions that are in this set should be only evaluated in the branch.
    /// Expressions that have a child that is not in the set
    /// are along the dominance frontier.
    fn dominated_by(&self, expr: &RcExpr, branch: usize) -> HashMap<*const Expr, RcExpr> {
        let branch_node = self.expr_branch_node[&(Rc::as_ptr(expr), branch)];
        let mut result = HashMap::new();
        let mut todo = vec![branch_node];
        while let Some(node) = todo.pop() {
            // if this node is not a branch node, it corresponds to an expression
            if let Some(expr) = self.node_to_expr.get(&node) {
                result.insert(Rc::as_ptr(expr), expr.clone());
            }

            for child in self
                .dominators
                .as_ref()
                .unwrap()
                .immediately_dominated_by(node)
            {
                todo.push(child);
            }
        }

        result
    }

    /// For a given branch, find all the expressions that need to be passed in
    /// as arguments to the region.
    /// The argument is always passed through, so it is not included in the result.
    pub(crate) fn branch_inputs(
        &self,
        expr: &RcExpr,
        branch: usize,
    ) -> HashMap<*const Expr, RcExpr> {
        let dominated_exprs = self.dominated_by(expr, branch);

        let mut result = HashMap::new();

        // when there are no dominated exprs, the branch expression
        // is the only one that needs to be passed through
        if dominated_exprs.is_empty() {
            let branch_node = self.expr_branch_node[&(Rc::as_ptr(expr), branch)];
            let branch_node_child = self.graph.neighbors(branch_node).next().unwrap();
            let branch_expr = self.node_to_expr[&branch_node_child].clone();
            result.insert(Rc::as_ptr(&branch_expr), branch_expr);
        }

        for (_expr_ptr, expr) in dominated_exprs.iter() {
            for child in expr.children_same_scope() {
                // if the child is not dominated by the branch, it needs to be passed through
                if dominated_exprs.get(&Rc::as_ptr(&child)).is_none() {
                    match child.as_ref() {
                        // unless it is an argument
                        Expr::Arg(_) => {}
                        _ => {
                            result.insert(Rc::as_ptr(&child), child.clone());
                        }
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
fn rcexpr_set(iterator: impl IntoIterator<Item = RcExpr>) -> HashMap<*const Expr, RcExpr> {
    iterator.into_iter().map(|e| (Rc::as_ptr(&e), e)).collect()
}

#[test]
fn test_simple_branch_inputs() {
    use dag_in_context::ast::*;
    let my_if = tif(ttrue(), int(1), int(2));
    let outside_computation = add(int(3), int(4));
    let root = add(my_if.clone(), outside_computation.clone());
    let rgraph = region_graph(&root);
    assert_eq!(rgraph.branch_inputs(&my_if, 0), rcexpr_set(vec![]));
    assert_eq!(rgraph.branch_inputs(&my_if, 1), rcexpr_set(vec![]));
}

#[test]
fn test_simple_branch_inputs_share_between_branches() {
    use dag_in_context::ast::*;
    let shared_expr = int(1);
    let my_if = tif(ttrue(), shared_expr.clone(), shared_expr.clone());
    let outside_computation = add(int(3), int(4));
    let root = add(my_if.clone(), outside_computation.clone());
    let rgraph = region_graph(&root);
    let expected = rcexpr_set(vec![shared_expr.clone()]);
    assert_eq!(rgraph.branch_inputs(&my_if, 0), expected);
    assert_eq!(rgraph.branch_inputs(&my_if, 1), expected);
}

#[test]
fn test_simple_branch_inputs_share_between_branches2() {
    use dag_in_context::ast::*;
    let shared_expr = int(1);
    let my_if = tif(
        ttrue(),
        add(shared_expr.clone(), shared_expr.clone()),
        add(shared_expr.clone(), int(10)),
    );
    let outside_computation = add(int(3), int(4));
    let root = add(my_if.clone(), outside_computation.clone());
    let rgraph = region_graph(&root);
    let expected = rcexpr_set(vec![shared_expr.clone()]);
    assert_eq!(rgraph.branch_inputs(&my_if, 0), expected);
    assert_eq!(rgraph.branch_inputs(&my_if, 1), expected);
}

#[test]
fn test_simple_branch_share_outside() {
    use dag_in_context::ast::*;
    let shared_expr = int(1);
    let my_if = tif(
        ttrue(),
        add(shared_expr.clone(), int(9)),
        add(int(10), int(11)),
    );
    let outside_computation = add(shared_expr.clone(), int(4));
    let root = add(my_if.clone(), outside_computation.clone());
    let rgraph = region_graph(&root);
    let expected = rcexpr_set(vec![shared_expr.clone()]);
    let expected2 = rcexpr_set(vec![]);
    assert_eq!(rgraph.branch_inputs(&my_if, 0), expected);
    assert_eq!(rgraph.branch_inputs(&my_if, 1), expected2);
}

#[test]
fn test_branch_share_effects() {
    use dag_in_context::ast::*;
    let addr = alloc(int(10), arg(), pointert(intt()));
    let shared_read = load(get(addr.clone(), 0), get(addr.clone(), 1));
    let shared_write = write(get(addr.clone(), 0), int(20), get(shared_read.clone(), 1));

    let root = tif(
        ttrue(),
        write(get(addr.clone(), 0), int(30), shared_write.clone()),
        write(get(addr.clone(), 0), int(40), shared_write.clone()),
    );

    let rgraph = region_graph(&root);
    let expected = rcexpr_set(vec![addr.clone(), shared_write.clone()]);

    assert_eq!(rgraph.branch_inputs(&root, 0), expected);
    assert_eq!(rgraph.branch_inputs(&root, 1), expected);
}
