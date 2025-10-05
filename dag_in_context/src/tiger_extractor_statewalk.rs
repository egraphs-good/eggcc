//! State-walk search utilities for Tiger extractor.
use crate::tiger_extractor_core::TigerExtractor;
use crate::tiger_extractor_types::{ExtractableSet, RegionSubEGraph};
use crate::tiger_format::TigerEGraph;
use egraph_serialize::ClassId;
use indexmap::IndexSet;
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
    pub fn unguided_find_state_walk_region(&self, rsub: &RegionSubEGraph) -> Vec<ClassId> {
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
            let mut eff_child = None;
            for &ch in &chosen.children {
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
}
