//! State-walk search utilities for Tiger extractor.
use crate::tiger_extractor_core::TigerExtractor;
use crate::tiger_extractor_types::{ExtractableSet, RegionSubEGraph};
use crate::tiger_format::{TigerEClass, TigerEGraph};
use egraph_serialize::ClassId;
use indexmap::{IndexMap, IndexSet};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

impl<'a> TigerExtractor<'a> {
    pub fn build_state_walk(&self, root_cid: ClassId) -> Vec<ClassId> {
        self.build_longest_state_walk(root_cid)
    }
    pub(crate) fn effectful_children(&self, cid: &ClassId) -> IndexSet<ClassId> {
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
    pub(crate) fn build_longest_state_walk(&self, root: ClassId) -> Vec<ClassId> {
        let mut best = Vec::<ClassId>::new();
        let mut stack = Vec::<ClassId>::new();
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
                if stack.len() > best.len() || (stack.len() == best.len() && stack < best) {
                    *best = stack.clone();
                }
            } else {
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

    // --- New: find argument (leaf effectful enode) inside a region egraph ---
    fn find_arg_in_region(&self, rsub: &RegionSubEGraph) -> Option<(usize, usize)> {
        // Heuristic parity improvement: pick deepest effectful leaf (no effectful children).
        if rsub.egraph.eclasses.is_empty() {
            return None;
        }
        let n = rsub.egraph.eclasses.len();
        // Compute depth from root (index 0) along effectful edges.
        let mut depth = vec![usize::MAX; n];
        use std::collections::VecDeque;
        let mut q = VecDeque::new();
        depth[0] = 0;
        q.push_back(0);
        while let Some(u) = q.pop_front() {
            let d = depth[u];
            let ec = &rsub.egraph.eclasses[u];
            for en in &ec.enodes {
                for &ch in &en.children {
                    if rsub.egraph.eclasses[ch].is_effectful {
                        if depth[ch] == usize::MAX {
                            depth[ch] = d + 1;
                            q.push_back(ch);
                        }
                    }
                }
            }
        }
        // Collect candidate (effectful eclass, enode) pairs that have no effectful child.
        let mut candidates: Vec<(usize, usize)> = Vec::new();
        for (ri, ec) in rsub.egraph.eclasses.iter().enumerate() {
            if !ec.is_effectful {
                continue;
            }
            for (en_i, en) in ec.enodes.iter().enumerate() {
                let mut has_eff_child = false;
                for &ch in &en.children {
                    if rsub.egraph.eclasses[ch].is_effectful {
                        has_eff_child = true;
                        break;
                    }
                }
                if !has_eff_child {
                    candidates.push((ri, en_i));
                }
            }
        }
        if candidates.is_empty() {
            // Fallback to previous simple scan
            for (ri, ec) in rsub.egraph.eclasses.iter().enumerate() {
                if ec.is_effectful {
                    if !ec.enodes.is_empty() {
                        return Some((ri, 0));
                    }
                }
            }
            return None;
        }
        candidates.sort_by(|&(a_ec, a_en), &(b_ec, b_en)| {
            // Depth: larger is better (reverse order), so compare b then a.
            let da = depth[a_ec];
            let db = depth[b_ec];
            db.cmp(&da)
                .then_with(|| {
                    // Cost tie-break: smaller subregion cost better.
                    let ca = rsub.n_subregion[a_ec][a_en];
                    let cb = rsub.n_subregion[b_ec][b_en];
                    ca.cmp(&cb)
                })
                .then_with(|| rsub.region_to_orig[a_ec].cmp(&rsub.region_to_orig[b_ec]))
                .then_with(|| a_en.cmp(&b_en))
        });
        Some(candidates[0])
    }

    // --- Reimplemented unguided state-walk search (C++ parity) ---
    pub fn unguided_find_state_walk_region(
        &self,
        rsub: &RegionSubEGraph,
    ) -> (Vec<(ClassId, usize)>, bool, IndexMap<ClassId, u32>) {
        if rsub.region_to_orig.is_empty() {
            return (Vec::new(), false, IndexMap::new());
        }
        let root_region_idx = 0usize;
        let Some((init_region_idx, init_enode_idx)) = self.find_arg_in_region(rsub) else {
            return (Vec::new(), false, IndexMap::new());
        };
        let mut rev_edges: Vec<Vec<(usize, usize)>> = vec![vec![]; rsub.egraph.eclasses.len()];
        for (ri, ec) in rsub.egraph.eclasses.iter().enumerate() {
            if !ec.is_effectful {
                continue;
            }
            for (en_i, en) in ec.enodes.iter().enumerate() {
                for &ch in &en.children {
                    if rsub.egraph.eclasses[ch].is_effectful {
                        rev_edges[ch].push((ri, en_i));
                    }
                }
            }
        }
        #[derive(Clone, Eq, PartialEq, Hash)]
        struct Key {
            idx: usize,
            bits: Vec<u64>,
        }
        #[derive(Clone)]
        struct RichState {
            cost: usize,
            len: usize,
            key: Key,
            parent: Option<usize>,
            via_enode: usize,
            es: ExtractableSet,
        }
        use std::collections::hash_map::Entry;
        let mut rstates: Vec<RichState> = Vec::new();
        let mut heap: BinaryHeap<(Reverse<(usize, usize)>, usize)> = BinaryHeap::new();
        let mut best: HashMap<Key, (usize, usize)> = HashMap::new();
        let mut wlcnt: HashMap<ClassId, u32> = HashMap::new();
        let mut weak_linearity = false;
        let mut seed: ExtractableSet = vec![false; self.tiger.eclasses.len()];
        if let Some(&ti) = self
            .tiger
            .class_index
            .get(&rsub.region_to_orig[init_region_idx])
        {
            seed[ti] = true;
        }
        let sat0 = self.saturate_pure_counters(&seed);
        let bits0 = self.compress_extractable(&sat0);
        let init_key0 = Key {
            idx: init_region_idx,
            bits: bits0.clone(),
        };
        rstates.push(RichState {
            cost: 0,
            len: 1,
            key: init_key0.clone(),
            parent: None,
            via_enode: init_enode_idx,
            es: sat0,
        });
        heap.push((Reverse((0usize, 1usize)), 0));
        *wlcnt
            .entry(rsub.region_to_orig[init_region_idx].clone())
            .or_insert(0) += 1;
        best.insert(init_key0, (0, 1));
        let mut goal_state: Option<usize> = None;
        let is_extractable = |es: &ExtractableSet, parent_idx: usize, en_idx: usize| -> bool {
            let pen = &rsub.egraph.eclasses[parent_idx].enodes[en_idx];
            for &ch in &pen.children {
                let ch_cid = &rsub.egraph.eclasses[ch].original;
                if let Some(&ti) = self.tiger.class_index.get(ch_cid) {
                    if !es[ti] {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        };
        let mut region_to_global: Vec<Option<usize>> = vec![None; rsub.egraph.eclasses.len()];
        for (ri, ec) in rsub.egraph.eclasses.iter().enumerate() {
            if let Some(&ti) = self.tiger.class_index.get(&ec.original) {
                region_to_global[ri] = Some(ti);
            }
        }
        while let Some((Reverse((cost, len)), si)) = heap.pop() {
            // Take state snapshot
            let (cur_region_idx, cur_cost, cur_len, cur_es, cur_parent) = {
                let st = &rstates[si];
                if st.cost != cost || st.len != len {
                    continue;
                }
                (st.key.idx, st.cost, st.len, st.es.clone(), st.parent)
            };
            if cur_region_idx == root_region_idx {
                goal_state = Some(si);
                break;
            }
            // Collect expansions first
            let mut expansions: Vec<(usize, usize, ExtractableSet, Vec<u64>, usize, usize)> =
                Vec::new();
            for &(parent_idx, pen_i) in &rev_edges[cur_region_idx] {
                if !is_extractable(&cur_es, parent_idx, pen_i) {
                    continue;
                }
                let mut new_es = cur_es.clone();
                if let Some(gidx) = region_to_global[parent_idx] {
                    if !new_es[gidx] {
                        new_es[gidx] = true;
                    }
                }
                let saturated = self.saturate_pure_counters(&new_es);
                let new_bits = self.compress_extractable(&saturated);
                let new_cost = cur_cost + rsub.n_subregion[parent_idx][pen_i];
                let new_len = cur_len + 1;
                expansions.push((parent_idx, pen_i, saturated, new_bits, new_cost, new_len));
            }
            for (parent_idx, pen_i, saturated, new_bits, new_cost, new_len) in expansions {
                let key = Key {
                    idx: parent_idx,
                    bits: new_bits,
                };
                let mut push_new = false;
                match best.entry(key.clone()) {
                    Entry::Occupied(mut e) => {
                        let (bc, bl) = *e.get();
                        if new_cost < bc || (new_cost == bc && new_len < bl) {
                            e.insert((new_cost, new_len));
                            push_new = true;
                        }
                    }
                    Entry::Vacant(e) => {
                        e.insert((new_cost, new_len));
                        push_new = true;
                    }
                }
                if push_new {
                    let cid_parent = rsub.region_to_orig[parent_idx].clone();
                    let cnt = wlcnt.entry(cid_parent).or_insert(0);
                    *cnt += 1;
                    if *cnt > 1 {
                        weak_linearity = true;
                    }
                    let new_index = rstates.len();
                    rstates.push(RichState {
                        cost: new_cost,
                        len: new_len,
                        key: key.clone(),
                        parent: Some(si),
                        via_enode: pen_i,
                        es: saturated,
                    });
                    heap.push((Reverse((new_cost, new_len)), new_index));
                }
            }
        }
        let goal = match goal_state {
            Some(g) => g,
            None => {
                return (Vec::new(), weak_linearity, wlcnt.into_iter().collect());
            }
        };
        let mut path_rev: Vec<(ClassId, usize)> = Vec::new();
        let mut cur = Some(goal);
        while let Some(ci) = cur {
            let st = &rstates[ci];
            let cid = rsub.region_to_orig[st.key.idx].clone();
            path_rev.push((cid, st.via_enode));
            cur = st.parent;
        }
        path_rev.reverse();
        (path_rev, weak_linearity, wlcnt.into_iter().collect())
    }

    pub(crate) fn guided_find_state_walk_region(
        &self,
        rsub: &RegionSubEGraph,
    ) -> Vec<(ClassId, usize)> {
        if rsub.region_to_orig.is_empty() {
            return vec![];
        }
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
        // Precompute a heuristic: minimal per-effectful-class enode cost (subregion + pure children)
        let mut min_enode_cost: Vec<usize> = vec![0; rsub.egraph.eclasses.len()];
        for (i, ec) in rsub.egraph.eclasses.iter().enumerate() {
            if ec.is_effectful {
                let mut best = usize::MAX;
                for (en_i, en) in ec.enodes.iter().enumerate() {
                    let add = rsub.n_subregion[i][en_i]
                        + en.children
                            .iter()
                            .filter(|&&ch| !rsub.egraph.eclasses[ch].is_effectful)
                            .count();
                    if add < best {
                        best = add;
                    }
                }
                min_enode_cost[i] = if best == usize::MAX { 0 } else { best };
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
            let chosen = &last_ec.enodes[*last_en];
            // Gather effectful children and order by heuristic (min future enode cost), then lex id.
            let mut eff_children: Vec<usize> = chosen
                .children
                .iter()
                .copied()
                .filter(|&ch| rsub.egraph.eclasses[ch].is_effectful)
                .collect();
            eff_children.sort_by(|&a, &b| {
                min_enode_cost[a].cmp(&min_enode_cost[b]).then_with(|| {
                    rsub.egraph.eclasses[a]
                        .original
                        .cmp(&rsub.egraph.eclasses[b].original)
                })
            });
            if eff_children.is_empty() {
                continue;
            }
            for child_idx in eff_children {
                let child_cid = rsub.egraph.eclasses[child_idx].original.clone();
                if path.classes.iter().any(|(c, _)| *c == child_cid) {
                    continue;
                }
                let child_ec = &rsub.egraph.eclasses[child_idx];
                // Push only the cheapest enode first; others deferred after discovering longer path candidates.
                // Collect enodes sorted by (local add cost, enode index) to mimic greedy heuristic.
                let mut cenodes: Vec<(usize, usize)> = child_ec
                    .enodes
                    .iter()
                    .enumerate()
                    .map(|(cen_i, cen)| {
                        let add = rsub.n_subregion[child_idx][cen_i]
                            + cen
                                .children
                                .iter()
                                .filter(|&&ch| !rsub.egraph.eclasses[ch].is_effectful)
                                .count();
                        (add, cen_i)
                    })
                    .collect();
                cenodes.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                for (add_cost, cen_i) in cenodes.into_iter() {
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

    pub fn analyze_state_walk_ordering(
        &self,
        sw: &[(ClassId, usize)],
        rsub: Option<&RegionSubEGraph>,
    ) -> Vec<ClassId> {
        // Work over either a region subgraph (if provided) or full tiger graph.
        // This mirrors C++ analyzeStateWalkOrdering: compute an ordering of pure eclasses
        // whose extraction readiness depends (transitively) on effectful nodes along the walk.
        let (eclasses, class_index): (Vec<&TigerEClass>, &IndexMap<ClassId, usize>) =
            if let Some(rs) = rsub {
                (rs.egraph.eclasses.iter().collect(), &rs.egraph.class_index)
            } else {
                (
                    self.tiger.eclasses.iter().collect(),
                    &self.tiger.class_index,
                )
            };
        let n = eclasses.len();
        let mut contains_get = vec![false; n];
        let mut vis = vec![false; n];
        let mut edges: Vec<Vec<(usize, usize)>> = vec![vec![]; n]; // child -> list of (pure parent, enode)
        let mut counters: Vec<Vec<usize>> = vec![vec![]; n];
        use std::collections::VecDeque;
        let mut q = VecDeque::new();
        for i in 0..n {
            let ec = eclasses[i];
            if !ec.is_effectful {
                counters[i] = vec![0; ec.enodes.len()];
                for (j, en) in ec.enodes.iter().enumerate() {
                    counters[i][j] = en.children.len();
                    if en.children.is_empty() && !vis[i] {
                        q.push_back(i);
                        vis[i] = true;
                    }
                    for &ch in &en.children {
                        edges[ch].push((i, j));
                        if eclasses[ch].is_effectful {
                            contains_get[i] = true;
                        }
                    }
                }
            }
        }
        while let Some(u) = q.pop_front() {
            for &(vc, vn) in &edges[u] {
                if counters[vc][vn] > 0 {
                    counters[vc][vn] -= 1;
                }
                if counters[vc][vn] == 0 && !vis[vc] {
                    vis[vc] = true;
                    q.push_back(vc);
                }
            }
        }
        // Map walk ClassIds to indices
        let mut walk_indices: Vec<usize> = Vec::new();
        for (cid, _) in sw {
            if let Some(&idx) = class_index.get(cid) {
                walk_indices.push(idx);
            }
        }
        let mut ret: Vec<ClassId> = Vec::new();
        // Traverse walk backwards
        for wi in (0..walk_indices.len()).rev() {
            let root_idx = walk_indices[wi];
            if !vis[root_idx] {
                // need to explore region rooted here
                let mut loc_q = VecDeque::new();
                vis[root_idx] = true;
                loc_q.push_back(root_idx);
                while let Some(u) = loc_q.pop_front() {
                    if wi + 1 < walk_indices.len() && contains_get[u] {
                        // not last (arg) position
                        let cid = eclasses[u].original.clone();
                        ret.push(cid);
                    }
                    for &(vc, vn) in &edges[u] {
                        if counters[vc][vn] > 0 {
                            counters[vc][vn] -= 1;
                        }
                        if counters[vc][vn] == 0 && !vis[vc] {
                            vis[vc] = true;
                            loc_q.push_back(vc);
                        }
                    }
                }
            }
        }
        ret
    }
}
