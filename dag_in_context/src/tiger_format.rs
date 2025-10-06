use egraph_serialize::{ClassId, EGraph, NodeId};
use indexmap::{IndexMap, IndexSet};

/// Simplified "tiger" view of an e-graph used by the prototype extractor.
///
/// The original C++ prototype (tiger.cpp) uses three structs:
///   ENode { head, eclass, ch }
///   EClass { enodes, isEffectful }
///   EGraph { eclasses }
///
/// Here we reproduce those in Rust, backed by the serialized e-graph
/// produced by `egraph_serialize` so that we can incrementally port
/// the prototype algorithms.
#[derive(Debug, Clone)]
pub struct TigerENode {
    pub head: String,
    pub eclass_idx: usize,
    pub children: Vec<usize>,
    pub original_class: ClassId,
    pub original_node: NodeId,
}

#[derive(Debug, Clone)]
pub struct TigerEClass {
    pub enodes: Vec<TigerENode>,
    pub is_effectful: bool,
    /// Original ClassId.
    pub original: ClassId,
}

#[derive(Debug, Clone)]
pub struct TigerEGraph {
    pub eclasses: Vec<TigerEClass>,
    /// Map original ClassId -> tiger index.
    pub class_index: IndexMap<ClassId, usize>,
}

impl TigerEGraph {
    pub fn num_eclasses(&self) -> usize {
        self.eclasses.len()
    }
}

/// Default (very lightweight) effectfulness heuristic.
/// For now we mark an eclass effectful iff its type string (if any)
/// contains the substring "State". This mirrors the intent of the
/// C++ prototype which propagated effectful types; we will refine
/// this once the typechecker-based path is wired in.
pub fn default_is_effectful(egraph: &EGraph, cid: &ClassId) -> bool {
    egraph
        .class_data
        .get(cid)
        .and_then(|d| d.typ.as_ref())
        .map(|t| t.contains("State"))
        .unwrap_or(false)
}

/// Perform a simple fixpoint over type nodes to discover effectful types, then
/// mark expression nodes whose children reference an effectful type.
pub fn analyze_effectful_expr_eclasses(egraph: &EGraph) -> IndexSet<ClassId> {
    // 1. Identify type eclasses and build adjacency (type parent -> type children)
    let mut type_children: IndexMap<ClassId, IndexSet<ClassId>> = IndexMap::new();
    let mut all_type_classes: IndexSet<ClassId> = IndexSet::new();
    for (cid, class) in egraph.classes() {
        let sort = egraph.class_data[cid].typ.as_deref().unwrap_or("");
        if sort == "Type" {
            all_type_classes.insert(cid.clone());
        }
        if sort == "Type" {
            let mut kids = IndexSet::new();
            for nid in &class.nodes {
                let node = &egraph[nid];
                for ch in &node.children {
                    let cc = egraph.nid_to_cid(ch).clone();
                    kids.insert(cc);
                }
            }
            type_children.insert(cid.clone(), kids);
        }
    }
    // 2. Seed effectful types: any type eclass containing an op with substring "State"
    let mut is_effectful_type: IndexSet<ClassId> = IndexSet::new();
    for cid in &all_type_classes {
        let class = &egraph.classes()[cid];
        if class
            .nodes
            .iter()
            .any(|nid| egraph[nid].op.contains("State"))
        {
            is_effectful_type.insert(cid.clone());
        }
    }
    // 3. Propagate: if a type has any child effectful type => it is effectful
    let mut changed = true;
    while changed {
        changed = false;
        for cid in &all_type_classes {
            if is_effectful_type.contains(cid) {
                continue;
            }
            if let Some(kids) = type_children.get(cid) {
                if kids.iter().any(|k| is_effectful_type.contains(k)) {
                    is_effectful_type.insert(cid.clone());
                    changed = true;
                }
            }
        }
    }
    // 4. Mark expression eclasses effectful if they refer to an effectful type child.
    let mut effectful_exprs: IndexSet<ClassId> = IndexSet::new();
    for (cid, class) in egraph.classes() {
        let sort = egraph.class_data[cid].typ.as_deref().unwrap_or("");
        if sort == "Expr" {
            'outer: for nid in &class.nodes {
                let node = &egraph[nid];
                for ch in &node.children {
                    let cc = egraph.nid_to_cid(ch);
                    if is_effectful_type.contains(cc) {
                        effectful_exprs.insert(cid.clone());
                        break 'outer;
                    }
                }
            }
        }
    }
    effectful_exprs
}

/// Build the tiger representation from a serialized egraph using a custom predicate
/// to decide whether an eclass is effectful.
pub fn build_tiger_egraph_with<F>(egraph: &EGraph, mut is_effectful: F) -> TigerEGraph
where
    F: FnMut(&EGraph, &ClassId) -> bool,
{
    // Pre-compute improved effectful expression set
    let improved = analyze_effectful_expr_eclasses(egraph);
    // Stable ordering: iterate over the classes map in its insertion order
    // (IndexMap in the serializer preserves determinism) and assign
    // contiguous indices 0..n just like the C++ structure expects.
    let mut class_index: IndexMap<ClassId, usize> = IndexMap::new();
    for cid in egraph.classes().keys() {
        let next = class_index.len();
        class_index.insert(cid.clone(), next);
    }

    // Pre-build child class index lookups for speed.
    let mut tiger_eclasses: Vec<TigerEClass> = Vec::with_capacity(class_index.len());
    tiger_eclasses.resize_with(class_index.len(), || TigerEClass {
        enodes: Vec::new(),
        is_effectful: false,
        original: ClassId::from("__uninit__".to_string()),
    });

    // First pass: fill eclasses with meta & effectful flag.
    for (cid, &idx) in &class_index {
        let eff = if improved.contains(cid) {
            true
        } else {
            is_effectful(egraph, cid)
        };
        tiger_eclasses[idx].is_effectful = eff;
        tiger_eclasses[idx].original = cid.clone();
    }

    // Second pass: create tiger enodes.
    for (cid, class) in egraph.classes() {
        let eclass_idx = class_index[cid];
        for node_id in &class.nodes {
            let node = &egraph[node_id];
            let children: Vec<usize> = node
                .children
                .iter()
                .map(|child_nid| {
                    let child_cid = egraph.nid_to_cid(child_nid);
                    class_index[child_cid]
                })
                .collect();
            let tiger_node = TigerENode {
                head: node.op.clone(),
                eclass_idx,
                children,
                original_class: cid.clone(),
                original_node: node_id.clone(),
            };
            tiger_eclasses[eclass_idx].enodes.push(tiger_node);
        }
    }

    // Ensure function definitions and their bodies are treated as effectful so that
    // reconstruction always considers them anchors, even when the body itself is pure.
    let mut function_classes: Vec<usize> = Vec::new();
    let mut function_bodies: Vec<usize> = Vec::new();
    for (ec_idx, ec) in tiger_eclasses.iter().enumerate() {
        let mut contains_function = false;
        for en in &ec.enodes {
            if en.head == "Function" {
                contains_function = true;
                if let Some(body_idx) = en.children.get(3) {
                    function_bodies.push(*body_idx);
                }
            }
        }
        if contains_function {
            function_classes.push(ec_idx);
        }
    }
    for idx in function_classes {
        tiger_eclasses[idx].is_effectful = true;
    }
    for idx in function_bodies {
        tiger_eclasses[idx].is_effectful = true;
    }

    TigerEGraph {
        eclasses: tiger_eclasses,
        class_index,
    }
}

/// Convenience wrapper using the default effectful predicate.
pub fn build_tiger_egraph(egraph: &EGraph) -> TigerEGraph {
    build_tiger_egraph_with(egraph, default_is_effectful)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_empty() {
        let eg = EGraph::default();
        let tg = build_tiger_egraph(&eg);
        assert_eq!(tg.num_eclasses(), 0);
        // Removed to_tiger_string assertion (function deleted) to satisfy clippy
        assert!(tg.eclasses.is_empty());
    }
}
