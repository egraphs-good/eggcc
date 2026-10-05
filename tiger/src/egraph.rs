//! The e-graph representation the extractor works on.
//!
//! This is a plain, index-based e-graph: e-classes are numbered densely and
//! every e-node lives in exactly one e-class. An e-class is *effectful* when
//! its terms carry the program state (eggcc's `StateT`); the effectful e-nodes
//! chosen for a region form its *statewalk*. Everything else is pure.

use std::collections::VecDeque;

use egglog::ast::Literal;

pub type EClassId = usize;
pub type ENodeId = usize;
/// Position of an e-node inside an [`Extraction`].
pub type ExtractionId = usize;

#[derive(Clone, Debug, Default)]
pub struct ENode {
    /// Constructor name, or the literal's text for primitive leaves.
    pub op: String,
    /// `Some` for primitive leaves (ints, bools, floats, strings).
    pub lit: Option<Literal>,
    pub children: Vec<EClassId>,
}

impl ENode {
    pub fn is_primitive(&self) -> bool {
        self.lit.is_some()
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct EClass {
    pub enodes: Vec<ENode>,
    pub is_effectful: bool,
}

#[derive(Clone, Debug, Default)]
pub struct EGraph {
    pub classes: Vec<EClass>,
}

impl EGraph {
    pub fn len(&self) -> usize {
        self.classes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    pub fn class_ids(&self) -> std::ops::Range<EClassId> {
        0..self.len()
    }

    pub fn enode(&self, class: EClassId, node: ENodeId) -> &ENode {
        &self.classes[class].enodes[node]
    }

    pub fn is_effectful(&self, class: EClassId) -> bool {
        self.classes[class].is_effectful
    }

    /// Every `(class, node)` pair, in order.
    pub fn enode_ids(&self) -> impl Iterator<Item = (EClassId, ENodeId)> + '_ {
        self.classes
            .iter()
            .enumerate()
            .flat_map(|(c, class)| (0..class.enodes.len()).map(move |n| (c, n)))
    }

    /// The first effectful child of `enode`, if it has one.
    pub fn effectful_child(&self, enode: &ENode) -> Option<EClassId> {
        enode
            .children
            .iter()
            .copied()
            .find(|&c| self.is_effectful(c))
    }

    /// For each e-class, the e-nodes that have it as a child (with multiplicity).
    pub fn parents(&self) -> Vec<Vec<(EClassId, ENodeId)>> {
        let mut parents = vec![Vec::new(); self.len()];
        for (c, n) in self.enode_ids() {
            for &child in &self.enode(c, n).children {
                parents[child].push((c, n));
            }
        }
        parents
    }

    /// The number of children of every e-node, indexed like the e-graph.
    pub fn child_counts(&self) -> Vec<Vec<usize>> {
        self.classes
            .iter()
            .map(|class| class.enodes.iter().map(|n| n.children.len()).collect())
            .collect()
    }

    /// Drop e-nodes that cannot be part of any finite term and e-classes that
    /// are unreachable from `root` (every e-class is kept when `root` is `None`).
    /// Returns the pruned e-graph and the mapping from this e-graph into it.
    pub fn prune_unextractable(&self, root: Option<EClassId>) -> (EGraph, EGraphMapping) {
        // An e-node is extractable once all its children are; an e-class once one of its e-nodes is.
        let parents = self.parents();
        let mut remaining = self.child_counts();
        let mut extractable = vec![false; self.len()];
        let mut queue = VecDeque::new();
        for (c, n) in self.enode_ids() {
            if remaining[c][n] == 0 && !extractable[c] {
                extractable[c] = true;
                queue.push_back(c);
            }
        }
        while let Some(u) = queue.pop_front() {
            for &(pc, pn) in &parents[u] {
                remaining[pc][pn] -= 1;
                if remaining[pc][pn] == 0 && !extractable[pc] {
                    extractable[pc] = true;
                    queue.push_back(pc);
                }
            }
        }

        // Reachability from the root through extractable e-nodes.
        let mut reachable = vec![root.is_none(); self.len()];
        if let Some(root) = root {
            reachable[root] = true;
            queue.push_back(root);
            while let Some(u) = queue.pop_front() {
                for enode in &self.classes[u].enodes {
                    if enode.children.iter().all(|&v| extractable[v]) {
                        for &v in &enode.children {
                            if !reachable[v] {
                                reachable[v] = true;
                                queue.push_back(v);
                            }
                        }
                    }
                }
            }
        }

        let mut pruned = EGraph::default();
        let mut mapping = EGraphMapping::unmapped(self);
        for c in self.class_ids() {
            if reachable[c] && extractable[c] {
                mapping.classes[c] = Some(pruned.len());
                pruned.classes.push(EClass {
                    enodes: Vec::new(),
                    is_effectful: self.is_effectful(c),
                });
            }
        }
        for c in self.class_ids() {
            let Some(target) = mapping.classes[c] else {
                continue;
            };
            for (n, enode) in self.classes[c].enodes.iter().enumerate() {
                let children: Option<Vec<EClassId>> =
                    enode.children.iter().map(|&v| mapping.classes[v]).collect();
                if let Some(children) = children {
                    let class = &mut pruned.classes[target];
                    mapping.enodes[c][n] = Some(class.enodes.len());
                    class.enodes.push(ENode {
                        op: enode.op.clone(),
                        lit: enode.lit.clone(),
                        children,
                    });
                }
            }
        }
        debug_assert!(crate::checks::is_wellformed(&pruned, false, true));
        debug_assert!(crate::checks::is_valid_mapping(
            &mapping, self, &pruned, true, true, true, true
        ));
        (pruned, mapping)
    }
}

/// One e-node of an extracted term DAG. Children refer to earlier positions in
/// the [`Extraction`].
#[derive(Clone, Debug, Default)]
pub struct ExtractedNode {
    pub class: EClassId,
    pub node: ENodeId,
    pub children: Vec<ExtractionId>,
}

/// An extracted term DAG in topological order, children before parents. The
/// root is the last node.
pub type Extraction = Vec<ExtractedNode>;

/// A partial mapping from the e-classes and e-nodes of one e-graph (the
/// *source*) to those of another (the *target*).
#[derive(Clone, Debug, Default)]
pub struct EGraphMapping {
    pub classes: Vec<Option<EClassId>>,
    pub enodes: Vec<Vec<Option<ENodeId>>>,
}

impl EGraphMapping {
    /// A mapping with the shape of `source` that maps nothing yet.
    pub fn unmapped(source: &EGraph) -> Self {
        EGraphMapping {
            classes: vec![None; source.len()],
            enodes: source
                .classes
                .iter()
                .map(|class| vec![None; class.enodes.len()])
                .collect(),
        }
    }

    /// The identity mapping on `g`.
    pub fn identity(g: &EGraph) -> Self {
        EGraphMapping {
            classes: (0..g.len()).map(Some).collect(),
            enodes: g
                .classes
                .iter()
                .map(|class| (0..class.enodes.len()).map(Some).collect())
                .collect(),
        }
    }

    pub fn class(&self, class: EClassId) -> EClassId {
        self.classes[class].expect("e-class is not mapped")
    }

    pub fn enode(&self, class: EClassId, node: ENodeId) -> ENodeId {
        self.enodes[class][node].expect("e-node is not mapped")
    }

    /// The inverse mapping, from `target` back into the source.
    pub fn inverse(&self, target: &EGraph) -> EGraphMapping {
        let mut inv = EGraphMapping::unmapped(target);
        for (c, &mapped) in self.classes.iter().enumerate() {
            if let Some(tc) = mapped {
                inv.classes[tc] = Some(c);
                for (n, &mapped) in self.enodes[c].iter().enumerate() {
                    if let Some(tn) = mapped {
                        inv.enodes[tc][tn] = Some(n);
                    }
                }
            }
        }
        inv
    }

    /// `self` followed by `next`: a mapping from this source into `next`'s target.
    pub fn then(&self, next: &EGraphMapping) -> EGraphMapping {
        EGraphMapping {
            classes: self
                .classes
                .iter()
                .map(|c| c.and_then(|c| next.classes[c]))
                .collect(),
            enodes: self
                .enodes
                .iter()
                .enumerate()
                .map(|(c, nodes)| {
                    nodes
                        .iter()
                        .map(|n| match (self.classes[c], n) {
                            (Some(tc), Some(tn)) => next.enodes[tc][*tn],
                            _ => None,
                        })
                        .collect()
                })
                .collect(),
        }
    }

    /// Re-express an extraction over the source e-graph in terms of the target.
    pub fn apply(&self, extraction: &Extraction) -> Extraction {
        extraction
            .iter()
            .map(|en| ExtractedNode {
                class: self.class(en.class),
                node: self.enode(en.class, en.node),
                children: en.children.clone(),
            })
            .collect()
    }
}
