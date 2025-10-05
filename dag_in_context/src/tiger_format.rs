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
    /// An operator / head string (currently just the node op; the C++ version
    /// sometimes bakes in extra disambiguation info – we can extend later).
    pub head: String,
    /// The ID (index) of the containing class in the tiger vector (not the original ClassId).
    pub eclass_idx: usize,
    /// Children are indices of tiger eclasses (not original ClassIds).
    pub children: Vec<usize>,
    /// Original class id for back reference.
    pub original_class: ClassId,
    /// Original node id (useful for mapping back into the serialized egraph).
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

/// Build the tiger representation from a serialized egraph using a custom predicate
/// to decide whether an eclass is effectful.
pub fn build_tiger_egraph_with<F>(egraph: &EGraph, mut is_effectful: F) -> TigerEGraph
where
    F: FnMut(&EGraph, &ClassId) -> bool,
{
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
        let eff = is_effectful(egraph, cid);
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

    TigerEGraph {
        eclasses: tiger_eclasses,
        class_index,
    }
}

/// Convenience wrapper using the default effectful predicate.
pub fn build_tiger_egraph(egraph: &EGraph) -> TigerEGraph {
    build_tiger_egraph_with(egraph, default_is_effectful)
}

/// Emit a textual form close to example.in / tiger.cpp expectations.
/// Format:
///   <num_eclasses>\n
///   For each eclass i:
///     # i\n
///     <effectful_flag> <num_enodes>\n
///     For each enode:
///        <head>\n
///        <k child0 child1 ...>  (children line)
pub fn to_tiger_string(tg: &TigerEGraph) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(&mut out, "{}", tg.eclasses.len()).unwrap();
    for (i, ec) in tg.eclasses.iter().enumerate() {
        writeln!(&mut out, "# {}", i).unwrap();
        writeln!(
            &mut out,
            "{} {}",
            if ec.is_effectful { 1 } else { 0 },
            ec.enodes.len()
        )
        .unwrap();
        for en in &ec.enodes {
            writeln!(&mut out, "{}", en.head).unwrap();
            write!(&mut out, "{}", en.children.len()).unwrap();
            for c in &en.children {
                write!(&mut out, " {}", c).unwrap();
            }
            writeln!(&mut out).unwrap();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_empty() {
        let eg = EGraph::default();
        let tg = build_tiger_egraph(&eg);
        assert_eq!(tg.num_eclasses(), 0);
        assert!(to_tiger_string(&tg).starts_with("0\n"));
    }
}
