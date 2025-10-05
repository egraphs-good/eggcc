use crate::greedy_dag_extractor::get_root;
use crate::tiger_format::{build_tiger_egraph, TigerEGraph};
use egraph_serialize::{ClassId, EGraph};
use indexmap::{IndexMap, IndexSet};

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

/// A very early partial port of the prototype tiger extractor.
/// Currently it:
///  * Builds the tiger egraph structure
///  * For each requested function root, computes a simple "state walk" consisting
///    of the chain of effectful eclasses reachable by repeatedly picking the first
///    effectful child encountered (depth-first).
///  * Chooses the first enode in every eclass as the representative (placeholder)
///  * Produces a debug summary string.
///
/// This is intentionally minimal so we can iterate; more logic from tiger.cpp
/// (region partitioning, subregion cost accounting, linear path search heuristics)
/// can be layered in later.
pub struct TigerExtractor<'a> {
    serialized: &'a EGraph,
    tiger: TigerEGraph,
}

pub struct TigerExtractionResult {
    pub chosen_enodes: IndexMap<ClassId, usize>, // eclass -> index into its enodes vec
    pub state_walks: IndexMap<ClassId, Vec<ClassId>>, // function root class -> linear effectful path
    pub regions: IndexMap<ClassId, Vec<TigerRegion>>, // function root body -> region segments
    pub region_stats: IndexMap<ClassId, Vec<TigerRegionStats>>, // parallel to regions
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
        let mut chosen_enodes: IndexMap<ClassId, usize> = IndexMap::new();
        for (cid, class) in self.serialized.classes() {
            if !class.nodes.is_empty() {
                chosen_enodes.insert(cid.clone(), 0);
            }
        }

        let mut state_walks = IndexMap::new();
        let mut regions: IndexMap<ClassId, Vec<TigerRegion>> = IndexMap::new();
        let mut region_stats: IndexMap<ClassId, Vec<TigerRegionStats>> = IndexMap::new();
        let mut debug = String::new();
        for fname in functions {
            if let Some(root_body) = self.function_body_root(fname) {
                let walk = self.build_state_walk(root_body.clone());
                state_walks.insert(root_body.clone(), walk.clone());
                let segs = self.build_regions_for_walk(&walk);
                regions.insert(root_body.clone(), segs.clone());
                let stats = self.compute_region_stats(&segs);
                region_stats.insert(root_body.clone(), stats.clone());
                use std::fmt::Write;
                let _ = writeln!(
                    debug,
                    "# function {fname} state-walk len={} classes={:?}",
                    walk.len(),
                    walk
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
            }
        }
        TigerExtractionResult {
            chosen_enodes,
            state_walks,
            regions,
            region_stats,
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
}
