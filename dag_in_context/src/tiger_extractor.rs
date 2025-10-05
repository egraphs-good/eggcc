use crate::greedy_dag_extractor::get_root;
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
///  * Computes a longest effectful "state walk" per function body
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
    pub state_walks: IndexMap<ClassId, Vec<ClassId>>, // function root class -> linear effectful path
    pub regions: IndexMap<ClassId, Vec<TigerRegion>>, // function root body -> region segments
    pub region_stats: IndexMap<ClassId, Vec<TigerRegionStats>>, // parallel to regions
    pub extractions: IndexMap<ClassId, TigerExtraction>, // function root body -> extraction (naive)
    pub linearity_ok: IndexMap<ClassId, bool>,
    pub debug: String,
}

impl<'a> TigerExtractor<'a> {
    pub fn new(serialized: &'a EGraph) -> Self {
        let tiger = build_tiger_egraph(serialized);
        Self { serialized, tiger }
    }

    /// Build a naive state walk: starting from the function root's body eclass, follow
    /// the first effectful child repeatedly until no further effectful child exists.
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

    /// Unguided state-walk search (approximation of prototype UnguidedFindStateWalk).
    /// Operates over a RegionSubEGraph starting at region root index 0.
    /// Heuristic: greedily follow effectual child that minimizes accumulated subregion count.
    fn unguided_find_state_walk_region(&self, rsub: &RegionSubEGraph) -> Vec<ClassId> {
        if rsub.region_to_orig.is_empty() {
            return vec![];
        }
        let mut walk: Vec<ClassId> = Vec::new();
        let mut cur_idx: usize = 0; // region root assumed index 0
        loop {
            let orig_cid = rsub.region_to_orig[cur_idx].clone();
            walk.push(orig_cid.clone());
            // find first effectful child in region graph (there can be at most one kept by pruning)
            let t_idx = match rsub.egraph.class_index.get(&orig_cid) {
                Some(v) => *v,
                None => break,
            };
            let tec = &rsub.egraph.eclasses[t_idx];
            let mut next_region: Option<usize> = None;
            // choose among enodes the one whose (single) effectful child yields min future n_subregion if multiple possibilities
            // (In pruned region graph, each enode has at most one effectful child) – pick earliest with minimal extra subregions
            let mut best_score: Option<usize> = None;
            for (en_i, en) in tec.enodes.iter().enumerate() {
                // locate its first effectful child (if any)
                let mut effectful_child: Option<usize> = None;
                for &ch in &en.children {
                    // children are region indices
                    let child_cid = rsub.egraph.eclasses[ch].original.clone();
                    let child_t_idx = rsub.egraph.class_index[&child_cid];
                    if rsub.egraph.eclasses[child_t_idx].is_effectful {
                        effectful_child = Some(ch);
                        break; // only first stored
                    }
                }
                if let Some(ch) = effectful_child {
                    let score = rsub.n_subregion[t_idx][en_i];
                    if best_score.map(|b| score < b).unwrap_or(true) {
                        best_score = Some(score);
                        next_region = Some(ch);
                    }
                }
            }
            match next_region {
                Some(n) => cur_idx = n,
                None => break,
            }
        }
        walk
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
                if *remain == 0 {
                    // all children satisfied; compute candidate cost set = acc
                    if dis[p_ec].is_empty() || acc.len() < dis[p_ec].len() {
                        dis[p_ec] = acc.clone();
                        pick[p_ec] = Some(p_en);
                        heap.push(Reverse((dis[p_ec].len(), p_ec)));
                    }
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
    ) -> Option<(TigerExtraction, Vec<ClassId>)> {
        let rsub = create_region_egraph(&self.tiger, root);
        let walk = self.unguided_find_state_walk_region(&rsub);
        // perform SCost within region graph (root of region graph is root)
        let extraction = self.scost_region_extraction(&rsub, root)?;
        Some((self.reconstruct_extraction(extraction), walk))
    }
}

// Patch extract() to attempt advanced extraction first then fall back.
impl<'a> TigerExtractor<'a> {
    pub fn extract(&self, functions: &[String]) -> TigerExtractionResult {
        // chosen_enodes still filled for compatibility (will be overwritten per extraction)
        let mut chosen_enodes: IndexMap<ClassId, usize> = IndexMap::new();
        for (cid, class) in self.serialized.classes() {
            if !class.nodes.is_empty() {
                chosen_enodes.insert(cid.clone(), 0);
            }
        }

        let mut state_walks = IndexMap::new();
        let mut regions: IndexMap<ClassId, Vec<TigerRegion>> = IndexMap::new();
        let mut region_stats: IndexMap<ClassId, Vec<TigerRegionStats>> = IndexMap::new();
        let mut extractions: IndexMap<ClassId, TigerExtraction> = IndexMap::new();
        let mut linearity_ok: IndexMap<ClassId, bool> = IndexMap::new();
        let mut debug = String::new();
        for fname in functions {
            if let Some(root_body) = self.function_body_root(fname) {
                // Try advanced region extraction
                let mut used_advanced = false;
                if let Some((adv_extraction, adv_walk)) =
                    self.advanced_region_extraction(&root_body)
                {
                    state_walks.insert(root_body.clone(), adv_walk.clone());
                    let lin_ok_full = self.full_linearity_check(&adv_extraction);
                    linearity_ok.insert(root_body.clone(), lin_ok_full);
                    for node in &adv_extraction.nodes {
                        chosen_enodes.insert(node.eclass.clone(), node.enode_index);
                    }
                    extractions.insert(root_body.clone(), adv_extraction.clone());
                    use std::fmt::Write;
                    let _ = writeln!(
                        debug,
                        "# function {fname} ADVANCED extraction state-walk len={} nodes={} linearity(full)={} ",
                        adv_walk.len(),
                        adv_extraction.nodes.len(),
                        lin_ok_full
                    );
                    used_advanced = true;
                }
                if !used_advanced {
                    let walk = self.build_state_walk(root_body.clone());
                    state_walks.insert(root_body.clone(), walk.clone());
                    let segs = self.build_regions_for_walk(&walk);
                    regions.insert(root_body.clone(), segs.clone());
                    let stats = self.compute_region_stats(&segs);
                    region_stats.insert(root_body.clone(), stats.clone());

                    // Build allowed set: union of region members (already excludes unrelated effectual nodes), saturate pure closure.
                    let mut allowed: IndexSet<ClassId> = IndexSet::new();
                    for r in &segs {
                        for m in &r.members {
                            allowed.insert(m.clone());
                        }
                    }
                    allowed = self.saturate_pure(&allowed);

                    // Region restricted cost-based extraction
                    let extraction =
                        self.region_restricted_extraction(root_body.clone(), &walk, &allowed);
                    // Record chosen enodes mapping
                    for node in &extraction.nodes {
                        chosen_enodes.insert(node.eclass.clone(), node.enode_index);
                    }
                    let lin_ok = self.check_linearity(&extraction, &walk);
                    extractions.insert(root_body.clone(), extraction.clone());
                    linearity_ok.insert(root_body.clone(), lin_ok);
                    use std::fmt::Write;
                    let cost_root = extraction
                        .root_index
                        .map(|ri| self.extraction_cost(&extraction, ri))
                        .unwrap_or(0);
                    let _ = writeln!(
                        debug,
                        "# function {fname} state-walk len={} classes={:?} linearity_ok={} extraction_nodes={} root_cost={}",
                        walk.len(),
                        walk,
                        lin_ok,
                        extraction.nodes.len(),
                        cost_root
                    );
                    for (i, r) in segs.iter().enumerate() {
                        let st = &stats[i];
                        let _ = writeln!(
                            debug,
                            "  region[{i}] anchor={} next={:?} size={} enodes(total/eff/pure)={}/{}/{}",
                            r.anchor,
                            r.next_anchor,
                            r.members.len(),
                            st.total_enodes,
                            st.effectful_enodes,
                            st.pure_enodes
                        );
                    }
                    // Emit a few chosen enodes (first 10) for quick inspection
                    for node in extraction.nodes.iter().take(10) {
                        let _ = writeln!(
                            debug,
                            "    chosen eclass={} enode_index={} children={:?}",
                            node.eclass, node.enode_index, node.children
                        );
                    }
                }
            }
        }
        TigerExtractionResult {
            chosen_enodes,
            state_walks,
            regions,
            region_stats,
            extractions,
            linearity_ok,
            debug,
        }
    }

    /// Build region segments given a state walk. Each segment spans from an effectful anchor
    /// up to (but not traversing through) the next effectful anchor.
    fn build_regions_for_walk(&self, walk: &[ClassId]) -> Vec<TigerRegion> {
        let mut res = vec![];
        for (i, anchor) in walk.iter().enumerate() {
            let next_anchor = walk.get(i + 1).cloned();
            let mut members: IndexSet<ClassId> = IndexSet::new();
            // BFS
            let mut q: Vec<ClassId> = vec![anchor.clone()];
            while let Some(c) = q.pop() {
                if members.contains(&c) {
                    continue;
                }
                // If this is an effectful other-than-anchor and not the next anchor, do not include/expand
                if c != *anchor
                    && self.tiger.eclasses[self.tiger.class_index[&c]].is_effectful
                    && Some(c.clone()) != next_anchor
                {
                    continue;
                }
                members.insert(c.clone());
                // expand children
                if let Some(class) = self.serialized.classes().get(&c) {
                    for nid in &class.nodes {
                        let node = &self.serialized[nid];
                        for ch in &node.children {
                            let cc = self.serialized.nid_to_cid(ch).clone();
                            if !members.contains(&cc) {
                                q.push(cc);
                            }
                        }
                    }
                }
            }
            res.push(TigerRegion {
                anchor: anchor.clone(),
                next_anchor,
                members,
            });
        }
        res
    }

    fn function_body_root(&self, func: &str) -> Option<ClassId> {
        // Reuse get_root logic to find Function node, then pick its body child (4th)
        let func_root_nid = get_root(self.serialized, func);
        let func_node = &self.serialized[&func_root_nid];
        if func_node.op != "Function" {
            return None;
        }
        if func_node.children.len() != 4 {
            return None;
        }
        let body_nid = func_node.children[3].clone();
        Some(self.serialized.nid_to_cid(&body_nid).clone())
    }

    fn compute_region_stats(&self, regions: &[TigerRegion]) -> Vec<TigerRegionStats> {
        regions
            .iter()
            .map(|reg| {
                let mut total = 0usize;
                let mut eff = 0usize;
                for cid in &reg.members {
                    if let Some(idx) = self.tiger.class_index.get(cid) {
                        let tec = &self.tiger.eclasses[*idx];
                        for _en in &tec.enodes {
                            total += 1;
                            if tec.is_effectful {
                                eff += 1;
                            }
                        }
                    }
                }
                TigerRegionStats {
                    total_enodes: total,
                    effectful_enodes: eff,
                    pure_enodes: total - eff,
                }
            })
            .collect()
    }

    // --- Added: naive greedy extraction (first-enode per reachable eclass) ---
    #[allow(dead_code)]
    fn build_naive_extraction(&self, root: ClassId) -> TigerExtraction {
        let mut extraction = TigerExtraction::new();
        let mut visited: IndexSet<ClassId> = IndexSet::new();
        // Post-order traversal to ensure children inserted first
        fn dfs(
            this: &TigerExtractor,
            cid: ClassId,
            visited: &mut IndexSet<ClassId>,
            order: &mut Vec<ClassId>,
        ) {
            if !visited.insert(cid.clone()) {
                return;
            }
            if let Some(class) = this.serialized.classes().get(&cid) {
                for nid in &class.nodes {
                    // only look at first enode's children later, but traverse all to discover reachable eclasses
                    let node = &this.serialized[nid];
                    for ch in &node.children {
                        order.push(this.serialized.nid_to_cid(ch).clone());
                    }
                }
            }
            // Recurse on newly discovered children (unique via visited)
            let mut child_cids: Vec<ClassId> = {
                // replace drain(..).collect() with take for clippy friendliness
                let taken: Vec<ClassId> = std::mem::take(order);
                taken
            };
            child_cids.sort();
            child_cids.dedup();
            for c in child_cids {
                dfs(this, c, visited, &mut Vec::new());
            }
        }
        dfs(self, root.clone(), &mut visited, &mut Vec::new());
        // Topologically add nodes: simple iteration over visited
        let mut class_indices: Vec<ClassId> = visited.into_iter().collect();
        class_indices.sort(); // deterministic
        let mut eclass_to_ex_idx: IndexMap<ClassId, usize> = IndexMap::new();
        for cid in class_indices {
            let tiger_idx = match self.tiger.class_index.get(&cid) {
                Some(i) => *i,
                None => continue,
            };
            let tec = &self.tiger.eclasses[tiger_idx];
            if tec.enodes.is_empty() {
                continue;
            }
            // choose first enode
            let enode_index = 0usize;
            // map children (only from chosen enode)
            let chosen = &tec.enodes[enode_index];
            let mut child_indices = Vec::new();
            for ch_idx in &chosen.children {
                // ch_idx is tiger index (usize) of child eclass
                let child_cid = self.tiger.eclasses[*ch_idx].original.clone();
                if let Some(&ex_child) = eclass_to_ex_idx.get(&child_cid) {
                    child_indices.push(ex_child);
                }
            }
            let ex_idx = extraction.add_node(TigerExtractionENode {
                eclass: cid.clone(),
                enode_index,
                children: child_indices,
            });
            eclass_to_ex_idx.insert(cid.clone(), ex_idx);
        }
        extraction.root_index = eclass_to_ex_idx.get(&root).copied();
        extraction
    }

    // Improved linearity check: every adjacent pair in walk must have descendant relation in extraction.
    fn check_linearity(&self, extraction: &TigerExtraction, walk: &[ClassId]) -> bool {
        if extraction.root_index.is_none() {
            return false;
        }
        // build map eclass -> node index
        let mut map: HashMap<ClassId, usize> = HashMap::new();
        for (i, n) in extraction.nodes.iter().enumerate() {
            map.insert(n.eclass.clone(), i);
        }
        for w in walk {
            if !map.contains_key(w) {
                return false;
            }
        }
        // helper descendant check with early cutoff
        fn is_desc(nodes: &[TigerExtractionENode], a: usize, b: usize) -> bool {
            if a == b {
                return true;
            }
            let mut stack = vec![a];
            let mut seen = vec![false; nodes.len()];
            while let Some(cur) = stack.pop() {
                if cur == b {
                    return true;
                }
                if seen[cur] {
                    continue;
                }
                seen[cur] = true;
                for &ch in &nodes[cur].children {
                    stack.push(ch);
                }
            }
            false
        }
        for win in walk.windows(2) {
            let a = map[&win[0]];
            let b = map[&win[1]];
            if !is_desc(&extraction.nodes, a, b) {
                return false;
            }
        }
        true
    }

    // Added back: saturate pure closure of allowed set (used before region_restricted_extraction)
    pub fn saturate_pure(&self, allowed: &IndexSet<ClassId>) -> IndexSet<ClassId> {
        let mut res = allowed.clone();
        let mut q: Vec<ClassId> = res.iter().cloned().collect();
        while let Some(cid) = q.pop() {
            // skip if not known in tiger graph
            if let Some(&t_idx) = self.tiger.class_index.get(&cid) {
                let tec = &self.tiger.eclasses[t_idx];
                // only add pure descendants (closure over pure)
                for en in &tec.enodes {
                    for &child_t_idx in &en.children {
                        let child_ec = &self.tiger.eclasses[child_t_idx];
                        if child_ec.is_effectful { continue; }
                        let child_cid = child_ec.original.clone();
                        if res.insert(child_cid.clone()) { q.push(child_cid); }
                    }
                }
            }
        }
        res
    }

    // Region-restricted DP extraction: only traverse eclasses in allowed set.
    pub fn region_restricted_extraction(&self, root: ClassId, _walk: &[ClassId], allowed: &IndexSet<ClassId>) -> TigerExtraction {
        // Gather tiger indices for allowed set
        let mut extraction = TigerExtraction::new();
        // Map ClassId -> tiger index
        let mut allowed_tiger: IndexMap<ClassId, usize> = IndexMap::new();
        for cid in allowed.iter() {
            if let Some(&ti) = self.tiger.class_index.get(cid) { allowed_tiger.insert(cid.clone(), ti); }
        }
        // Memo: (tiger idx) -> (best enode index, cost, children tiger indices)
        #[derive(Clone, Debug)]
        struct Best {
            en: usize,
            cost: usize,
            children: Vec<usize>,
        }
        let mut memo: Vec<Option<Best>> = vec![None; self.tiger.eclasses.len()];
        fn solve(this: &TigerExtractor, ti: usize, allowed_tiger: &IndexMap<ClassId, usize>, memo: &mut [Option<Best>]) -> Option<Best> {
            if let Some(b) = &memo[ti] { return Some(b.clone()); }
            let ec = &this.tiger.eclasses[ti];
            let mut best: Option<Best> = None;
            for (en_i, en) in ec.enodes.iter().enumerate() {
                let mut child_infos: Vec<Best> = Vec::with_capacity(en.children.len());
                let mut ok = true;
                for &ch_ti in &en.children {
                    let ch_ec = &this.tiger.eclasses[ch_ti];
                    if !allowed_tiger.contains_key(&ch_ec.original) { ok = false; break; }
                    if let Some(ci) = solve(this, ch_ti, allowed_tiger, memo) { child_infos.push(ci); } else { ok = false; break; }
                }
                if !ok { continue; }
                let cost_children: usize = child_infos.iter().map(|c| c.cost).sum();
                let cost = 1 + cost_children; // simple unit cost
                if best.as_ref().map(|b| cost < b.cost).unwrap_or(true) {
                    best = Some(Best { en: en_i, cost, children: en.children.clone() });
                }
            }
            memo[ti] = best.clone();
            best
        }
        // Build set of reachable allowed classes from root (to avoid including stray unreachable ones)
        let mut reachable: IndexSet<usize> = IndexSet::new();
        if let Some(&root_ti) = self.tiger.class_index.get(&root) { 
            let mut stack = vec![root_ti];
            while let Some(ti) = stack.pop() {
                if !reachable.insert(ti) { continue; }
                let ec = &self.tiger.eclasses[ti];
                for en in &ec.enodes { for &ch in &en.children { stack.push(ch); } }
            }
        }
        // Solve for root
        let root_ti = match self.tiger.class_index.get(&root) { Some(v) => *v, None => { return extraction; } };
        if solve(self, root_ti, &allowed_tiger, &mut memo).is_none() { return extraction; }
        // Topological order (simple DFS ensuring children first) over memo entries actually used
        let mut order: Vec<usize> = Vec::new();
        let mut seen: Vec<bool> = vec![false; self.tiger.eclasses.len()];
        fn dfs_build(ti: usize, memo: &[Option<Best>], seen: &mut [bool], order: &mut Vec<usize>) {
            if seen[ti] { return; }
            seen[ti] = true;
            if let Some(b) = &memo[ti] { for &ch in &b.children { dfs_build(ch, memo, seen, order); } }
            order.push(ti);
        }
        dfs_build(root_ti, &memo, &mut seen, &mut order);
        // Map tiger index -> extraction index
        let mut map_ti_ex: HashMap<usize, usize> = HashMap::new();
        for ti in order { if let Some(b) = &memo[ti] { 
            let child_indices: Vec<usize> = b.children.iter().filter_map(|ch| map_ti_ex.get(ch).copied()).collect();
            let ex_idx = extraction.add_node(TigerExtractionENode { eclass: self.tiger.eclasses[ti].original.clone(), enode_index: b.en, children: child_indices });
            map_ti_ex.insert(ti, ex_idx);
        }}
        extraction.root_index = map_ti_ex.get(&root_ti).copied();
        extraction
    }

    // Cost of subtree rooted at extraction node index (unit cost)
    pub fn extraction_cost(&self, extraction: &TigerExtraction, root_idx: usize) -> usize {
        let mut memo = vec![None; extraction.nodes.len()];
        fn dfs(i: usize, nodes: &[TigerExtractionENode], memo: &mut [Option<usize>]) -> usize {
            if let Some(v) = memo[i] { return v; }
            let mut c = 1; for &ch in &nodes[i].children { c += dfs(ch, nodes, memo); }
            memo[i] = Some(c); c
        }
        dfs(root_idx, &extraction.nodes, &mut memo)
    }

    // Full linearity check (restored simplified version): ensure each child index < parent and forms a DAG rooted at root.
    pub fn full_linearity_check(&self, extraction: &TigerExtraction) -> bool {
        let root = match extraction.root_index { Some(r) => r, None => return false };
        // parents must appear after children
        for (i, node) in extraction.nodes.iter().enumerate() {
            for &ch in &node.children { if ch >= i { return false; } }
        }
        // reachability from root covers all nodes
        let mut seen = vec![false; extraction.nodes.len()];
        let mut stack = vec![root];
        while let Some(i) = stack.pop() { if seen[i] { continue; } seen[i] = true; for &ch in &extraction.nodes[i].children { stack.push(ch); } }
        if seen.iter().any(|b| !*b) { return false; }
        true
    }
}
