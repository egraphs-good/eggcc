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

use rpds::HashTrieSet;

fn set_intersect(
    a: &HashTrieSet<*const Expr>,
    b: &HashTrieSet<*const Expr>,
) -> HashTrieSet<*const Expr> {
    let mut res = HashTrieSet::new();
    for e in a.iter() {
        if b.contains(e) {
            res = res.insert(*e);
        }
    }
    res
}

fn set_union(
    a: &HashTrieSet<*const Expr>,
    b: &HashTrieSet<*const Expr>,
) -> HashTrieSet<*const Expr> {
    let mut res = a.clone();
    for e in b.iter() {
        res = res.insert(*e);
    }
    res
}

#[derive(Debug, Default)]
pub(crate) struct AlwaysExecutedCache {
    // for a given expression e, a set nodes that are always executed
    // regardless of branching
    always_executed: HashMap<*const Expr, HashTrieSet<*const Expr>>,
}

impl AlwaysExecutedCache {
    /// Given a conditional expression, finds all children that are executed regardless of
    /// the branch taken.
    /// Once a node is added to the result, children of that node are not added (since any node it
    /// depends on will be executed).
    pub(crate) fn always_executed(
        &mut self,
        conditional_expr: &RcExpr,
        region_root: &RcExpr,
    ) -> HashMap<*const Expr, RcExpr> {
        let children = match conditional_expr.as_ref() {
            Expr::If(_, then_branch, else_branch) => vec![then_branch.clone(), else_branch.clone()],
            Expr::Switch(_, branches) => branches.clone(),
            _ => unreachable!(),
        };

        let mut to_execute = HashTrieSet::new();
        // We execute anything executed in all branches
        for child in &children {
            to_execute = set_intersect(&to_execute, &self.get(child));
        }
        // Also anything definitely executed by the root node
        to_execute = set_union(&to_execute, &self.get(region_root));

        let mut stack = children;
        let mut result = HashMap::new();
        let mut processed = HashSet::new();
        while let Some(expr) = stack.pop() {
            if processed.contains(&Rc::as_ptr(&expr)) {
                continue;
            }
            processed.insert(Rc::as_ptr(&expr));
            if to_execute.contains(&Rc::as_ptr(&expr)) {
                result.insert(Rc::as_ptr(&expr), expr.clone());
            } else {
                let children = expr.children_same_scope();
                for child in children {
                    stack.push(child);
                }
            }
        }

        result
    }

    pub(crate) fn get(&self, expr: &RcExpr) -> HashTrieSet<*const Expr> {
        if let Some(set) = self.always_executed.get(&Rc::as_ptr(expr)) {
            set.clone()
        } else {
            match expr.as_ref() {
                Expr::If(pred, then_branch, else_branc) => {
                    let mut res = self.get(pred).insert(Rc::as_ptr(expr));
                    let then_set = self.get(then_branch);
                    let else_set = self.get(else_branc);
                    let intersection = set_intersect(&then_set, &else_set);
                    res = set_union(&res, &intersection);
                    res
                }
                Expr::Switch(pred, branches) => {
                    let mut res = self.get(pred).insert(Rc::as_ptr(expr));
                    let branch_sets = branches.iter().map(|e| self.get(e));
                    let branches_intersection =
                        branch_sets.fold(HashTrieSet::new(), |acc, x| set_intersect(&acc, &x));
                    res = set_union(&res, &branches_intersection);
                    res
                }
                _ => {
                    let children = expr.children_same_scope();
                    let mut res = HashTrieSet::new();
                    for (i, child) in children.iter().enumerate() {
                        if i == 0 {
                            res = self.get(child);
                        } else {
                            res = set_union(&res, &self.get(child));
                        }
                    }
                    res = res.insert(Rc::as_ptr(expr));
                    res
                }
            }
        }
    }
}

#[cfg(test)]
fn rcexpr_set(iterator: impl IntoIterator<Item = RcExpr>) -> HashMap<*const Expr, RcExpr> {
    iterator
        .into_iter()
        .map(|e| (Rc::as_ptr(&e), e.clone()))
        .collect()
}

#[test]
fn test_simple_branch_inputs() {
    use dag_in_context::ast::*;
    let my_if = tif(ttrue(), int(1), int(2));
    let outside_computation = add(int(3), int(4));
    let root = add(my_if.clone(), outside_computation.clone());
    let mut always_cache = AlwaysExecutedCache::default();
    assert_eq!(
        always_cache.always_executed(&my_if, &root),
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
    assert_eq!(always_cache.always_executed(&my_if, &root), expected);
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
    assert_eq!(always_cache.always_executed(&my_if, &root), expected);
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
    assert_eq!(always_cache.always_executed(&my_if, &root), expected);
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
    let mut always_cache = AlwaysExecutedCache::default();

    eprintln!("{:?}", always_cache.get(&root).iter().collect::<Vec<_>>());

    let expected = rcexpr_set(vec![addr.clone(), shared_write.clone()]);

    assert_eq!(always_cache.always_executed(&root, &root), expected);
}
