use egglog::{ast::Symbol, *};
use egraph_serialize::{ClassId, NodeId};
use indexmap::*;
use ordered_float::NotNan;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{HashMap, HashSet};

pub fn serialized_egraph(egraph: egglog::EGraph) -> egraph_serialize::EGraph {
    let config = SerializeConfig::default();
    let mut egraph = egraph.serialize(config);
    let root_nodes = egraph
        .nodes
        .iter()
        .filter(|(_nid, node)| node.op == "Program");
    for (nid, _n) in root_nodes {
        egraph.root_eclasses.push(egraph.nid_to_cid(nid).clone());
    }
    egraph
}

type Cost = NotNan<f64>;

pub struct CostSet {
    pub total: Cost,
    // TODO this would be more efficient as a
    // persistent data structure
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

fn get_node_cost(node: &egraph_serialize::Node) -> Cost {
    todo!()
}

fn calculate_cost_set(
    egraph: &egraph_serialize::EGraph,
    node_id: NodeId,
    costs: &FxHashMap<ClassId, CostSet>,
    best_cost: Cost,
    termdag: &mut TermDag,
    cm: &CostModel,
) -> CostSet {
    let node = &egraph[&node_id];
    let cid = egraph.nid_to_cid(&node_id);

    if node.children.is_empty() || cm.ignored.contains(node.op.as_str()) {
        let cost = get_node_cost(node);
        return CostSet {
            costs: HashMap::from([(cid.clone(), cost)]),
            total: cost,
            term: termdag.app(node.op.as_str().into(), vec![]),
        };
    }

    let childrens_classes = node
        .children
        .iter()
        .map(|c| egraph.nid_to_cid(&c).clone())
        .collect::<Vec<ClassId>>();

    if childrens_classes.contains(cid) {
        // Shortcut. Can't be cheaper so return junk.
        return CostSet {
            costs: Default::default(),
            total: std::f64::INFINITY.try_into().unwrap(),
            // returns junk children since this cost set is guaranteed to not be selected.
            term: termdag.app(node.op.as_str().into(), vec![]),
        };
    }

    // Clone the biggest set and insert the others into it.
    let id_of_biggest = childrens_classes
        .iter()
        .max_by_key(|s| costs.get(s).unwrap().costs.len())
        .unwrap();
    let mut result = costs.get(&id_of_biggest).unwrap().costs.clone();
    for child_cid in &childrens_classes {
        if child_cid == id_of_biggest {
            continue;
        }

        let next_cost = &costs.get(child_cid).unwrap().costs;
        for (key, value) in next_cost.iter() {
            result.insert(key.clone(), value.clone());
        }
    }

    let contains = result.contains_key(&cid);
    result.insert(cid.clone(), node.cost);

    let result_cost = if contains {
        std::f64::INFINITY.try_into().unwrap()
    } else {
        // TODO: result values
        // TODO: move the cost aggregation part to calculate_cost_set
        result.values().sum()
    };

    let child_terms = childrens_classes
        .iter()
        .map(|c| costs.get(c).unwrap().term.clone())
        .collect::<Vec<Term>>();

    return CostSet {
        costs: result,
        total: result_cost,
        term: termdag.app(node.op.as_str().into(), child_terms),
    };
}

pub fn extract(egraph: &egraph_serialize::EGraph, cm: &CostModel) -> HashMap<ClassId, CostSet> {
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
    let parents = build_parent_index(&egraph);
    let mut worklist = initialize_worklist(&egraph);
    let mut termdag = TermDag::default();
    let mut costs = FxHashMap::<ClassId, CostSet>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    while let Some(node_id) = worklist.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        if node.children.iter().all(|c| costs.contains_key(n2c(c))) {
            let lookup = costs.get(class_id);
            let mut prev_cost: Cost = std::f64::INFINITY.try_into().unwrap();
            if lookup.is_some() {
                prev_cost = lookup.unwrap().total;
            }

            let cost_set =
                calculate_cost_set(egraph, node_id.clone(), &costs, prev_cost, &mut termdag, cm);
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

struct CostModel {
    ops: HashMap<&'static str, Cost>,
    // Children of these constructors are ignored
    ignored: HashSet<&'static str>,
}

impl CostModel {
    fn simple_cost_model() -> CostModel {
        let ops = vec![
            // ========== Leaf operators ==========
            // Bop
            // TODO: actually we also need type info
            // to figure out the cost
            ("Add", 1.),
            ("Sub", 1.),
            ("Mul", 1.),
            ("Div", 1.),
            ("Eq", 1.),
            ("LessThan", 1.),
            ("GreaterThan", 1.),
            ("LessEq", 1.),
            ("GreaterEq", 1.),
            ("And", 1.),
            ("Or", 1.),
            ("PtrAdd", 1.),
            ("Print", 1.),
            ("Load", 1.),
            ("Free", 1.),
            // Uop
            ("Not", 1.),
            // Top
            ("Write", 1.),
            // Order
            ("Parallel", 0.),
            ("Sequential", 0.),
            ("Reversed", 0.),
            // ========== Non-leaf operators ==========
            ("Alloc", 100.),
            // TODO: The cost of Call is more complicated than that.
            // Call
            ("Call", 10.),
        ];
        let ignored = HashSet::from(["InLoop", "InFunc", "InSwitch", "InIf"]);
        let ops: HashMap<_, _> = ops
            .into_iter()
            .map(|(op, cost)| (op, NotNan::new(cost).unwrap()))
            .collect();
        CostModel { ops, ignored }
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
    queue: std::collections::VecDeque<T>,
}

impl<T> Default for UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn default() -> Self {
        UniqueQueue {
            set: Default::default(),
            queue: std::collections::VecDeque::new(),
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
