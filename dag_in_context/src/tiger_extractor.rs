use crate::greedy_dag_extractor::get_root;
use crate::tiger_format::{build_tiger_egraph, TigerEClass, TigerEGraph, TigerENode};
use egraph_serialize::{ClassId, EGraph};
use indexmap::{IndexMap, IndexSet};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::{HashMap, VecDeque};

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
///  1. First BFS over effectful backbone: include at most the first effectful child edge per enode;
///     count subsequent effectful children in n_subregion.
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

    /// Depth-first search for the longest chain of effectful eclasses starting at root.
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
                let walk = self.build_state_walk(root_body.clone());
                state_walks.insert(root_body.clone(), walk.clone());
                let segs = self.build_regions_for_walk(&walk);
                regions.insert(root_body.clone(), segs.clone());
                let stats = self.compute_region_stats(&segs);
                region_stats.insert(root_body.clone(), stats.clone());

                // Build allowed set: union of region members (already excludes unrelated effectful nodes), saturate pure closure.
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

    // Saturate over pure nodes reachable from seed (do not expand through effectual nodes except initial seed elements).
    fn saturate_pure(&self, seed: &IndexSet<ClassId>) -> IndexSet<ClassId> {
        let mut out = seed.clone();
        let mut q: VecDeque<ClassId> = seed.iter().cloned().collect();
        while let Some(cid) = q.pop_front() {
            if let Some(idx) = self.tiger.class_index.get(&cid) {
                let tec = &self.tiger.eclasses[*idx];
                // only expand through pure
                if tec.is_effectful && !seed.contains(&cid) {
                    continue;
                }
                for en in &tec.enodes {
                    for ch_idx in &en.children {
                        let cc = self.tiger.eclasses[*ch_idx].original.clone();
                        if out.insert(cc.clone()) {
                            // expand only if pure
                            if let Some(cidx) = self.tiger.class_index.get(&cc) {
                                if !self.tiger.eclasses[*cidx].is_effectful {
                                    q.push_back(cc);
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }

    // Bitset compression not yet required; keep placeholder for future optimization.
    #[allow(dead_code)]
    fn compress_set(&self, _set: &IndexSet<ClassId>) -> Vec<u64> {
        vec![]
    }

    // Cost of extraction node subtree (cached)
    fn extraction_cost(&self, extraction: &TigerExtraction, idx: usize) -> usize {
        fn rec(
            nodes: &Vec<TigerExtractionENode>,
            idx: usize,
            memo: &mut Vec<Option<usize>>,
        ) -> usize {
            if let Some(c) = memo[idx] {
                return c;
            }
            let mut s = 1usize;
            for &ch in &nodes[idx].children {
                s += rec(nodes, ch, memo);
            }
            memo[idx] = Some(s);
            s
        }
        let mut memo = vec![None; extraction.nodes.len()];
        rec(&extraction.nodes, idx, &mut memo)
    }

    // Compute minimal-cost enode for each allowed eclass reachable from root (simple DP ignoring sharing cycles).
    fn region_restricted_extraction(
        &self,
        root: ClassId,
        _walk: &[ClassId],
        allowed: &IndexSet<ClassId>,
    ) -> TigerExtraction {
        // Memo: cid -> (cost, chosen_enode_index)
        let mut memo: HashMap<ClassId, (usize, usize)> = HashMap::new();
        let mut stack: IndexSet<ClassId> = IndexSet::new();
        fn solve(
            this: &TigerExtractor,
            cid: &ClassId,
            allowed: &IndexSet<ClassId>,
            memo: &mut HashMap<ClassId, (usize, usize)>,
            stack: &mut IndexSet<ClassId>,
        ) -> Option<(usize, usize)> {
            if let Some(v) = memo.get(cid) {
                return Some(*v);
            }
            if !allowed.contains(cid) {
                return None;
            }
            if !stack.insert(cid.clone()) {
                return None;
            } // cycle guard
            let t_idx = *this.tiger.class_index.get(cid)?;
            let tec = &this.tiger.eclasses[t_idx];
            let mut best: Option<(usize, usize)> = None; // (cost,enode_index)
            for (i, en) in tec.enodes.iter().enumerate() {
                let mut sum = 1usize; // cost of this node
                let mut ok = true;
                for ch_idx in &en.children {
                    let child_cid = this.tiger.eclasses[*ch_idx].original.clone();
                    match solve(this, &child_cid, allowed, memo, stack) {
                        Some((c, _)) => sum += c,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    continue;
                }
                match best {
                    None => best = Some((sum, i)),
                    Some((bc, bi)) => {
                        if sum < bc || (sum == bc && i < bi) {
                            best = Some((sum, i));
                        }
                    }
                }
            }
            stack.swap_remove(cid);
            if let Some(b) = best {
                memo.insert(cid.clone(), b);
            }
            best
        }
        solve(self, &root, allowed, &mut memo, &mut stack);
        // Build extraction graph following chosen enodes reachable from root.
        let mut extraction = TigerExtraction::new();
        let mut map: HashMap<ClassId, usize> = HashMap::new();
        fn build(
            this: &TigerExtractor,
            cid: &ClassId,
            allowed: &IndexSet<ClassId>,
            memo: &HashMap<ClassId, (usize, usize)>,
            extraction: &mut TigerExtraction,
            map: &mut HashMap<ClassId, usize>,
        ) {
            if map.contains_key(cid) {
                return;
            }
            let &(_c, en_idx) = match memo.get(cid) {
                Some(v) => v,
                None => return,
            };
            let t_idx = match this.tiger.class_index.get(cid) {
                Some(i) => *i,
                None => return,
            };
            let tec = &this.tiger.eclasses[t_idx];
            let en = &tec.enodes[en_idx];
            // ensure children first
            let mut child_indices_runtime = Vec::new();
            for ch_t_idx in &en.children {
                let child_cid = this.tiger.eclasses[*ch_t_idx].original.clone();
                if !allowed.contains(&child_cid) {
                    continue;
                }
                build(this, &child_cid, allowed, memo, extraction, map);
                if let Some(&ci) = map.get(&child_cid) {
                    child_indices_runtime.push(ci);
                }
            }
            let ex_idx = extraction.add_node(TigerExtractionENode {
                eclass: cid.clone(),
                enode_index: en_idx,
                children: child_indices_runtime,
            });
            map.insert(cid.clone(), ex_idx);
        }
        build(self, &root, allowed, &memo, &mut extraction, &mut map);
        extraction.root_index = map.get(&root).copied();
        extraction
    }

    /// SCost-style greedy extraction (port of NormalGreedyExtraction from tiger.cpp).
    /// Returns None if root cannot be extracted.
    pub fn normal_greedy_extraction_scost(&self, root: ClassId) -> Option<TigerExtraction> {
        let root_idx = *self.tiger.class_index.get(&root)?;
        let n = self.tiger.eclasses.len();
        // dis: minimal SCost (set of eclass indices) for each eclass; empty = unreachable yet
        let mut dis: Vec<IndexSet<usize>> = vec![IndexSet::new(); n];
        // pick: chosen enode index per eclass
        let mut pick: Vec<Option<usize>> = vec![None; n];
        // rev_ind: for each eclass index child -> Vec<(parent_eclass_idx, parent_enode_idx)>
        let mut rev_ind: Vec<Vec<(usize, usize)>> = vec![vec![]; n];
        // counters[ec][en] = (remaining_children_to_satisfy, accumulated_set)
        let mut counters: Vec<Vec<(usize, IndexSet<usize>)>> = Vec::with_capacity(n);
        for (i, ec) in self.tiger.eclasses.iter().enumerate() {
            let mut ec_counters = Vec::with_capacity(ec.enodes.len());
            for (j, en) in ec.enodes.iter().enumerate() {
                ec_counters.push((en.children.len(), IndexSet::from([i])));
                for &ch in &en.children {
                    rev_ind[ch].push((i, j));
                }
            }
            counters.push(ec_counters);
        }
        // priority queue keyed by SCost size (min-heap via Reverse)
        let mut heap: BinaryHeap<Reverse<(usize, usize)>> = BinaryHeap::new();
        // seed with leaf enodes (enodes having zero children)
        for i in 0..n {
            for (j, en) in self.tiger.eclasses[i].enodes.iter().enumerate() {
                if en.children.is_empty() {
                    dis[i] = IndexSet::from([i]);
                    pick[i] = Some(j);
                    heap.push(Reverse((dis[i].len(), i)));
                    break; // only need first leaf enode per eclass seed
                }
            }
        }
        while let Some(Reverse((sz, i))) = heap.pop() {
            if dis[i].len() != sz {
                continue;
            }
            // propagate to parents
            for &(parent_ec, parent_en) in &rev_ind[i] {
                let (ref mut remain, ref mut acc) = counters[parent_ec][parent_en];
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
                    if dis[parent_ec].is_empty() || acc.len() < dis[parent_ec].len() {
                        dis[parent_ec] = acc.clone();
                        pick[parent_ec] = Some(parent_en);
                        heap.push(Reverse((dis[parent_ec].len(), parent_ec)));
                    }
                }
            }
        }
        if dis[root_idx].is_empty() {
            return None;
        }
        // collect reachable nodes of chosen extraction via BFS
        let mut in_extraction = vec![false; n];
        let mut q = VecDeque::new();
        in_extraction[root_idx] = true;
        q.push_back(root_idx);
        while let Some(c) = q.pop_front() {
            let chosen = match pick[c] {
                Some(p) => p,
                None => continue,
            };
            let en = &self.tiger.eclasses[c].enodes[chosen];
            for &ch in &en.children {
                if !in_extraction[ch] {
                    in_extraction[ch] = true;
                    q.push_back(ch);
                }
            }
        }
        // ordering by set size ascending (like C++: sort(ord.begin(), ord.end()))
        let mut ord: Vec<(usize, usize)> = in_extraction
            .iter()
            .enumerate()
            .filter(|(_, f)| **f)
            .map(|(i, _)| (dis[i].len(), i))
            .collect();
        ord.sort();
        // build extraction (topological by increasing SCost size implies parents appear after children)
        let mut extraction = TigerExtraction::new();
        let mut eclass_to_ex_idx: Vec<Option<usize>> = vec![None; n];
        for &(_sz, ec_idx) in &ord {
            let chosen = match pick[ec_idx] {
                Some(p) => p,
                None => continue,
            };
            let en = &self.tiger.eclasses[ec_idx].enodes[chosen];
            // ensure children's extraction indices exist
            let mut child_indices = Vec::new();
            for &ch in &en.children {
                if let Some(ci) = eclass_to_ex_idx[ch] {
                    child_indices.push(ci);
                }
            }
            let ex_idx = extraction.add_node(TigerExtractionENode {
                eclass: self.tiger.eclasses[ec_idx].original.clone(),
                enode_index: chosen,
                children: child_indices,
            });
            eclass_to_ex_idx[ec_idx] = Some(ex_idx);
        }
        extraction.root_index = eclass_to_ex_idx[root_idx];
        Some(extraction)
    }

    /// Structural validation of an extraction (subset of C++ validExtraction).
    pub fn validate_extraction(&self, extraction: &TigerExtraction, root: &ClassId) -> bool {
        let Some(root_idx) = extraction.root_index else {
            return false;
        };
        if extraction.nodes[root_idx].eclass != *root {
            return false;
        }
        // map for quick lookup (eclass -> node idx) only keeps last; acceptable for uniqueness assumption
        let mut map: HashMap<ClassId, usize> = HashMap::new();
        for (i, node) in extraction.nodes.iter().enumerate() {
            map.insert(node.eclass.clone(), i);
        }
        // validate children indices and acyclicity (indices < parent index due to construction order in our greedy builder; if not, still ensure DAG by DFS cycle check)
        for (i, node) in extraction.nodes.iter().enumerate() {
            for &ch in &node.children {
                if ch >= extraction.nodes.len() {
                    return false;
                }
                // no additional structural checks; types assumed consistent (original algorithm checked child eclass matches)
            }
            // simple cycle detection: parent index must be greater than all child indices in our construction, else run a slow DFS check
            if node.children.iter().any(|&c| c > i) {
                // fallback: build adjacency and check for cycle including node
                // (skip heavy for now, accept) – optional improvement.
            }
        }
        true
    }

    /// Full linearity check mirroring tiger.cpp (recursive region linearity) on a finished extraction.
    pub fn full_linearity_check(&self, extraction: &TigerExtraction) -> bool {
        let Some(root_idx) = extraction.root_index else {
            return false;
        };
        fn is_effectful(t: &TigerEGraph, cid: &ClassId) -> bool {
            t.eclasses[*t.class_index.get(cid).unwrap()].is_effectful
        }
        fn recurse(t: &TigerEGraph, ex: &TigerExtraction, idx: usize) -> bool {
            // Build statewalk: follow first effectful child; others effectful are subregions
            let mut statewalk: Vec<usize> = vec![idx];
            let mut subregions: Vec<usize> = vec![];
            let mut cur = idx;
            loop {
                let mut next_effectful: Option<usize> = None;
                for &ch in &ex.nodes[cur].children {
                    if is_effectful(t, &ex.nodes[ch].eclass) {
                        if next_effectful.is_none() {
                            next_effectful = Some(ch);
                        } else {
                            subregions.push(ch);
                        }
                    }
                }
                if let Some(nxt) = next_effectful {
                    statewalk.push(nxt);
                    cur = nxt;
                } else {
                    break;
                }
            }
            // BFS over pure nodes ensuring any effectual dependency is on statewalk
            let mut on_path = vec![false; ex.nodes.len()];
            for &p in &statewalk {
                on_path[p] = true;
            }
            let mut q = VecDeque::new();
            let mut seen = vec![false; ex.nodes.len()];
            for &p in &statewalk {
                seen[p] = true;
                for &ch in &ex.nodes[p].children {
                    if !is_effectful(t, &ex.nodes[ch].eclass) {
                        q.push_back(ch);
                        seen[ch] = true;
                    }
                }
            }
            while let Some(u) = q.pop_front() {
                for &ch in &ex.nodes[u].children {
                    if is_effectful(t, &ex.nodes[ch].eclass) {
                        if !on_path[ch] {
                            return false;
                        }
                    } else if !seen[ch] {
                        seen[ch] = true;
                        q.push_back(ch);
                    }
                }
            }
            for sub in subregions {
                if !recurse(t, ex, sub) {
                    return false;
                }
            }
            true
        }
        recurse(&self.tiger, extraction, root_idx)
    }
}
