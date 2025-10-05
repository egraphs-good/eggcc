#![allow(clippy::collapsible_if)]
use crate::greedy_dag_extractor::get_root; // used by function_body_root
use crate::tiger_format::{build_tiger_egraph, TigerEClass, TigerEGraph, TigerENode};
use egraph_serialize::{ClassId, EGraph};
use indexmap::{IndexMap, IndexSet};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::{HashMap, VecDeque};

// Global alias (was incorrectly inside impl causing unstable inherent associated type error)
pub type ExtractableSet = Vec<bool>;

#[derive(Debug, Clone)]
pub struct TigerRegion {
    /// Effectful anchor at start of this region segment
    pub anchor: ClassId,
    /// Optional next effectful anchor (None for last segment)
    pub next_anchor: Option<ClassId>,
    /// All eclasses (including anchor and maybe next anchor) in this region segment
    pub members: IndexSet<ClassId>,
}

#[derive(Debug, Clone)]
pub struct TigerRegionStats {
    pub total_enodes: usize,
    pub effectful_enodes: usize,
    pub pure_enodes: usize,
}

// --- Added full extraction data structures (early port) ---
#[derive(Debug, Clone)]
pub struct TigerExtractionENode {
    pub eclass: ClassId,
    /// index into the tiger.eclasses[tiger.class_index[eclass]].enodes vec
    pub enode_index: usize,
    pub children: Vec<usize>, // indices into extraction.nodes
}

#[derive(Debug, Clone, Default)]
pub struct TigerExtraction {
    pub nodes: Vec<TigerExtractionENode>,
    pub root_index: Option<usize>,
}

impl TigerExtraction {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            root_index: None,
        }
    }

    pub fn add_node(&mut self, node: TigerExtractionENode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }
}

/// A very early partial port of the prototype tiger extractor.
/// Currently it:
///  * Builds the tiger egraph structure
///  * Computes a longest effectual "state walk" per function body
///  * Partitions into region segments with per-region stats
///  * Builds a naive greedy extraction (pick first enode per eclass) rooted at the function body
///  * Runs a light linearity sanity check (placeholder)
///  * Emits debug info
pub struct TigerExtractor<'a> {
    serialized: &'a EGraph,
    tiger: TigerEGraph,
}

#[derive(Debug, Clone)]
/// Region-subgraph mapping produced by `create_region_egraph` (port of C++ createRegionEGraph).
pub struct RegionSubEGraph {
    /// Pruned / transformed egraph restricted to a region root and its reachable pure closure.
    pub egraph: TigerEGraph,
    /// Map from original ClassId -> region egraph index.
    pub orig_to_region: IndexMap<ClassId, usize>,
    /// Vector mapping region index -> original ClassId.
    pub region_to_orig: Vec<ClassId>,
    /// For each region eclass (outer index) and each of its enodes (inner index),
    /// number of additional effectful children beyond the first (aka subregion children).
    pub n_subregion: Vec<Vec<usize>>, // mirrors tiger.cpp nsubregion
}

impl RegionSubEGraph {
    pub fn size(&self) -> usize {
        self.egraph.eclasses.len()
    }
}

/// Build a region-restricted egraph starting at `region_root`.
/// Mirrors tiger.cpp createRegionEGraph:
///  1. First BFS over effectual backbone: include at most the first effectual child edge per enode;
///     count subsequent effectual children in n_subregion.
pub fn create_region_egraph(tiger: &TigerEGraph, region_root: &ClassId) -> RegionSubEGraph {
    let mut orig_to_region: IndexMap<ClassId, usize> = IndexMap::new();
    let mut region_to_orig: Vec<ClassId> = Vec::new();
    let mut n_subregion: Vec<Vec<usize>> = Vec::new();

    // Inline function logic without capturing mutable borrows simultaneously.
    fn ensure_mapping(
        tiger: &TigerEGraph,
        cid: &ClassId,
        orig_to_region: &mut IndexMap<ClassId, usize>,
        region_to_orig: &mut Vec<ClassId>,
        n_subregion: &mut Vec<Vec<usize>>,
    ) {
        if orig_to_region.contains_key(cid) {
            return;
        }
        if let Some(&t_idx) = tiger.class_index.get(cid) {
            let row_len = tiger.eclasses[t_idx].enodes.len();
            let new_idx = region_to_orig.len();
            region_to_orig.push(cid.clone());
            orig_to_region.insert(cid.clone(), new_idx);
            n_subregion.push(vec![0usize; row_len]);
        }
    }

    ensure_mapping(
        tiger,
        region_root,
        &mut orig_to_region,
        &mut region_to_orig,
        &mut n_subregion,
    );

    // 1. Effectful backbone expansion
    let mut idx = 0;
    while idx < region_to_orig.len() {
        let orig_cid = region_to_orig[idx].clone();
        let t_idx = match tiger.class_index.get(&orig_cid) {
            Some(v) => *v,
            None => {
                idx += 1;
                continue;
            }
        };
        let tec = &tiger.eclasses[t_idx];
        let r_idx = *orig_to_region.get(&orig_cid).unwrap();
        for (en_i, en) in tec.enodes.iter().enumerate() {
            let mut seen_effectful = false;
            for &child_t_idx in &en.children {
                let child_ec = &tiger.eclasses[child_t_idx];
                if child_ec.is_effectful {
                    if seen_effectful {
                        n_subregion[r_idx][en_i] += 1;
                        continue;
                    }
                    seen_effectful = true;
                    ensure_mapping(
                        tiger,
                        &child_ec.original,
                        &mut orig_to_region,
                        &mut region_to_orig,
                        &mut n_subregion,
                    );
                }
            }
        }
        idx += 1;
    }

    // 2. Pure closure expansion
    let mut idx2 = 0;
    while idx2 < region_to_orig.len() {
        let orig_cid = region_to_orig[idx2].clone();
        let t_idx = match tiger.class_index.get(&orig_cid) {
            Some(v) => *v,
            None => {
                idx2 += 1;
                continue;
            }
        };
        let tec = &tiger.eclasses[t_idx];
        for en in &tec.enodes {
            for &child_t_idx in &en.children {
                let child_ec = &tiger.eclasses[child_t_idx];
                if !child_ec.is_effectful {
                    ensure_mapping(
                        tiger,
                        &child_ec.original,
                        &mut orig_to_region,
                        &mut region_to_orig,
                        &mut n_subregion,
                    );
                }
            }
        }
        idx2 += 1;
    }

    // 3. Rebuild pruned TigerEGraph (preserving order of region_to_orig as indices)
    let mut region_eclasses: Vec<TigerEClass> = Vec::with_capacity(region_to_orig.len());
    for (r_idx, orig_cid) in region_to_orig.iter().enumerate() {
        let t_idx = match tiger.class_index.get(orig_cid) {
            Some(v) => *v,
            None => continue,
        };
        let orig_class = &tiger.eclasses[t_idx];
        let mut new_class = TigerEClass {
            enodes: Vec::new(),
            is_effectful: orig_class.is_effectful,
            original: orig_cid.clone(),
        };
        for en in &orig_class.enodes {
            let mut new_children = Vec::new();
            let mut seen_effectful = false;
            for &child_t_idx in &en.children {
                let child_ec = &tiger.eclasses[child_t_idx];
                let child_cid = child_ec.original.clone();
                if child_ec.is_effectful {
                    if seen_effectful {
                        continue;
                    } // prune additional effectful children in region graph
                    seen_effectful = true;
                }
                if let Some(&mapped_idx) = orig_to_region.get(&child_cid) {
                    new_children.push(mapped_idx);
                }
            }
            new_class.enodes.push(TigerENode {
                head: en.head.clone(),
                eclass_idx: r_idx,
                children: new_children,
                original_class: orig_cid.clone(),
                original_node: en.original_node.clone(),
            });
        }
        region_eclasses.push(new_class);
    }
    // Build class_index for region graph (region indices already contiguous)
    let mut region_class_index: IndexMap<ClassId, usize> = IndexMap::new();
    for (i, cid) in region_to_orig.iter().enumerate() {
        region_class_index.insert(cid.clone(), i);
    }
    let region_graph = TigerEGraph {
        eclasses: region_eclasses,
        class_index: region_class_index,
    };

    RegionSubEGraph {
        egraph: region_graph,
        orig_to_region,
        region_to_orig,
        n_subregion,
    }
}

pub struct TigerExtractionResult {
    pub chosen_enodes: IndexMap<ClassId, usize>, // eclass -> enode index
    pub state_walks: IndexMap<ClassId, Vec<ClassId>>, // function root class -> linear effectual path
    pub regions: IndexMap<ClassId, Vec<TigerRegion>>, // function root body -> region segments
    pub region_stats: IndexMap<ClassId, Vec<TigerRegionStats>>, // parallel to regions
    pub extractions: IndexMap<ClassId, TigerExtraction>, // function root body -> extraction (naive)
    pub linearity_ok: IndexMap<ClassId, bool>,
    pub debug: String,
    // New: guided state walk with enode indices (if available)
    pub guided_state_walks: IndexMap<ClassId, Vec<(ClassId, usize)>>,
    // New: weak linearity excess (extra effectful eclasses beyond state walk)
    pub weak_linearity_excess: IndexMap<ClassId, usize>,
    // New: weak linearity violation flag observed during unguided search (wlcnt > 1)
    pub weak_linearity_violation: IndexMap<ClassId, bool>,
}

impl<'a> TigerExtractor<'a> {
    pub fn new(serialized: &'a EGraph) -> Self {
        let tiger = build_tiger_egraph(serialized);
        Self { serialized, tiger }
    }

    /// Build a naive state walk: starting from the function root's body eclass, follow
    /// the first effectual child repeatedly until no further effectual child exists.
    fn build_state_walk(&self, root_cid: ClassId) -> Vec<ClassId> {
        self.build_longest_state_walk(root_cid)
    }

    /// Find all effectful children of an eclass (unique set of child eclasses that are effectual)
    fn effectful_children(&self, cid: &ClassId) -> IndexSet<ClassId> {
        let mut res = IndexSet::new();
        if let Some(class) = self.serialized.classes().get(cid) {
            for nid in &class.nodes {
                let node = &self.serialized[nid];
                for ch in &node.children {
                    let cc = self.serialized.nid_to_cid(ch).clone();
                    if let Some(idx) = self.tiger.class_index.get(&cc) {
                        if self.tiger.eclasses[*idx].is_effectful {
                            res.insert(cc);
                        }
                    }
                }
            }
        }
        res
    }

    /// Depth-first search for the longest chain of effectual eclasses starting at root.
    /// Ties are broken lexicographically by the resulting sequence for determinism.
    fn build_longest_state_walk(&self, root: ClassId) -> Vec<ClassId> {
        let mut best: Vec<ClassId> = vec![];
        let mut stack: Vec<ClassId> = vec![];
        fn dfs(
            this: &TigerExtractor,
            cur: ClassId,
            stack: &mut Vec<ClassId>,
            best: &mut Vec<ClassId>,
            visited: &mut IndexSet<ClassId>,
        ) {
            stack.push(cur.clone());
            visited.insert(cur.clone());
            let children = this.effectful_children(&cur);
            if children.is_empty() {
                // leaf: update best if longer or (same length and lexicographically smaller)
                if stack.len() > best.len() || (stack.len() == best.len() && stack < best) {
                    *best = stack.clone();
                }
            } else {
                // deterministic order: sort stringified ids
                let mut ordered: Vec<_> = children.into_iter().collect();
                ordered.sort();
                for nxt in ordered {
                    if !visited.contains(&nxt) {
                        dfs(this, nxt.clone(), stack, best, visited);
                    }
                }
            }
            stack.pop();
            visited.swap_remove(&cur);
        }
        let mut visited = IndexSet::new();
        dfs(self, root.clone(), &mut stack, &mut best, &mut visited);
        if best.is_empty() {
            vec![root]
        } else {
            best
        }
    }

    /// SCost extraction within a RegionSubEGraph returning an extraction expressed in original ClassIds.
    fn scost_region_extraction(
        &self,
        rsub: &RegionSubEGraph,
        root_orig: &ClassId,
    ) -> Option<TigerExtraction> {
        let root_ridx = *rsub.orig_to_region.get(root_orig)?; // expect 0
        let g = &rsub.egraph;
        let n = g.eclasses.len();
        let mut dis: Vec<IndexSet<usize>> = vec![IndexSet::new(); n];
        let mut pick: Vec<Option<usize>> = vec![None; n];
        let mut rev_ind: Vec<Vec<(usize, usize)>> = vec![vec![]; n];
        let mut counters: Vec<Vec<(usize, IndexSet<usize>)>> = Vec::with_capacity(n);
        for (i, ec) in g.eclasses.iter().enumerate() {
            let mut ec_counters = Vec::with_capacity(ec.enodes.len());
            for (j, en) in ec.enodes.iter().enumerate() {
                ec_counters.push((en.children.len(), IndexSet::from([i])));
                for &ch in &en.children {
                    rev_ind[ch].push((i, j));
                }
            }
            counters.push(ec_counters);
        }
        let mut heap: BinaryHeap<Reverse<(usize, usize)>> = BinaryHeap::new();
        for i in 0..n {
            for (j, en) in g.eclasses[i].enodes.iter().enumerate() {
                if en.children.is_empty() {
                    dis[i] = IndexSet::from([i]);
                    pick[i] = Some(j);
                    heap.push(Reverse((1, i)));
                    break; // only need first leaf enode per eclass seed
                }
            }
        }
        while let Some(Reverse((sz, i))) = heap.pop() {
            if dis[i].len() != sz {
                continue;
            }
            // propagate to parents
            for &(p_ec, p_en) in &rev_ind[i] {
                let (ref mut remain, ref mut acc) = counters[p_ec][p_en];
                if *remain == 0 {
                    continue;
                }
                // union dis[i] into accumulator
                for v in &dis[i] {
                    acc.insert(*v);
                }
                *remain -= 1;
                if *remain == 0 && (dis[p_ec].is_empty() || acc.len() < dis[p_ec].len()) {
                    dis[p_ec] = acc.clone();
                    pick[p_ec] = Some(p_en);
                    heap.push(Reverse((dis[p_ec].len(), p_ec)));
                }
            }
        }
        if dis[root_ridx].is_empty() {
            return None;
        }
        // BFS collect reachable
        let mut in_extraction = vec![false; n];
        let mut q = VecDeque::new();
        in_extraction[root_ridx] = true;
        q.push_back(root_ridx);
        while let Some(c) = q.pop_front() {
            if let Some(chosen) = pick[c] {
                let en = &g.eclasses[c].enodes[chosen];
                for &ch in &en.children {
                    if !in_extraction[ch] {
                        in_extraction[ch] = true;
                        q.push_back(ch);
                    }
                }
            }
        }
        let mut ord: Vec<(usize, usize)> = in_extraction
            .iter()
            .enumerate()
            .filter(|(_, f)| **f)
            .map(|(i, _)| (dis[i].len(), i))
            .collect();
        ord.sort();
        let mut extraction = TigerExtraction::new();
        let mut eclass_to_ex_idx: Vec<Option<usize>> = vec![None; n];
        for &(_sz, ec_idx) in &ord {
            let chosen = match pick[ec_idx] {
                Some(p) => p,
                None => continue,
            };
            let en = &g.eclasses[ec_idx].enodes[chosen];
            // ensure children's extraction indices exist
            let mut child_indices = Vec::new();
            for &ch in &en.children {
                if let Some(ci) = eclass_to_ex_idx[ch] {
                    child_indices.push(ci);
                }
            }
            let ex_idx = extraction.add_node(TigerExtractionENode {
                eclass: g.eclasses[ec_idx].original.clone(),
                enode_index: chosen,
                children: child_indices,
            });
            eclass_to_ex_idx[ec_idx] = Some(ex_idx);
        }
        extraction.root_index = eclass_to_ex_idx[root_ridx];
        Some(extraction)
    }

    /// Reconstruct (merge) multiple region extractions into a single extraction (placeholder simple pass-through).
    fn reconstruct_extraction(&self, extraction: TigerExtraction) -> TigerExtraction {
        extraction
    }

    /// Advanced region extraction using unguided state walk + SCost within region graph.
    fn advanced_region_extraction(
        &self,
        root: &ClassId,
    ) -> Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>, bool)> {
        let rsub = create_region_egraph(&self.tiger, root);
        let guided_full = self.guided_find_state_walk_region(&rsub);
        let (walk_ids, guided_pairs, wl_violation) = if guided_full.len() > 1 {
            (
                guided_full.iter().map(|(c, _)| c.clone()).collect(),
                guided_full.clone(),
                false,
            )
        } else {
            let p = self.unguided_find_state_walk_region(&rsub);
            (p, Vec::new(), false)
        };
        let mut extraction = self.scost_region_extraction(&rsub, root)?;
        // If we have guided pairs try specialized region walk extraction; prefer if improves region linearity.
        if !guided_pairs.is_empty() {
            if let Some(walk_ex) = self.region_extraction_with_state_walk(&rsub, &guided_pairs) {
                // choose walk_ex if it passes region linearity and scost version does not or if smaller node count
                let scost_lin = self.region_linearity_check(&extraction);
                let walk_lin = self.region_linearity_check(&walk_ex);
                if walk_lin && (!scost_lin || walk_ex.nodes.len() <= extraction.nodes.len()) {
                    extraction = walk_ex;
                }
            }
        }
        Some((
            self.reconstruct_extraction(extraction),
            walk_ids,
            guided_pairs,
            wl_violation,
        ))
    }

    /// Advanced multi-region extraction: merge per-region SCost results along state-walk.
    fn advanced_multi_region_extraction(
        &self,
        root: &ClassId,
    ) -> Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>, bool)> {
        // Build full region graph once to get unguided state walk anchors
        let full_rsub = create_region_egraph(&self.tiger, root);
        if full_rsub.region_to_orig.is_empty() {
            return None;
        }
        let guided_full = self.guided_find_state_walk_region(&full_rsub);
        let (walk, guided_pairs, wl_violation) = if guided_full.len() > 1 {
            (
                guided_full.iter().map(|(c, _)| c.clone()).collect(),
                guided_full,
                false,
            )
        } else {
            let p = self.unguided_find_state_walk_region(&full_rsub);
            (p, Vec::new(), false)
        };
        if walk.is_empty() {
            return None;
        }
        // Per-anchor region extraction
        let mut per_region: Vec<TigerExtraction> = Vec::new();
        for anchor in &walk {
            let rsub = create_region_egraph(&self.tiger, anchor);
            if let Some(ex) = self.scost_region_extraction(&rsub, anchor) {
                per_region.push(ex);
            } else {
                return None;
            }
        }
        // Merge extractions: first occurrence of eclass wins; children rewired to previously inserted nodes.
        let mut merged = TigerExtraction::new();
        use std::collections::HashMap;
        let mut eclass_to_global: HashMap<ClassId, usize> = HashMap::new();
        for rex in &per_region {
            for node in &rex.nodes {
                if !eclass_to_global.contains_key(&node.eclass) {
                    let mut new_children = Vec::new();
                    for &ch_idx in &node.children {
                        let ch_ec = &rex.nodes[ch_idx].eclass;
                        if let Some(&gidx) = eclass_to_global.get(ch_ec) {
                            new_children.push(gidx);
                        }
                    }
                    let gidx = merged.add_node(TigerExtractionENode {
                        eclass: node.eclass.clone(),
                        enode_index: node.enode_index,
                        children: new_children,
                    });
                    eclass_to_global.insert(node.eclass.clone(), gidx);
                }
            }
        }
        if let Some(first_anchor) = walk.first() {
            merged.root_index = eclass_to_global.get(first_anchor).copied();
        }
        Some((merged, walk, guided_pairs, wl_violation))
    }

    /// Recursive multi-region extraction mirroring C++ reconstructExtraction semantics.
    fn advanced_recursive_multi_region_extraction(
        &self,
        root: &ClassId,
    ) -> Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>, bool)> {
        // Build initial region egraph for root to get a state walk (guided if possible)
        let rsub_root = create_region_egraph(&self.tiger, root);
        if rsub_root.region_to_orig.is_empty() {
            return None;
        }
        let guided_full = self.guided_find_state_walk_region(&rsub_root);
        let (walk_ids, guided_pairs, wl_violation) = if guided_full.len() > 1 {
            (
                guided_full.iter().map(|(c, _)| c.clone()).collect(),
                guided_full.clone(),
                false,
            )
        } else {
            let p = self.unguided_find_state_walk_region(&rsub_root);
            (p, Vec::new(), false)
        };
        if walk_ids.is_empty() {
            return None;
        }
        // Builder struct to accumulate global extraction
        use std::collections::HashMap;
        struct Builder<'b> {
            ext: TigerExtraction,
            tiger: &'b TigerEGraph,
            // memo of region root -> extraction node index
            region_root_node: HashMap<ClassId, usize>,
        }
        impl<'b> Builder<'b> {
            fn new(tiger: &'b TigerEGraph) -> Self {
                Self {
                    ext: TigerExtraction::new(),
                    tiger,
                    region_root_node: HashMap::new(),
                }
            }
            fn extract_region_recursive(
                &mut self,
                region_root: &ClassId,
                this: &TigerExtractor,
            ) -> Option<usize> {
                if let Some(&idx) = self.region_root_node.get(region_root) {
                    return Some(idx);
                }
                // Build region subgraph & SCost extraction
                let rsub = create_region_egraph(self.tiger, region_root);
                let re = this.scost_region_extraction(&rsub, region_root)?; // already in original ClassIds
                                                                            // Map chosen enode per class for this region
                let mut chosen_enode: HashMap<ClassId, usize> = HashMap::new();
                for n in &re.nodes {
                    chosen_enode.insert(n.eclass.clone(), n.enode_index);
                }
                // Local map original class -> extraction index (within global ext)
                let mut local_map: HashMap<ClassId, usize> = HashMap::new();
                for n in &re.nodes {
                    // nodes are topological (children first)
                    let cid = &n.eclass;
                    // Chosen tiger enode
                    let t_idx = match this.tiger.class_index.get(cid) {
                        Some(v) => *v,
                        None => continue,
                    };
                    let ten = &this.tiger.eclasses[t_idx].enodes[n.enode_index];
                    // Prepare children indices; ensure subregion extractions added for additional effectual children before parent insertion.
                    let mut child_indices: Vec<usize> = Vec::new();
                    let mut seen_effectual = false;
                    for &ch_ti in &ten.children {
                        let ch_ec = &this.tiger.eclasses[ch_ti];
                        let ch_cid = &ch_ec.original;
                        if ch_ec.is_effectful {
                            if !seen_effectual {
                                // first effectual child: must belong to same region extraction so should have been added earlier
                                if let Some(&ci) = local_map.get(ch_cid) {
                                    child_indices.push(ci);
                                }
                                seen_effectual = true;
                            } else {
                                // additional effectful child: recurse as subregion
                                if let Some(sr_root_idx) =
                                    self.extract_region_recursive(ch_cid, this)
                                {
                                    child_indices.push(sr_root_idx);
                                }
                            }
                        } else {
                            // pure child inside region extraction
                            if let Some(&ci) = local_map.get(ch_cid) {
                                child_indices.push(ci);
                            }
                        }
                    }
                    // Add current node AFTER ensuring all (including subregions) children indices exist
                    let ex_idx = self.ext.add_node(TigerExtractionENode {
                        eclass: cid.clone(),
                        enode_index: n.enode_index,
                        children: child_indices,
                    });
                    local_map.insert(cid.clone(), ex_idx);
                }
                // Region root should now be in local_map
                let root_idx = *local_map.get(region_root)?;
                self.region_root_node.insert(region_root.clone(), root_idx);
                Some(root_idx)
            }
        }
        let mut builder = Builder::new(&self.tiger);
        let root_idx = builder.extract_region_recursive(root, self)?;
        builder.ext.root_index = Some(root_idx);
        Some((builder.ext, walk_ids, guided_pairs, wl_violation))
    }

    /// Validate structural properties of an extraction similar to tiger.cpp validExtraction.
    pub fn valid_extraction(&self, extraction: &TigerExtraction, root: &ClassId) -> bool {
        let Some(root_idx) = extraction.root_index else {
            return false;
        };
        if extraction.nodes.is_empty() {
            return false;
        }
        if extraction.nodes[root_idx].eclass != *root {
            return false;
        }
        // bounds & child structure / acyclicity
        for (i, n) in extraction.nodes.iter().enumerate() {
            // child indices must be < i (topological order we maintain)
            for &ch in &n.children {
                if ch >= i {
                    return false;
                }
            }
        }
        true
    }

    /// Recursive region-style linearity check (port of checkLinearRegionRec logic).
    pub fn region_linearity_check(&self, extraction: &TigerExtraction) -> bool {
        let Some(root_idx) = extraction.root_index else {
            return false;
        };
        // quick map eclass -> effectful flag
        let mut effectful: HashMap<ClassId, bool> = HashMap::new();
        for n in &extraction.nodes {
            if let Some(&ti) = self.tiger.class_index.get(&n.eclass) {
                effectful.insert(n.eclass.clone(), self.tiger.eclasses[ti].is_effectful);
            }
        }
        fn rec(
            nodes: &[TigerExtractionENode],
            cur: usize,
            effectful: &HashMap<ClassId, bool>,
        ) -> bool {
            // Build state walk within this region
            let mut statewalk: Vec<usize> = vec![cur];
            let mut onpath = vec![false; nodes.len()];
            onpath[cur] = true;
            let mut subregions: Vec<usize> = Vec::new();
            for i in 0..statewalk.len() {
                // will extend as we push
                let u = statewalk[i];
                let mut next_eff: Option<usize> = None;
                for &ch in &nodes[u].children {
                    if *effectful.get(&nodes[ch].eclass).unwrap_or(&false) {
                        if next_eff.is_none() {
                            next_eff = Some(ch);
                            statewalk.push(ch);
                            onpath[ch] = true;
                        } else {
                            subregions.push(ch);
                        }
                    }
                }
            }
            // BFS over pure nodes reachable from path; any effectful seen must lie on path
            let mut q: VecDeque<usize> = VecDeque::new();
            let mut seen = vec![false; nodes.len()];
            for &p in &statewalk {
                q.push_back(p);
                seen[p] = true;
            }
            while let Some(u) = q.pop_front() {
                for &ch in &nodes[u].children {
                    if *effectful.get(&nodes[ch].eclass).unwrap_or(&false) {
                        if !onpath[ch] {
                            return false;
                        }
                    } else if !seen[ch] {
                        seen[ch] = true;
                        q.push_back(ch);
                    }
                }
            }
            // recurse into subregions
            for &sr in &subregions {
                if !rec(nodes, sr, effectful) {
                    return false;
                }
            }
            true
        }
        rec(&extraction.nodes, root_idx, &effectful)
    }

    /// Build an extraction constrained to a guided state walk (simplified port of regionExtractionWithStateWalk).
    fn region_extraction_with_state_walk(
        &self,
        rsub: &RegionSubEGraph,
        walk: &[(ClassId, usize)],
    ) -> Option<TigerExtraction> {
        if walk.is_empty() {
            return None;
        }
        let mut new_classes: Vec<TigerEClass> = Vec::new();
        let mut base_map: Vec<Option<usize>> = vec![None; rsub.egraph.eclasses.len()];
        for (i, ec) in rsub.egraph.eclasses.iter().enumerate() {
            let mut clone_ec = TigerEClass {
                enodes: vec![],
                is_effectful: ec.is_effectful,
                original: ec.original.clone(),
            };
            if !ec.is_effectful {
                for en in &ec.enodes {
                    clone_ec.enodes.push(TigerENode {
                        head: en.head.clone(),
                        eclass_idx: new_classes.len(),
                        children: en.children.clone(),
                        original_class: ec.original.clone(),
                        original_node: en.original_node.clone(),
                    });
                }
            }
            base_map[i] = Some(new_classes.len());
            new_classes.push(clone_ec);
        }
        let mut last_idx: Option<usize> = None;
        for (cid, en_idx) in walk.iter().rev() {
            // reverse walk like C++
            let r_idx = *rsub.orig_to_region.get(cid)?;
            let orig_ec = &rsub.egraph.eclasses[r_idx];
            if *en_idx >= orig_ec.enodes.len() {
                return None;
            }
            let chosen = &orig_ec.enodes[*en_idx];
            let target_idx = if orig_ec.is_effectful
                && new_classes[base_map[r_idx].unwrap()].enodes.is_empty()
            {
                base_map[r_idx].unwrap()
            } else if orig_ec.is_effectful
                && !new_classes[base_map[r_idx].unwrap()].enodes.is_empty()
            {
                let dup_idx = new_classes.len();
                new_classes.push(TigerEClass {
                    enodes: vec![],
                    is_effectful: true,
                    original: orig_ec.original.clone(),
                });
                dup_idx
            } else {
                base_map[r_idx].unwrap()
            };
            let mut new_children: Vec<usize> = Vec::new();
            let mut replaced = false;
            for &ch in &chosen.children {
                let ch_ec = &rsub.egraph.eclasses[ch];
                let mapped = base_map[ch].unwrap();
                if ch_ec.is_effectful {
                    if let Some(li) = last_idx {
                        if !replaced {
                            new_children.push(li);
                            replaced = true;
                            continue;
                        }
                    }
                    if !replaced {
                        new_children.push(mapped);
                        replaced = true;
                    }
                } else {
                    new_children.push(mapped);
                }
            }
            new_classes[target_idx].enodes.push(TigerENode {
                head: chosen.head.clone(),
                eclass_idx: target_idx,
                children: new_children,
                original_class: cid.clone(),
                original_node: chosen.original_node.clone(),
            });
            last_idx = Some(target_idx);
        }
        let root_new = last_idx?;
        let mut class_index: IndexMap<ClassId, usize> = IndexMap::new();
        for (i, ec) in new_classes.iter().enumerate() {
            class_index.insert(ec.original.clone(), i);
        }
        let g = TigerEGraph {
            eclasses: new_classes,
            class_index,
        };
        // SCost extraction on g rooted at root_new
        let n = g.eclasses.len();
        let mut dis: Vec<IndexSet<usize>> = vec![IndexSet::new(); n];
        let mut pick: Vec<Option<usize>> = vec![None; n];
        let mut rev_ind: Vec<Vec<(usize, usize)>> = vec![vec![]; n];
        let mut counters: Vec<Vec<(usize, IndexSet<usize>)>> = Vec::with_capacity(n);
        for (i, ec) in g.eclasses.iter().enumerate() {
            let mut ec_counters = Vec::with_capacity(ec.enodes.len());
            for (j, en) in ec.enodes.iter().enumerate() {
                ec_counters.push((en.children.len(), IndexSet::from([i])));
                for &ch in &en.children {
                    rev_ind[ch].push((i, j));
                }
            }
            counters.push(ec_counters);
        }
        let mut heap: BinaryHeap<Reverse<(usize, usize)>> = BinaryHeap::new();
        for i in 0..n {
            for (j, en) in g.eclasses[i].enodes.iter().enumerate() {
                if en.children.is_empty() {
                    dis[i] = IndexSet::from([i]);
                    pick[i] = Some(j);
                    heap.push(Reverse((1, i)));
                    break;
                }
            }
        }
        while let Some(Reverse((sz, i))) = heap.pop() {
            if dis[i].len() != sz {
                continue;
            }
            for &(p_ec, p_en) in &rev_ind[i] {
                let (ref mut rem, ref mut acc) = counters[p_ec][p_en];
                if *rem == 0 {
                    continue;
                }
                for v in &dis[i] {
                    acc.insert(*v);
                }
                *rem -= 1;
                if *rem == 0 && (dis[p_ec].is_empty() || acc.len() < dis[p_ec].len()) {
                    dis[p_ec] = acc.clone();
                    pick[p_ec] = Some(p_en);
                    heap.push(Reverse((dis[p_ec].len(), p_ec)));
                }
            }
        }
        if dis[root_new].is_empty() {
            return None;
        }
        let mut in_extraction = vec![false; n];
        let mut q = VecDeque::new();
        in_extraction[root_new] = true;
        q.push_back(root_new);
        while let Some(c) = q.pop_front() {
            if let Some(pe) = pick[c] {
                let en = &g.eclasses[c].enodes[pe];
                for &ch in &en.children {
                    if !in_extraction[ch] {
                        in_extraction[ch] = true;
                        q.push_back(ch);
                    }
                }
            }
        }
        let mut ord: Vec<(usize, usize)> = in_extraction
            .iter()
            .enumerate()
            .filter(|(_, f)| **f)
            .map(|(i, _)| (dis[i].len(), i))
            .collect();
        ord.sort();
        let mut extraction = TigerExtraction::new();
        let mut eclass_to_ex_idx: Vec<Option<usize>> = vec![None; n];
        for &(_sz, ec_idx) in &ord {
            let Some(chosen) = pick[ec_idx] else { continue };
            let en = &g.eclasses[ec_idx].enodes[chosen];
            let mut child_indices = Vec::new();
            for &ch in &en.children {
                if let Some(ci) = eclass_to_ex_idx[ch] {
                    child_indices.push(ci);
                }
            }
            let ex_idx = extraction.add_node(TigerExtractionENode {
                eclass: g.eclasses[ec_idx].original.clone(),
                enode_index: chosen,
                children: child_indices,
            });
            eclass_to_ex_idx[ec_idx] = Some(ex_idx);
        }
        extraction.root_index = eclass_to_ex_idx[root_new];
        Some(extraction)
    }

    /// Unguided state-walk search (bitset/prioritized version). Replaces earlier simple DFS.
    fn unguided_find_state_walk_region(&self, rsub: &RegionSubEGraph) -> Vec<ClassId> {
        if rsub.region_to_orig.is_empty() {
            return vec![];
        }
        #[derive(Clone, Eq, PartialEq, Hash)]
        struct Key {
            idx: usize,
            bits: Vec<u64>,
        }
        #[derive(Eq, PartialEq)]
        struct Ranked {
            cost: usize,
            len: usize,
            path: Vec<ClassId>,
            key: Key,
        }
        impl Ord for Ranked {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other
                    .cost
                    .cmp(&self.cost)
                    .then_with(|| other.len.cmp(&self.len))
                    .then_with(|| self.path.cmp(&other.path))
            }
        }
        impl PartialOrd for Ranked {
            fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(o))
            }
        }
        use std::collections::{BinaryHeap, HashMap};
        let root_cid = &rsub.region_to_orig[0];
        let mut base: ExtractableSet = vec![false; self.tiger.eclasses.len()];
        if let Some(&ti) = self.tiger.class_index.get(root_cid) {
            base[ti] = true;
        }
        let saturated = self.saturate_pure_counters(&base);
        let bits = self.compress_extractable(&saturated);
        let start_key = Key {
            idx: 0,
            bits: bits.clone(),
        };
        let mut heap: BinaryHeap<Ranked> = BinaryHeap::new();
        heap.push(Ranked {
            cost: 0,
            len: 1,
            path: vec![root_cid.clone()],
            key: start_key.clone(),
        });
        let mut best_state: HashMap<Key, (usize, usize)> = HashMap::new();
        best_state.insert(start_key, (0, 1));
        let mut best_result = vec![root_cid.clone()];
        let mut best_res_cost = usize::MAX;
        let mut best_res_len = usize::MAX;
        while let Some(rank) = heap.pop() {
            if let Some(&(bc, bl)) = best_state.get(&rank.key) {
                if rank.cost > bc || (rank.cost == bc && rank.len > bl) {
                    continue;
                }
            }
            let cur_idx = rsub.egraph.class_index[rank.path.last().unwrap()];
            let tec = &rsub.egraph.eclasses[cur_idx];
            let mut extended = false;
            for (en_i, en) in tec.enodes.iter().enumerate() {
                if let Some((child_idx, add_sub)) = en.children.iter().copied().find_map(|ch| {
                    if rsub.egraph.eclasses[ch].is_effectful {
                        Some((ch, rsub.n_subregion[cur_idx][en_i]))
                    } else {
                        None
                    }
                }) {
                    let child_cid = rsub.egraph.eclasses[child_idx].original.clone();
                    if rank.path.iter().any(|c| *c == child_cid) {
                        continue;
                    }
                    extended = true;
                    let mut new_es = base.clone();
                    if let Some(&ti) = self.tiger.class_index.get(&child_cid) {
                        new_es[ti] = true;
                    }
                    let sat = self.saturate_pure_counters(&new_es);
                    let new_bits = self.compress_extractable(&sat);
                    let mut new_path = rank.path.clone();
                    new_path.push(child_cid.clone());
                    let new_cost = rank.cost + add_sub;
                    let new_len = rank.len + 1;
                    let key = Key {
                        idx: child_idx,
                        bits: new_bits,
                    };
                    let entry = best_state
                        .entry(key.clone())
                        .or_insert((usize::MAX, usize::MAX));
                    if new_cost < entry.0 || (new_cost == entry.0 && new_len < entry.1) {
                        *entry = (new_cost, new_len);
                        heap.push(Ranked {
                            cost: new_cost,
                            len: new_len,
                            path: new_path,
                            key,
                        });
                    }
                }
            }
            if !extended
                && (rank.cost < best_res_cost
                    || (rank.cost == best_res_cost && rank.len < best_res_len)
                    || (rank.cost == best_res_cost
                        && rank.len == best_res_len
                        && rank.path < best_result))
            {
                best_res_cost = rank.cost;
                best_res_len = rank.len;
                best_result = rank.path.clone();
            }
        }
        best_result
    }

    // Re-add missing methods (previously removed during refactor)
    pub fn saturate_pure_counters(&self, seed: &ExtractableSet) -> ExtractableSet {
        let n = self.tiger.eclasses.len();
        let mut ret = vec![false; n];
        let mut edges: Vec<Vec<(usize, usize)>> = vec![vec![]; n];
        let mut counters: Vec<Vec<usize>> = vec![vec![]; n];
        use std::collections::VecDeque;
        let mut q: VecDeque<usize> = VecDeque::new();
        for ec_idx in 0..n {
            let ec = &self.tiger.eclasses[ec_idx];
            if seed.get(ec_idx).copied().unwrap_or(false) {
                ret[ec_idx] = true;
                q.push_back(ec_idx);
                continue;
            }
            if !ec.is_effectful {
                counters[ec_idx] = vec![0; ec.enodes.len()];
                let mut has_leaf = false;
                for (en_i, en) in ec.enodes.iter().enumerate() {
                    if en.children.is_empty() {
                        has_leaf = true;
                    }
                    counters[ec_idx][en_i] = en.children.len();
                    for &ch in &en.children {
                        edges[ch].push((ec_idx, en_i));
                    }
                }
                if has_leaf {
                    ret[ec_idx] = true;
                    q.push_back(ec_idx);
                }
            }
        }
        while let Some(u) = q.pop_front() {
            for &(p_ec, p_en) in &edges[u] {
                if ret[p_ec] {
                    continue;
                }
                if counters[p_ec][p_en] > 0 {
                    counters[p_ec][p_en] -= 1;
                }
                if counters[p_ec][p_en] == 0 {
                    ret[p_ec] = true;
                    q.push_back(p_ec);
                }
            }
        }
        ret
    }
    pub fn compress_extractable(&self, es: &ExtractableSet) -> Vec<u64> {
        let n = self.tiger.eclasses.len();
        let mut bits = vec![0u64; (n + 63) / 64];
        for i in 0..n {
            if !self.tiger.eclasses[i].is_effectful && es.get(i).copied().unwrap_or(false) {
                bits[i / 64] |= 1u64 << (i % 64);
            }
        }
        bits
    }
    /// Guided state-walk search (reinserted) capturing (ClassId, chosen_enode_index).
    fn guided_find_state_walk_region(&self, rsub: &RegionSubEGraph) -> Vec<(ClassId, usize)> {
        if rsub.region_to_orig.is_empty() {
            return vec![];
        }
        use std::collections::BinaryHeap;
        #[derive(Clone, Eq, PartialEq)]
        struct Path {
            classes: Vec<(ClassId, usize)>,
            len: usize,
            cost: usize,
        }
        #[derive(Eq, PartialEq)]
        struct Ranked(Path);
        impl Ord for Ranked {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0
                    .len
                    .cmp(&other.0.len)
                    .then_with(|| other.0.cost.cmp(&self.0.cost))
                    .then_with(|| {
                        let a: Vec<&ClassId> = self.0.classes.iter().map(|(c, _)| c).collect();
                        let b: Vec<&ClassId> = other.0.classes.iter().map(|(c, _)| c).collect();
                        a.cmp(&b)
                    })
            }
        }
        impl PartialOrd for Ranked {
            fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(o))
            }
        }
        let mut best: Option<Path> = None;
        let mut heap: BinaryHeap<Ranked> = BinaryHeap::new();
        let root_cid = rsub.region_to_orig[0].clone();
        let root_ec = &rsub.egraph.eclasses[0];
        for (en_i, en) in root_ec.enodes.iter().enumerate() {
            let cost_local = rsub.n_subregion[0][en_i]
                + en.children
                    .iter()
                    .filter(|&&ch| !rsub.egraph.eclasses[ch].is_effectful)
                    .count();
            heap.push(Ranked(Path {
                classes: vec![(root_cid.clone(), en_i)],
                len: 1,
                cost: cost_local,
            }));
        }
        while let Some(Ranked(path)) = heap.pop() {
            let better = match &best {
                None => true,
                Some(b) => {
                    path.len > b.len
                        || (path.len == b.len && path.cost < b.cost)
                        || (path.len == b.len
                            && path.cost == b.cost
                            && path.classes.iter().map(|(c, _)| c).collect::<Vec<_>>()
                                < b.classes.iter().map(|(c, _)| c).collect::<Vec<_>>())
                }
            };
            if better {
                best = Some(path.clone());
            }
            let (last_cid, last_en) = path.classes.last().unwrap();
            let last_idx = rsub.egraph.class_index[last_cid];
            let last_ec = &rsub.egraph.eclasses[last_idx];
            let chosen_en = &last_ec.enodes[*last_en];
            let mut eff_child: Option<usize> = None;
            for &ch in &chosen_en.children {
                if rsub.egraph.eclasses[ch].is_effectful {
                    eff_child = Some(ch);
                    break;
                }
            }
            if let Some(child_idx) = eff_child {
                let child_cid = rsub.egraph.eclasses[child_idx].original.clone();
                if path.classes.iter().any(|(c, _)| *c == child_cid) {
                    continue;
                }
                let child_ec = &rsub.egraph.eclasses[child_idx];
                for (cen_i, cen) in child_ec.enodes.iter().enumerate() {
                    let add_cost = rsub.n_subregion[child_idx][cen_i]
                        + cen
                            .children
                            .iter()
                            .filter(|&&ch| !rsub.egraph.eclasses[ch].is_effectful)
                            .count();
                    let mut new_path = path.classes.clone();
                    new_path.push((child_cid.clone(), cen_i));
                    heap.push(Ranked(Path {
                        len: path.len + 1,
                        cost: path.cost + add_cost,
                        classes: new_path,
                    }));
                }
            }
        }
        best.map(|p| p.classes).unwrap_or_default()
    }

    // Helper: get function body root eclass id from serialized egraph
    fn function_body_root(&self, func: &str) -> Option<ClassId> {
        use egraph_serialize::NodeId;
        let root_nid: NodeId = get_root(self.serialized, func);
        Some(self.serialized.nid_to_cid(&root_nid).clone())
    }

    // Helper: collect all effectful reachable eclasses from root following any effectful edge (first-order over full serialized egraph)
    fn effectful_reachable(&self, root: &ClassId) -> IndexSet<ClassId> {
        let mut seen: IndexSet<ClassId> = IndexSet::new();
        let mut q: VecDeque<ClassId> = VecDeque::new();
        seen.insert(root.clone());
        q.push_back(root.clone());
        while let Some(cur) = q.pop_front() {
            if let Some(class) = self.serialized.classes().get(&cur) {
                for nid in &class.nodes {
                    let node = &self.serialized[nid];
                    for ch in &node.children {
                        let cc = self.serialized.nid_to_cid(ch).clone();
                        if let Some(&ti) = self.tiger.class_index.get(&cc) {
                            if self.tiger.eclasses[ti].is_effectful && !seen.contains(&cc) {
                                seen.insert(cc.clone());
                                q.push_back(cc);
                            }
                        }
                    }
                }
            }
        }
        seen
    }

    // Naive extraction: pick first enode per eclass reachable from root via DFS (effectful + pure closure)
    fn naive_extraction(&self, root: &ClassId) -> TigerExtraction {
        let mut extraction = TigerExtraction::new();
        let mut memo: HashMap<ClassId, usize> = HashMap::new();
        fn rec(
            this: &TigerExtractor,
            cid: &ClassId,
            memo: &mut HashMap<ClassId, usize>,
            ext: &mut TigerExtraction,
        ) -> Option<usize> {
            if let Some(&idx) = memo.get(cid) {
                return Some(idx);
            }
            let &ti = this.tiger.class_index.get(cid)?;
            let tec = &this.tiger.eclasses[ti];
            if tec.enodes.is_empty() {
                return None;
            }
            let chosen_en = 0usize; // first enode
            let en = &tec.enodes[chosen_en];
            let mut child_indices: Vec<usize> = Vec::new();
            for &ch_ti in &en.children {
                let ch_ec = &this.tiger.eclasses[ch_ti];
                let ch_cid = &ch_ec.original;
                // recurse regardless of effectfulness to capture pure closure
                if let Some(ci) = rec(this, ch_cid, memo, ext) {
                    child_indices.push(ci);
                }
            }
            let idx = ext.add_node(TigerExtractionENode {
                eclass: cid.clone(),
                enode_index: chosen_en,
                children: child_indices,
            });
            memo.insert(cid.clone(), idx);
            Some(idx)
        }
        if let Some(idx) = rec(self, root, &mut memo, &mut extraction) {
            extraction.root_index = Some(idx);
        }
        extraction
    }

    pub fn extract(&self, functions: &[String]) -> TigerExtractionResult {
        let mut chosen_enodes: IndexMap<ClassId, usize> = IndexMap::new();
        let mut state_walks: IndexMap<ClassId, Vec<ClassId>> = IndexMap::new();
        let mut regions: IndexMap<ClassId, Vec<TigerRegion>> = IndexMap::new();
        let mut region_stats: IndexMap<ClassId, Vec<TigerRegionStats>> = IndexMap::new();
        let mut extractions: IndexMap<ClassId, TigerExtraction> = IndexMap::new();
        let mut linearity_ok: IndexMap<ClassId, bool> = IndexMap::new();
        let mut guided_state_walks: IndexMap<ClassId, Vec<(ClassId, usize)>> = IndexMap::new();
        let mut weak_linearity_excess: IndexMap<ClassId, usize> = IndexMap::new();
        let mut weak_linearity_violation: IndexMap<ClassId, bool> = IndexMap::new();
        let mut debug_lines: Vec<String> = Vec::new();

        for func in functions {
            if let Some(root_body) = self.function_body_root(func) {
                let mut used_strategy = String::new();
                let mut wl_flag = false;
                // Attempt strategies in order
                let mut best: Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>)> = None;
                if let Some((ex, walk, guided, wl)) =
                    self.advanced_recursive_multi_region_extraction(&root_body)
                {
                    wl_flag = wl;
                    best = Some((ex, walk, guided));
                    used_strategy = "recursive-multi-region".into();
                } else if let Some((ex, walk, guided, wl)) =
                    self.advanced_multi_region_extraction(&root_body)
                {
                    wl_flag = wl;
                    best = Some((ex, walk, guided));
                    used_strategy = "multi-region".into();
                } else if let Some((ex, walk, guided, wl)) =
                    self.advanced_region_extraction(&root_body)
                {
                    wl_flag = wl;
                    best = Some((ex, walk, guided));
                    used_strategy = "single-region".into();
                }
                let (extraction, walk_ids, guided_pairs) = if let Some(b) = best {
                    b
                } else {
                    used_strategy = "fallback-naive".into();
                    let walk_ids = self.build_state_walk(root_body.clone());
                    (self.naive_extraction(&root_body), walk_ids, Vec::new())
                };
                let lin_ok = self.region_linearity_check(&extraction)
                    && self.valid_extraction(&extraction, &root_body);
                // record
                if let Some(ridx) = extraction.root_index {
                    chosen_enodes.insert(root_body.clone(), extraction.nodes[ridx].enode_index);
                }
                state_walks.insert(root_body.clone(), walk_ids.clone());
                if !guided_pairs.is_empty() {
                    guided_state_walks.insert(root_body.clone(), guided_pairs.clone());
                }
                let eff_reach = self.effectful_reachable(&root_body);
                let excess = eff_reach
                    .len()
                    .saturating_sub(state_walks[&root_body].len());
                weak_linearity_excess.insert(root_body.clone(), excess);
                weak_linearity_violation.insert(root_body.clone(), wl_flag || excess > 0);
                extractions.insert(root_body.clone(), extraction.clone());
                linearity_ok.insert(root_body.clone(), lin_ok);
                // placeholder empty region info (parity TODO)
                regions.insert(root_body.clone(), Vec::new());
                region_stats.insert(root_body.clone(), Vec::new());
                debug_lines.push(format!(
                    "func={} strategy={} lin_ok={} wl_violation={} excess={}",
                    func,
                    used_strategy,
                    lin_ok,
                    weak_linearity_violation[&root_body],
                    weak_linearity_excess[&root_body]
                ));
            }
        }
        TigerExtractionResult {
            chosen_enodes,
            state_walks,
            regions,
            region_stats,
            extractions,
            linearity_ok,
            debug: debug_lines.join("\n"),
            guided_state_walks,
            weak_linearity_excess,
            weak_linearity_violation,
        }
    }
}

#[allow(dead_code)]
fn _touch_root_symbol<F: FnOnce()>(f: F) {
    f();
}
