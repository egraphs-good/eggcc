//! This file is a helper for translation from the dag IR to RVSDGs.
//! It contains the `RegionGraph` struct, which is used to create a dependency graph
//! for a region (a loop or a function body).
//! Using the graph, we can compute a dominance frontier for if and switch statement
//! branches.
//! Using the dominance frontier, we decide which nodes need to be computed
//! in the resulting region for a branch.

use std::rc::Rc;

use hashbrown::{HashMap, HashSet};
use petgraph::graph::{DiGraph, NodeIndex};
use tree_in_context::schema::{Expr, RcExpr};

struct RegionGraph {
    graph: DiGraph<(), ()>,
    expr_to_node: HashMap<*const Expr, NodeIndex>,
    node_to_expr: HashMap<NodeIndex, *const Expr>,
    /// For each branch node, we need an extra node in the graph.
    /// This allows us to query for the nodes dominated by the branch.
    expr_branch_node: HashMap<(*const Expr, usize), NodeIndex>,
}

/// In the DAG IR, there are two nodes that create new "regions"
/// by binding an argument: Function and DoWhile.
/// This function creates a dependency graph of all the computations for a given region
/// (it doesn't traverse into nested regions).
pub(crate) fn region_graph(expr: &RcExpr) -> RegionGraph {
    let mut todo = vec![expr.clone()];
    let mut processed = HashSet::<*const Expr>::new();
    let mut rgraph = RegionGraph {
        graph: DiGraph::new(),
        expr_to_node: HashMap::new(),
        node_to_expr: HashMap::new(),
        expr_branch_node: HashMap::new(),
    };
    while let Some(expr) = todo.pop() {
        if !processed.insert(Rc::as_ptr(&expr)) {
            continue;
        }
        // for if or switch statements, we need to create branch nodes
        match expr.as_ref() {
            Expr::If(inputs, then_branch, else_branch) => {
                let then_branch_node = rgraph.graph.add_node(());
                rgraph
                    .expr_branch_node
                    .insert((Rc::as_ptr(&expr), 0), then_branch_node);
                let else_branch_node = rgraph.graph.add_node(());
                rgraph
                    .expr_branch_node
                    .insert((Rc::as_ptr(&expr), 1), else_branch_node);
                let expr_node = rgraph.node(&expr);
                let inputs_node = rgraph.node(inputs);
                let then_node = rgraph.node(then_branch);
                let else_node = rgraph.node(else_branch);

                // direct edge to inputs
                rgraph.graph.add_edge(expr_node, inputs_node, ());
                // edges to newly made branch nodes
                rgraph.graph.add_edge(expr_node, then_branch_node, ());
                rgraph.graph.add_edge(expr_node, else_branch_node, ());
                // branch nodes point to actual branch expressions
                rgraph.graph.add_edge(then_branch_node, then_node, ());
                rgraph.graph.add_edge(else_branch_node, else_node, ());
                todo.push(inputs.clone());
                todo.push(then_branch.clone());
                todo.push(else_branch.clone());
            }
            _ => {
                // for loops, don't recur into subregions
                let children = match expr.as_ref() {
                    Expr::DoWhile(inputs, _body) => {
                        vec![inputs.clone()]
                    }
                    _ => expr.children(),
                };

                let expr_node = rgraph.node(&expr);
                for child in children {
                    let child_node = rgraph.node(&child);
                    rgraph.graph.add_edge(expr_node, child_node, ());
                    todo.push(child);
                }
            }
        }
    }
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
                self.node_to_expr.insert(new_node, Rc::as_ptr(expr));
                new_node
            }
        }
    }
}
