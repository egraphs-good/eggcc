use egglog::*;
use egraph_serialize::{ClassId, NodeId};
use indexmap::*;
use ordered_float::NotNan;
use rustc_hash::FxHashMap;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn serialized_egraph(
    egglog_egraph: egglog::EGraph,
) -> (egraph_serialize::EGraph, HashSet<String>) {
    let config = SerializeConfig::default();
    let mut egraph = egglog_egraph.serialize(config);
    let root_nodes = egraph
        .nodes
        .iter()
        .filter(|(_nid, node)| node.op == "Program");
    for (nid, _n) in root_nodes {
        egraph.root_eclasses.push(egraph.nid_to_cid(nid).clone());
    }
    let unextractables = egglog_egraph
        .functions
        .iter()
        .filter_map(|(name, func)| {
            if func.is_extractable() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect();
    (egraph, unextractables)
}

type Cost = NotNan<f64>;

pub struct CostSet {
    pub total: Cost,
    // TODO perhaps more efficient as
    // persistent data structure?
    pub costs: HashMap<ClassId, Cost>,
    pub term: Term,
}

fn build_parent_index(egraph: &egraph_serialize::EGraph) -> IndexMap<ClassId, Vec<NodeId>> {
    let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);

    for class in egraph.classes().values() {
        parents.insert(class.id.clone(), Vec::new());
    }

    for class in egraph.classes().values() {
        for node in &class.nodes {
            for c in &egraph[node].children {
                // compute parents of this enode
                parents[n2c(c)].push(node.clone());
            }
        }
    }
    parents
}

fn initialize_worklist(egraph: &egraph_serialize::EGraph) -> UniqueQueue<NodeId> {
    let mut analysis_pending = UniqueQueue::default();
    for class in egraph.classes().values() {
        for node in &class.nodes {
            // start the analysis from leaves
            if egraph[node].is_leaf() {
                analysis_pending.insert(node.clone());
            }
        }
    }
    analysis_pending
}

fn get_term(op: &str, cost_sets: &[&CostSet], termdag: &mut TermDag) -> Term {
    if cost_sets.is_empty() {
        if op.starts_with('\"') {
            return termdag.lit(ast::Literal::String(op[1..op.len() - 1].into()));
        }
        if let Ok(n) = op.parse::<i64>() {
            return termdag.lit(ast::Literal::Int(n));
        }
        if op == "true" {
            return termdag.lit(ast::Literal::Bool(true));
        }
        if op == "false" {
            return termdag.lit(ast::Literal::Bool(false));
        }
    }
    termdag.app(
        op.into(),
        cost_sets.iter().map(|cs| cs.term.clone()).collect(),
    )
}

/// Given an operator, eclass, and cost sets for children eclasses,
/// calculate the new cost set for this operator.
/// This is done by unioning the child costs sets and summing them up,
/// except for special cases like regions.
fn get_node_cost(
    op: &str,
    cid: &ClassId,
    // non-empty cost sets for children eclasses
    child_cost_sets: &[&CostSet],
    cm: &impl CostModel,
    termdag: &mut TermDag,
) -> CostSet {
    let mut total = cm.get_op_cost(op);
    let mut costs = HashMap::from([(cid.clone(), total)]);
    let term = get_term(op, child_cost_sets, termdag);

    let unshared_children = cm.unshared_children(op);
    if !cm.ignore_children(op) {
        for (i, child_set) in child_cost_sets.iter().enumerate() {
            if unshared_children.contains(&i) {
                // don't add to the cost set, but do add to the total
                total += child_set.total;
            } else {
                for (child_cid, child_cost) in &child_set.costs {
                    // it was already present in the set
                    if let Some(existing) = costs.insert(child_cid.clone(), *child_cost) {
                        assert_eq!(
                            existing, *child_cost,
                            "Two different costs found for the same child enode!"
                        );
                    } else {
                        total += child_cost;
                    }
                }
            }
        }
    }

    CostSet { total, costs, term }
}

fn calculate_cost_set(
    egraph: &egraph_serialize::EGraph,
    node_id: NodeId,
    costs: &FxHashMap<ClassId, CostSet>,
    termdag: &mut TermDag,
    cm: &impl CostModel,
) -> CostSet {
    let node = &egraph[&node_id];
    let cid = egraph.nid_to_cid(&node_id);

    // early return
    if node.children.is_empty() {
        return get_node_cost(&node.op, cid, &[], cm, termdag);
    }

    let children_classes = node
        .children
        .iter()
        .map(|c| egraph.nid_to_cid(c).clone())
        .collect::<Vec<ClassId>>();

    if children_classes.contains(cid) {
        // Shortcut. Can't be cheaper so return junk.
        return CostSet {
            costs: Default::default(),
            total: std::f64::INFINITY.try_into().unwrap(),
            // returns junk children since this cost set is guaranteed to not be selected.
            term: termdag.app(node.op.as_str().into(), vec![]),
        };
    }

    let cost_sets: Vec<_> = children_classes
        .iter()
        .map(|c| costs.get(c).unwrap())
        .collect();

    // cycle detection
    if cost_sets.iter().any(|cs| cs.costs.contains_key(cid)) {
        return CostSet {
            costs: Default::default(),
            total: std::f64::INFINITY.try_into().unwrap(),
            // returns junk children since this cost set is guaranteed to not be selected.
            term: termdag.app(node.op.as_str().into(), vec![]),
        };
    }

    get_node_cost(&node.op, cid, &cost_sets, cm, termdag)
}

pub fn extract(
    egraph: &egraph_serialize::EGraph,
    // TODO: once our egglog program uses `subsume` actions,
    // unextractables will be more complex, as right now
    // it only checks unextractable at the function level.
    unextractables: HashSet<String>,
    termdag: &mut TermDag,
    cm: impl CostModel,
) -> HashMap<ClassId, CostSet> {
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
    let parents = build_parent_index(egraph);
    let mut worklist = initialize_worklist(egraph);
    let mut costs = FxHashMap::<ClassId, CostSet>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    while let Some(node_id) = worklist.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        if unextractables.contains(&node.op) {
            continue;
        }
        if node.children.iter().all(|c| costs.contains_key(n2c(c))) {
            let lookup = costs.get(class_id);
            let mut prev_cost: Cost = std::f64::INFINITY.try_into().unwrap();
            if lookup.is_some() {
                prev_cost = lookup.unwrap().total;
            }

            let cost_set = calculate_cost_set(egraph, node_id.clone(), &costs, termdag, &cm);
            if cost_set.total < prev_cost {
                costs.insert(class_id.clone(), cost_set);
                worklist.extend(parents[class_id].iter().cloned());
            }
        }
    }

    let mut root_eclasses = egraph.root_eclasses.clone();
    root_eclasses.sort();
    root_eclasses.dedup();

    root_eclasses
        .iter()
        .map(|cid| (cid.clone(), costs.remove(cid).unwrap()))
        .collect()
}

pub trait CostModel {
    // TODO: we could do better with type info
    fn get_op_cost(&self, op: &str) -> Cost;

    // if true, the op's children are ignored
    fn ignore_children(&self, op: &str) -> bool;

    // returns a slice of indices into the children vec
    fn unshared_children(&self, op: &str) -> &'static [usize];
}

pub struct DefaultCostModel;

impl CostModel for DefaultCostModel {
    fn get_op_cost(&self, op: &str) -> Cost {
        match op {
            // Leaves
            "Const" => 1.,
            "Arg" => 0.,
            _ if op.parse::<i64>().is_ok() || op.starts_with('"') => 0.,
            "true" | "false" | "()" => 0.,
            // Lists
            "Empty" | "Single" | "Concat" | "Get" | "Nil" | "Cons" => 0.,
            // Types
            "IntT" | "BoolT" | "PointerT" | "StateT" => 0.,
            "Base" | "TupleT" | "TNil" | "TCons" => 0.,
            "Int" | "Bool" => 0.,
            // Algebra
            "Add" | "PtrAdd" | "Sub" | "And" | "Or" | "Not" => 10.,
            "Mul" => 30.,
            "Div" => 50.,
            // Comparisons
            "Eq" | "LessThan" | "GreaterThan" | "LessEq" | "GreaterEq" => 10.,
            // Effects
            "Print" | "Write" | "Load" => 50.,
            "Alloc" | "Free" => 100.,
            "Call" => 1000., // TODO: we could make this more accurate
            // Control
            "Program" | "Function" => 1.,
            "DoWhile" => 100., // TODO: we could make this more accurate
            "If" | "Switch" => 50.,
            // Unreachable
            "HasType" | "HasArgType" | "ContextOf" | "NoContext" | "ExpectType" => 0.,
            "ExprIsPure" | "ListExprIsPure" | "BinaryOpIsPure" | "UnaryOpIsPure" => 0.,
            "IsLeaf" | "BodyContainsExpr" | "ScopeContext" => 0.,
            "Region" | "Full" | "IntI" | "BoolI" => 0.,
            // Schema
            "Bop" | "Uop" | "Top" => 0.,
            "InContext" => 0.,
            _ if self.ignore_children(op) => 0.,
            _ => panic!("no cost for {op}"),
        }
        .try_into()
        .unwrap()
    }

    fn ignore_children(&self, op: &str) -> bool {
        matches!(op, "InLoop" | "NoContext" | "InSwitch" | "InIf")
    }

    fn unshared_children(&self, op: &str) -> &'static [usize] {
        match op {
            "DoWhile" => &[1],
            "Function" => &[3],
            "If" => &[2, 3],
            "Switch" => &[2], // TODO: Switch branches can share nodes
            _ => &[],
        }
    }
}

/** A data structure to maintain a queue of unique elements.

Notably, insert/pop operations have O(1) expected amortized runtime complexity.

Thanks @Bastacyclop for the implementation!
*/
#[derive(Clone)]
pub(crate) struct UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    set: HashSet<T>,
    queue: VecDeque<T>,
}

impl<T> Default for UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn default() -> Self {
        UniqueQueue {
            set: Default::default(),
            queue: Default::default(),
        }
    }
}

impl<T> UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn insert(&mut self, t: T) {
        if self.set.insert(t.clone()) {
            self.queue.push_back(t);
        }
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for t in iter.into_iter() {
            self.insert(t);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let res = self.queue.pop_front();
        res.as_ref().map(|t| self.set.remove(t));
        res
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        let r = self.queue.is_empty();
        debug_assert_eq!(r, self.set.is_empty());
        r
    }
}
