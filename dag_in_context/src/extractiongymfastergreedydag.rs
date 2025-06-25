// Calculates the cost where shared nodes are just costed once,
// For example (+ (* x x ) (* x x )) has one mulitplication
// included in the cost.

use std::collections::HashMap;

use crate::fastercbcextractor::ExtractionResult;

use super::*;
use egraph_serialize::*;
use ordered_float::NotNan;
use rustc_hash::{FxHashMap, FxHashSet};

struct CostSet {
    // It's slightly faster if this is an HashMap rather than an fxHashMap.
    costs: HashMap<ClassId, Cost>,
    total: Cost,
    choice: NodeId,
}

pub struct FasterGreedyDagExtractor;

impl FasterGreedyDagExtractor {
    fn calculate_cost_set(
        egraph: &EGraph,
        node_id: NodeId,
        costs: &FxHashMap<ClassId, CostSet>,
        best_cost: Cost,
    ) -> CostSet {
        let node = &egraph[&node_id];
        let cid = egraph.nid_to_cid(&node_id);

        if node.children.is_empty() {
            return CostSet {
                costs: HashMap::from([(cid.clone(), node.cost)]),
                total: node.cost,
                choice: node_id.clone(),
            };
        }

        // Get unique classes of children.
        let mut childrens_classes = node
            .children
            .iter()
            .map(|c| egraph.nid_to_cid(c).clone())
            .collect::<Vec<ClassId>>();
        childrens_classes.sort();
        childrens_classes.dedup();

        let first_cost = costs.get(&childrens_classes[0]).unwrap();

        if childrens_classes.contains(cid)
            || (childrens_classes.len() == 1 && (node.cost + first_cost.total > best_cost))
        {
            // Shortcut. Can't be cheaper so return junk.
            return CostSet {
                costs: Default::default(),
                total: NotNan::new(f64::INFINITY).unwrap(),
                choice: node_id.clone(),
            };
        }

        // Clone the biggest set and insert the others into it.
        let id_of_biggest = childrens_classes
            .iter()
            .max_by_key(|s| costs.get(s).unwrap().costs.len())
            .unwrap();
        let mut result = costs.get(id_of_biggest).unwrap().costs.clone();
        for child_cid in &childrens_classes {
            if child_cid == id_of_biggest {
                continue;
            }

            let next_cost = &costs.get(child_cid).unwrap().costs;
            for (key, value) in next_cost.iter() {
                result.insert(key.clone(), *value);
            }
        }

        let contains = result.contains_key(cid);
        result.insert(cid.clone(), node.cost);

        let result_cost = if contains {
            NotNan::new(f64::INFINITY).unwrap()
        } else {
            result.values().sum()
        };

        CostSet {
            costs: result,
            total: result_cost,
            choice: node_id.clone(),
        }
    }
}

impl FasterGreedyDagExtractor {
    pub fn extract(&self, egraph: &EGraph, _roots: &[ClassId]) -> ExtractionResult {
        let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
        let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
        let mut analysis_pending = UniqueQueue::default();

        for class in egraph.classes().values() {
            parents.insert(class.id.clone(), Vec::new());
        }

        for class in egraph.classes().values() {
            for node in &class.nodes {
                for c in &egraph[node].children {
                    // compute parents of this enode
                    parents[n2c(c)].push(node.clone());
                }

                // start the analysis from leaves
                if egraph[node].is_leaf() {
                    analysis_pending.insert(node.clone());
                }
            }
        }

        let mut result = ExtractionResult::default();
        let mut costs = FxHashMap::<ClassId, CostSet>::with_capacity_and_hasher(
            egraph.classes().len(),
            Default::default(),
        );

        while let Some(node_id) = analysis_pending.pop() {
            let class_id = n2c(&node_id);
            let node = &egraph[&node_id];
            if node.children.iter().all(|c| costs.contains_key(n2c(c))) {
                let lookup = costs.get(class_id);
                let mut prev_cost = NotNan::new(f64::INFINITY).unwrap();
                if lookup.is_some() {
                    prev_cost = lookup.unwrap().total;
                }

                let cost_set = Self::calculate_cost_set(egraph, node_id.clone(), &costs, prev_cost);
                if cost_set.total < prev_cost {
                    costs.insert(class_id.clone(), cost_set);
                    analysis_pending.extend(parents[class_id].iter().cloned());
                }
            }
        }

        for (cid, cost_set) in costs {
            result.choose(cid, cost_set.choice);
        }

        result
    }
}

/** A data structure to maintain a queue of unique elements.

Notably, insert/pop operations have O(1) expected amortized runtime complexity.

Thanks @Bastacyclop for the implementation!
*/
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    set: FxHashSet<T>, // hashbrown::
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
