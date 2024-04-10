//! This file is a helper for translation from the dag IR to RVSDGs.
//! It contains the `AlwaysExecutedCache` struct, which is used to
//! query nodes that are guaranteed to be executed given that a particular node is executed.
//! This information is used by `from_dag.rs` to compute the input nodes to branch regions.

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use dag_in_context::schema::{Expr, RcExpr};

#[derive(Debug, Default)]
pub(crate) struct AlwaysExecutedCache {
    // for a given expression e, a set of nodes that are always executed
    // regardless of branching
    always_executed: HashMap<*const Expr, HashSet<*const Expr>>,
}

impl AlwaysExecutedCache {
    /// Given a conditional expression, finds all children that are executed regardless of
    /// the branch taken.
    /// Once a node is added to the result, children of that node are not added (since any node it
    /// depends on will be executed).
    pub(crate) fn get_without_subchildren_for_branch(
        &mut self,
        conditional_expr: &RcExpr,
        region_root: &RcExpr,
    ) -> Vec<RcExpr> {
        let children = match conditional_expr.as_ref() {
            Expr::If(_, then_branch, else_branch) => vec![then_branch.clone(), else_branch.clone()],
            Expr::Switch(_, branches) => branches.clone(),
            _ => unreachable!(),
        };

        // Find all nodes that are executed in all branches
        // It's important that this is done before removing subchildren, since we are intersecting
        // sets of nodes.
        let mut to_execute = self.get(&children[0]);
        // We execute anything executed in all branches, so perform set intersection
        for child in &children {
            to_execute = to_execute.intersection(&self.get(child)).cloned().collect();
        }

        // Optimization: also, always execute anything executed by the root node
        to_execute.extend(&self.get(region_root));

        // Now we want to find the subset of to_execute without any subchildren.
        let mut stack = children;
        let mut result = HashMap::new();
        let mut processed = HashSet::new();
        while let Some(expr) = stack.pop() {
            if processed.contains(&Rc::as_ptr(&expr)) {
                continue;
            }
            processed.insert(Rc::as_ptr(&expr));
            if to_execute.contains(&Rc::as_ptr(&expr)) {
                // don't recur into children
                result.insert(Rc::as_ptr(&expr), expr.clone());
            } else {
                let children = expr.children_same_scope();
                for child in children {
                    stack.push(child);
                }
            }
        }

        rcexpr_set(result.values().cloned())
    }

    /// Get the set of expressions that are always executed given this expression
    /// is executed, including itself.
    pub(crate) fn get(&self, expr: &RcExpr) -> HashSet<*const Expr> {
        if let Some(set) = self.always_executed.get(&Rc::as_ptr(expr)) {
            set.clone()
        } else {
            match expr.as_ref() {
                Expr::If(pred, then_branch, else_branc) => {
                    let mut res = self.get(pred);
                    res.insert(Rc::as_ptr(expr));
                    let then_set = self.get(then_branch);
                    let else_set = self.get(else_branc);
                    let intersection = then_set.intersection(&else_set);
                    res.extend(intersection);
                    res
                }
                Expr::Switch(pred, branches) => {
                    let mut res = self.get(pred);
                    res.insert(Rc::as_ptr(expr));
                    let branch_sets: Vec<HashSet<*const Expr>> =
                        branches.iter().map(|e| self.get(e)).collect();
                    let mut branches_intersection = branch_sets[0].clone();
                    for branch in &branch_sets[1..] {
                        branches_intersection = branches_intersection
                            .intersection(branch)
                            .cloned()
                            .collect();
                    }

                    res.extend(branches_intersection);
                    res
                }
                _ => {
                    let children = expr.children_same_scope();
                    let mut res = HashSet::new();
                    for (i, child) in children.iter().enumerate() {
                        if i == 0 {
                            // replace set for first iteration, which is more efficient
                            res = self.get(child);
                        } else {
                            res.extend(&self.get(child));
                        }
                    }
                    res.insert(Rc::as_ptr(expr));
                    res
                }
            }
        }
    }
}

fn rcexpr_set(iterator: impl IntoIterator<Item = RcExpr>) -> Vec<RcExpr> {
    let mut vec: Vec<RcExpr> = iterator.into_iter().collect();
    vec.sort();
    vec.dedup();
    vec
}

#[test]
fn test_simple_branch_inputs() {
    use dag_in_context::ast::*;
    let my_if = tif(ttrue(), int(1), int(2));
    let outside_computation = add(int(3), int(4));
    let root = add(my_if.clone(), outside_computation.clone());
    let mut always_cache = AlwaysExecutedCache::default();
    assert_eq!(
        always_cache.get_without_subchildren_for_branch(&my_if, &root),
        rcexpr_set(vec![])
    );
}

#[test]
fn test_simple_branch_inputs_share_between_branches() {
    use dag_in_context::ast::*;
    let shared_expr = int(1);
    let my_if = tif(ttrue(), shared_expr.clone(), shared_expr.clone());
    let outside_computation = add(int(3), int(4));
    let root = add(my_if.clone(), outside_computation.clone());
    let mut always_cache = AlwaysExecutedCache::default();
    let expected = rcexpr_set(vec![shared_expr.clone()]);
    assert_eq!(
        always_cache.get_without_subchildren_for_branch(&my_if, &root),
        expected
    );
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
    let mut always_cache = AlwaysExecutedCache::default();
    let expected = rcexpr_set(vec![shared_expr.clone()]);
    assert_eq!(
        always_cache.get_without_subchildren_for_branch(&my_if, &root),
        expected
    );
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
    let mut always_cache = AlwaysExecutedCache::default();
    let expected = rcexpr_set(vec![shared_expr]);
    assert_eq!(
        always_cache.get_without_subchildren_for_branch(&my_if, &root),
        expected
    );
}

#[test]
fn test_branch_share_effects() {
    use dag_in_context::ast::*;
    let addr = alloc(0, int(10), arg(), pointert(intt()));
    let shared_read = load(get(addr.clone(), 0), get(addr.clone(), 1));
    let shared_write = write(get(addr.clone(), 0), int(20), get(shared_read.clone(), 1));

    let root = tif(
        ttrue(),
        write(get(addr.clone(), 0), int(30), shared_write.clone()),
        write(get(addr.clone(), 0), int(40), shared_write.clone()),
    );
    let mut always_cache = AlwaysExecutedCache::default();

    let expected = rcexpr_set(vec![addr.clone(), shared_write.clone()]);

    assert_eq!(
        always_cache.get_without_subchildren_for_branch(&root, &root),
        expected
    );
}
