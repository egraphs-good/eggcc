//! Greedy bottom-up extraction with *bag* costs.
//!
//! A bag cost is the sum of the costs of the distinct e-classes a term uses,
//! so shared subterms are only paid for once. It is cheap to maintain but not
//! guaranteed optimal, which is fine for an estimate.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::hash::BuildHasherDefault;

use indexmap::IndexMap;
use rustc_hash::FxHasher;

use crate::cost::{enode_cost, if_branches_cost, loop_body_cost, Cost, INFINITE};
use crate::egraph::{
    EClassId, EGraph, EGraphMapping, ENode, ENodeId, ExtractedNode, Extraction, ExtractionId,
};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

/// The cost of a term as a bag of the e-classes it uses.
#[derive(Clone, Debug)]
pub struct BagCost {
    /// Sum of `bag`, plus the term's own e-node costs.
    pub sum: Cost,
    /// Cheapest known cost of each e-class the term depends on.
    bag: FxIndexMap<EClassId, Cost>,
}

impl BagCost {
    pub fn new(own: Cost) -> Self {
        BagCost {
            sum: own,
            bag: FxIndexMap::default(),
        }
    }

    pub fn infinite() -> Self {
        BagCost::new(INFINITE)
    }

    /// Account for depending on `class`, whose chosen term costs `child`.
    pub fn add_child(&mut self, class: EClassId, child: &BagCost) {
        let mut overhead = child.sum;
        for (&cid, &c) in &child.bag {
            overhead -= c;
            self.add_class(cid, c);
        }
        self.add_class(class, overhead);
    }

    fn add_class(&mut self, class: EClassId, cost: Cost) {
        match self.bag.get_mut(&class) {
            Some(known) => {
                if *known > cost {
                    self.sum -= *known - cost;
                    *known = cost;
                }
            }
            None => {
                self.bag.insert(class, cost);
                self.sum += cost;
            }
        }
    }

    fn set_zero(&mut self) {
        self.sum = 0;
        self.bag.clear();
    }
}

impl PartialEq for BagCost {
    fn eq(&self, other: &Self) -> bool {
        self.sum == other.sum
    }
}

impl Eq for BagCost {}

impl PartialOrd for BagCost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BagCost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sum.cmp(&other.sum)
    }
}

/// Dijkstra-style state shared by the greedy passes: the best known cost and
/// pick per e-class, and a work heap ordered by cost (ties: larger id first).
struct Greedy<'g> {
    g: &'g EGraph,
    parents: Vec<Vec<(EClassId, ENodeId)>>,
    /// Children of each e-node whose e-class has not been settled yet.
    remaining: Vec<Vec<usize>>,
    best: Vec<BagCost>,
    pick: Vec<Option<ENodeId>>,
    heap: BinaryHeap<(Reverse<Cost>, EClassId)>,
}

impl<'g> Greedy<'g> {
    fn new(g: &'g EGraph) -> Self {
        let mut greedy = Greedy {
            g,
            parents: g.parents(),
            remaining: g.child_counts(),
            best: vec![BagCost::infinite(); g.len()],
            pick: vec![None; g.len()],
            heap: BinaryHeap::new(),
        };
        for (c, n) in g.enode_ids() {
            let enode = g.enode(c, n);
            if enode.is_leaf() {
                greedy.relax(c, n, BagCost::new(enode_cost(enode)));
            }
        }
        greedy
    }

    fn relax(&mut self, class: EClassId, node: ENodeId, cost: BagCost) {
        if cost < self.best[class] {
            self.heap.push((Reverse(cost.sum), class));
            self.best[class] = cost;
            self.pick[class] = Some(node);
        }
    }

    /// Pop the cheapest e-class. Returns `None` when the heap is empty, and
    /// skips entries made stale by a later, cheaper relaxation.
    fn pop(&mut self) -> Option<EClassId> {
        while let Some((Reverse(cost), class)) = self.heap.pop() {
            if cost == self.best[class].sum {
                return Some(class);
            }
        }
        None
    }

    fn pop_unchecked(&mut self) -> Option<(Cost, EClassId)> {
        self.heap.pop().map(|(Reverse(cost), class)| (cost, class))
    }

    /// Bag cost of `enode` given the current best costs of its children.
    fn plain_cost(&self, own: Cost, enode: &ENode) -> BagCost {
        let mut cost = BagCost::new(own);
        for &child in &enode.children {
            cost.add_child(child, &self.best[child]);
        }
        cost
    }

    fn picked(&self, class: EClassId) -> &'g ENode {
        self.g
            .enode(class, self.pick[class].expect("e-class has no pick"))
    }
}

/// For every e-class, the cheapest e-node and its bag cost, with control-flow
/// heuristics for `If` and `DoWhile`. Stops early once `root` is settled.
pub fn greedy_costs(g: &EGraph, root: Option<EClassId>) -> (Vec<Option<ENodeId>>, Vec<Cost>) {
    let mut greedy = Greedy::new(g);
    while let Some((cost, class)) = greedy.pop_unchecked() {
        if Some(class) == root {
            break;
        }
        if cost != greedy.best[class].sum {
            continue;
        }
        for idx in 0..greedy.parents[class].len() {
            let (pc, pn) = greedy.parents[class][idx];
            greedy.remaining[pc][pn] -= 1;
            if greedy.remaining[pc][pn] != 0 {
                continue;
            }
            let enode = g.enode(pc, pn);
            let cost = match enode.op.as_str() {
                "If" => {
                    debug_assert!(enode.children.len() == 4);
                    let [pred, inputs, then_, else_] = enode.children[..] else {
                        unreachable!()
                    };
                    let mut cost = BagCost::new(enode_cost(enode));
                    cost.add_child(pred, &greedy.best[pred]);
                    cost.add_child(inputs, &greedy.best[inputs]);
                    cost.sum += if_branches_cost(greedy.best[then_].sum, greedy.best[else_].sum);
                    cost
                }
                "DoWhile" => {
                    debug_assert!(enode.children.len() == 2);
                    let [inputs, body] = enode.children[..] else {
                        unreachable!()
                    };
                    let mut cost = greedy.best[inputs].clone();
                    cost.sum += loop_body_cost(greedy.best[body].sum);
                    cost
                }
                _ => greedy.plain_cost(enode_cost(enode), enode),
            };
            greedy.relax(pc, pn, cost);
        }
    }
    let costs = greedy.best.iter().map(|b| b.sum).collect();
    (greedy.pick, costs)
}

/// Estimated cost of every e-class.
pub fn estimate_class_costs(g: &EGraph) -> Vec<Cost> {
    greedy_costs(g, None).1
}

/// Cost of an effectful e-node for the statewalk DP: its own cost plus the
/// estimated cost of its pure children (effectful children are paid for by the
/// rest of the statewalk).
fn statewalk_enode_cost(g: &EGraph, class_cost: &[Cost], enode: &ENode) -> Cost {
    let pure_child_cost = |c: EClassId| if g.is_effectful(c) { 0 } else { class_cost[c] };
    match enode.op.as_str() {
        "If" => {
            debug_assert!(enode.children.len() == 4);
            let [pred, inputs, then_, else_] = enode.children[..] else {
                unreachable!()
            };
            enode_cost(enode)
                + pure_child_cost(pred)
                + pure_child_cost(inputs)
                + if_branches_cost(class_cost[then_], class_cost[else_])
        }
        "DoWhile" => {
            debug_assert!(enode.children.len() == 2);
            loop_body_cost(class_cost[enode.children[1]])
        }
        _ => {
            enode_cost(enode)
                + enode
                    .children
                    .iter()
                    .map(|&c| pure_child_cost(c))
                    .sum::<Cost>()
        }
    }
}

/// Statewalk costs for every effectful e-node (pure e-classes get an empty row).
pub fn statewalk_costs(g: &EGraph) -> Vec<Vec<Cost>> {
    let class_cost = estimate_class_costs(g);
    g.classes
        .iter()
        .map(|class| {
            if class.is_effectful {
                class
                    .enodes
                    .iter()
                    .map(|n| statewalk_enode_cost(g, &class_cost, n))
                    .collect()
            } else {
                Vec::new()
            }
        })
        .collect()
}

/// Re-index statewalk costs of the target e-graph for the source of `mapping`.
pub fn project_statewalk_costs(mapping: &EGraphMapping, costs: &[Vec<Cost>]) -> Vec<Vec<Cost>> {
    mapping
        .enodes
        .iter()
        .enumerate()
        .map(|(c, nodes)| {
            let tc = mapping.class(c);
            if costs[tc].is_empty() {
                Vec::new()
            } else {
                nodes
                    .iter()
                    .map(|n| costs[tc][n.expect("e-node is not mapped")])
                    .collect()
            }
        })
        .collect()
}

/// Greedily extract `root` from a *linearized* e-graph, in which every
/// effectful e-class has exactly one e-node (its statewalk pick).
///
/// Pure e-classes are settled cheapest first. Whenever an effectful e-class is
/// settled, the whole term below it is emitted and its e-classes become free
/// for everyone else to reuse (their cost drops to zero), which is what makes
/// the shared statewalk pay for pure subterms only once.
pub fn statewalk_greedy_extraction(g: &EGraph, root: EClassId) -> Extraction {
    let mut greedy = Greedy::new(g);
    let mut extraction: Extraction = Vec::new();
    let mut extracted: Vec<Option<ExtractionId>> = vec![None; g.len()];
    let mut processed = vec![false; g.len()];
    let mut member_index: Vec<Option<usize>> = vec![None; g.len()];

    while let Some(class) = greedy.pop() {
        if g.is_effectful(class) {
            emit_term(
                &mut greedy,
                class,
                &mut extraction,
                &mut extracted,
                &mut member_index,
            );
        }
        if class == root {
            break;
        }
        for idx in 0..greedy.parents[class].len() {
            let (pc, pn) = greedy.parents[class][idx];
            // A class can be popped again after its cost was zeroed; only count
            // it towards its parents once.
            if !processed[class] {
                greedy.remaining[pc][pn] -= 1;
            }
            if greedy.remaining[pc][pn] != 0 {
                continue;
            }
            let enode = g.enode(pc, pn);
            let own = if g.is_effectful(pc) {
                0
            } else {
                enode_cost(enode)
            };
            let cost = greedy.plain_cost(own, enode);
            greedy.relax(pc, pn, cost);
        }
        processed[class] = true;
    }
    debug_assert!(extracted[root].is_some());
    debug_assert!(crate::checks::is_effect_safe(g, root, &extraction));
    extraction
}

/// Append the picked term below `class` to `extraction` (skipping e-classes
/// already extracted), children before parents, then zero the cost of every
/// e-class it covers.
fn emit_term(
    greedy: &mut Greedy,
    class: EClassId,
    extraction: &mut Extraction,
    extracted: &mut [Option<ExtractionId>],
    member_index: &mut [Option<usize>],
) {
    let g = greedy.g;
    // Members: e-classes of the picked term that are not yet extracted, in BFS order.
    let mut members: Vec<EClassId> = vec![class];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new()];
    let mut pending: Vec<usize> = vec![0];
    member_index[class] = Some(0);
    let mut i = 0;
    while i < members.len() {
        for &child in &greedy.picked(members[i]).children {
            if extracted[child].is_none() {
                let ci = *member_index[child].get_or_insert_with(|| {
                    members.push(child);
                    dependents.push(Vec::new());
                    pending.push(0);
                    members.len() - 1
                });
                dependents[ci].push(i);
                pending[i] += 1;
            }
        }
        i += 1;
    }
    // Topological order (Kahn), children first; the root `class` comes out last.
    let mut order: Vec<usize> = (0..members.len()).filter(|&m| pending[m] == 0).collect();
    let mut i = 0;
    while i < order.len() {
        for &parent in &dependents[order[i]] {
            pending[parent] -= 1;
            if pending[parent] == 0 {
                order.push(parent);
            }
        }
        i += 1;
    }
    debug_assert!(order.len() == members.len());
    for &m in &order {
        let u = members[m];
        let node = greedy.pick[u].expect("member has a pick");
        let children = g
            .enode(u, node)
            .children
            .iter()
            .map(|&c| extracted[c].expect("children are emitted first"))
            .collect();
        extracted[u] = Some(extraction.len());
        extraction.push(ExtractedNode {
            class: u,
            node,
            children,
        });
    }
    for &u in &members {
        member_index[u] = None;
        if greedy.best[u].sum > 0 {
            greedy.best[u].set_zero();
            if u != class {
                greedy.heap.push((Reverse(0), u));
            }
        }
    }
}
