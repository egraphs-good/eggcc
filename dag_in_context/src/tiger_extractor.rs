// Algorithm implementations for TigerExtractor (struct & orchestrator in tiger_extractor_core)
#![allow(clippy::collapsible_if)]
use crate::tiger_extractor_core::TigerExtractor;
use crate::tiger_extractor_types::{
    create_region_egraph, ExtractableSet, RegionSubEGraph, TigerExtraction, TigerExtractionENode,
    TigerRegion, TigerRegionStats,
};
use crate::tiger_format::{TigerEClass, TigerEGraph, TigerENode};
use egraph_serialize::ClassId;
use indexmap::{IndexMap, IndexSet};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

impl<'a> TigerExtractor<'a> {
    // --- Basic helpers ---
    // build_state_walk, effectful_children, build_longest_state_walk moved to tiger_extractor_statewalk

    // --- SCost region extraction inside RegionSubEGraph ---
    fn scost_region_extraction(
        &self,
        rsub: &RegionSubEGraph,
        root_orig: &ClassId,
    ) -> Option<TigerExtraction> {
        let root_ridx = *rsub.orig_to_region.get(root_orig)?;
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
                    break;
                }
            }
        }
        while let Some(Reverse((sz, i))) = heap.pop() {
            if dis[i].len() != sz {
                continue;
            }
            for &(p_ec, p_en) in &rev_ind[i] {
                let (ref mut remain, ref mut acc) = counters[p_ec][p_en];
                if *remain == 0 {
                    continue;
                }
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
        // Updated ordering: tie-break on original index to mimic C++ stable behavior.
        let mut ord: Vec<(usize, usize)> = in_extraction
            .iter()
            .enumerate()
            .filter(|(_, f)| **f)
            .map(|(i, _)| (dis[i].len(), i))
            .collect();
        ord.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let mut extraction = TigerExtraction::new();
        let mut eclass_to_ex: Vec<Option<usize>> = vec![None; n];
        for &(_sz, ec_idx) in &ord {
            let Some(chosen) = pick[ec_idx] else { continue };
            let en = &g.eclasses[ec_idx].enodes[chosen];
            let mut child_indices = Vec::new();
            for &ch in &en.children {
                if let Some(ci) = eclass_to_ex[ch] {
                    child_indices.push(ci);
                }
            }
            let ex_idx = extraction.add_node(TigerExtractionENode {
                eclass: g.eclasses[ec_idx].original.clone(),
                enode_index: chosen,
                children: child_indices,
            });
            eclass_to_ex[ec_idx] = Some(ex_idx);
        }
        extraction.root_index = eclass_to_ex[root_ridx];
        Some(extraction)
    }

    // --- Guided / Unguided region state-walk based strategies ---
    pub fn advanced_region_extraction(
        &self,
        root: &ClassId,
    ) -> Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>, bool)> {
        let rsub = create_region_egraph(&self.tiger, root);
        let guided_full = self.guided_find_state_walk_region(&rsub);
        let (walk_pairs, guided_pairs, wl) = if guided_full.len() > 1 {
            (guided_full.clone(), guided_full.clone(), false)
        } else {
            let (p, wl) = self.unguided_find_state_walk_region(&rsub);
            (p.clone(), Vec::new(), wl)
        };
        let walk_ids: Vec<ClassId> = walk_pairs.iter().map(|(c, _)| c.clone()).collect();
        let mut extraction = self.scost_region_extraction(&rsub, root)?;
        if !guided_pairs.is_empty() {
            if let Some(walk_ex) = self.region_extraction_with_state_walk(&rsub, &guided_pairs) {
                let sc_lin = self.region_linearity_check(&extraction);
                let wk_lin = self.region_linearity_check(&walk_ex);
                if wk_lin && (!sc_lin || walk_ex.nodes.len() <= extraction.nodes.len()) {
                    extraction = walk_ex;
                }
            }
        }
        Some((extraction, walk_ids, guided_pairs, wl))
    }
    pub fn advanced_multi_region_extraction(
        &self,
        root: &ClassId,
    ) -> Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>, bool)> {
        let full_rsub = create_region_egraph(&self.tiger, root);
        if full_rsub.region_to_orig.is_empty() {
            return None;
        }
        let guided_full = self.guided_find_state_walk_region(&full_rsub);
        let (walk_pairs, guided_pairs, wl) = if guided_full.len() > 1 {
            (guided_full.clone(), guided_full, false)
        } else {
            let (p, wl) = self.unguided_find_state_walk_region(&full_rsub);
            (p.clone(), Vec::new(), wl)
        };
        if walk_pairs.is_empty() {
            return None;
        }
        let walk: Vec<ClassId> = walk_pairs.iter().map(|(c, _)| c.clone()).collect();
        let mut per_region = Vec::new();
        for anchor in &walk {
            let rsub = create_region_egraph(&self.tiger, anchor);
            let Some(ex) = self.scost_region_extraction(&rsub, anchor) else {
                return None;
            };
            per_region.push(ex);
        }
        let mut merged = TigerExtraction::new();
        let mut map: HashMap<ClassId, usize> = HashMap::new();
        for rex in &per_region {
            for node in &rex.nodes {
                if !map.contains_key(&node.eclass) {
                    let mut new_children = Vec::new();
                    for &ch in &node.children {
                        let ch_ec = &rex.nodes[ch].eclass;
                        if let Some(&gidx) = map.get(ch_ec) {
                            new_children.push(gidx);
                        }
                    }
                    let gidx = merged.add_node(TigerExtractionENode {
                        eclass: node.eclass.clone(),
                        enode_index: node.enode_index,
                        children: new_children,
                    });
                    map.insert(node.eclass.clone(), gidx);
                }
            }
        }
        if let Some(first_anchor) = walk.first() {
            merged.root_index = map.get(first_anchor).copied();
        }
        Some((merged, walk, guided_pairs, wl))
    }
    pub fn advanced_recursive_multi_region_extraction(
        &self,
        root: &ClassId,
    ) -> Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>, bool)> {
        let rsub_root = create_region_egraph(&self.tiger, root);
        if rsub_root.region_to_orig.is_empty() {
            return None;
        }
        let guided_full = self.guided_find_state_walk_region(&rsub_root);
        let (walk_pairs, guided_pairs, wl) = if guided_full.len() > 1 {
            (guided_full.clone(), guided_full.clone(), false)
        } else {
            let (p, wl) = self.unguided_find_state_walk_region(&rsub_root);
            (p.clone(), Vec::new(), wl)
        };
        if walk_pairs.is_empty() {
            return None;
        }
        let walk_ids: Vec<ClassId> = walk_pairs.iter().map(|(c, _)| c.clone()).collect();
        struct Builder<'b> {
            ext: TigerExtraction,
            tiger: &'b TigerEGraph,
            region_root_node: HashMap<ClassId, usize>,
            region_wl: HashMap<ClassId, bool>,
        }
        impl<'b> Builder<'b> {
            fn new(tiger: &'b TigerEGraph) -> Self {
                Self {
                    ext: TigerExtraction::new(),
                    tiger,
                    region_root_node: HashMap::new(),
                    region_wl: HashMap::new(),
                }
            }
            fn extract_region_recursive(
                &mut self,
                region_root: &ClassId,
                this: &TigerExtractor,
            ) -> Option<(usize, bool)> {
                // returns (extraction_root_index, weak_linearity_seen)
                if let Some(&idx) = self.region_root_node.get(region_root) {
                    let wl = *self.region_wl.get(region_root).unwrap_or(&false);
                    return Some((idx, wl));
                }
                // Build region sub-egraph
                let rsub = create_region_egraph(self.tiger, region_root);
                let mut wl_region = false;
                // Try unguided state-walk based region extraction (C++ parity)
                let mut re = if !rsub.region_to_orig.is_empty() {
                    let (walk, wl) = this.unguided_find_state_walk_region(&rsub);
                    wl_region |= wl;
                    if !walk.is_empty() {
                        if let Some(wex) = this.region_extraction_with_state_walk(&rsub, &walk) {
                            if this.region_linearity_check(&wex) {
                                Some(wex)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                // Fallback to SCost extraction if walk strategy failed
                if re.is_none() {
                    re = this.scost_region_extraction(&rsub, region_root);
                }
                let re = re?;
                // Rebuild into global extraction graph, recursively expanding subregions beyond first effectful child
                let mut local: HashMap<ClassId, usize> = HashMap::new();
                for n in &re.nodes {
                    let cid = &n.eclass;
                    let t_idx = *this.tiger.class_index.get(cid)?;
                    let ten = &this.tiger.eclasses[t_idx].enodes[n.enode_index];
                    let mut child_indices = Vec::new();
                    let mut seen_eff = false;
                    for &ch_ti in &ten.children {
                        let ch_ec = &this.tiger.eclasses[ch_ti];
                        let ch_cid = &ch_ec.original;
                        if ch_ec.is_effectful {
                            if !seen_eff {
                                if let Some(&ci) = local.get(ch_cid) {
                                    child_indices.push(ci);
                                }
                                seen_eff = true;
                            } else if let Some((sr, wl_sub)) =
                                self.extract_region_recursive(ch_cid, this)
                            {
                                wl_region |= wl_sub;
                                child_indices.push(sr);
                            }
                        } else if let Some(&ci) = local.get(ch_cid) {
                            child_indices.push(ci);
                        }
                    }
                    let ex_idx = self.ext.add_node(TigerExtractionENode {
                        eclass: cid.clone(),
                        enode_index: n.enode_index,
                        children: child_indices,
                    });
                    local.insert(cid.clone(), ex_idx);
                }
                let root_idx = *local.get(region_root)?;
                self.region_root_node.insert(region_root.clone(), root_idx);
                self.region_wl.insert(region_root.clone(), wl_region);
                Some((root_idx, wl_region))
            }
        }
        let mut builder = Builder::new(&self.tiger);
        let (root_idx, wl_rec) = builder.extract_region_recursive(root, self)?;
        builder.ext.root_index = Some(root_idx);
        let overall_wl = wl || wl_rec; // combine top-level walk wl with recursive region wls
        Some((builder.ext, walk_ids, guided_pairs, overall_wl))
    }

    // --- Validation & linearity ---
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
        for (i, n) in extraction.nodes.iter().enumerate() {
            for &ch in &n.children {
                if ch >= i {
                    return false;
                }
            }
            let Some(&ti) = self.tiger.class_index.get(&n.eclass) else {
                return false;
            };
            if n.enode_index >= self.tiger.eclasses[ti].enodes.len() {
                return false;
            }
            let ten = &self.tiger.eclasses[ti].enodes[n.enode_index];
            let mut orig: IndexSet<ClassId> = IndexSet::new();
            for &ch_ti in &ten.children {
                orig.insert(self.tiger.eclasses[ch_ti].original.clone());
            }
            for &ci in &n.children {
                if !orig.contains(&extraction.nodes[ci].eclass) {
                    return false;
                }
            }
        }
        true
    }
    pub fn region_linearity_check(&self, extraction: &TigerExtraction) -> bool {
        let Some(root_idx) = extraction.root_index else {
            return false;
        };
        let mut eff: HashMap<ClassId, bool> = HashMap::new();
        for n in &extraction.nodes {
            if let Some(&ti) = self.tiger.class_index.get(&n.eclass) {
                eff.insert(n.eclass.clone(), self.tiger.eclasses[ti].is_effectful);
            }
        }
        fn rec(nodes: &[TigerExtractionENode], cur: usize, eff: &HashMap<ClassId, bool>) -> bool {
            let mut statewalk = vec![cur];
            let mut onpath = vec![false; nodes.len()];
            onpath[cur] = true;
            let mut sub = Vec::new();
            for i in 0..statewalk.len() {
                let u = statewalk[i];
                let mut next_eff = None;
                for &ch in &nodes[u].children {
                    if *eff.get(&nodes[ch].eclass).unwrap_or(&false) {
                        if next_eff.is_none() {
                            next_eff = Some(ch);
                            statewalk.push(ch);
                            onpath[ch] = true;
                        } else {
                            sub.push(ch);
                        }
                    }
                }
            }
            let mut q = VecDeque::new();
            let mut seen = vec![false; nodes.len()];
            for &p in &statewalk {
                q.push_back(p);
                seen[p] = true;
            }
            while let Some(u) = q.pop_front() {
                for &ch in &nodes[u].children {
                    if *eff.get(&nodes[ch].eclass).unwrap_or(&false) {
                        if !onpath[ch] {
                            return false;
                        }
                    } else if !seen[ch] {
                        seen[ch] = true;
                        q.push_back(ch);
                    }
                }
            }
            for &sr in &sub {
                if !rec(nodes, sr, eff) {
                    return false;
                }
            }
            true
        }
        rec(&extraction.nodes, root_idx, &eff)
    }

    // --- Region building from walk (stats) ---
    pub fn build_regions_for_walk(
        &self,
        walk: &[ClassId],
    ) -> (Vec<TigerRegion>, Vec<TigerRegionStats>) {
        if walk.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let mut regions = Vec::new();
        let mut stats = Vec::new();
        for (i, anchor) in walk.iter().enumerate() {
            let next = walk.get(i + 1).cloned();
            let mut members: IndexSet<ClassId> = IndexSet::new();
            let mut q = VecDeque::new();
            members.insert(anchor.clone());
            q.push_back(anchor.clone());
            while let Some(cur) = q.pop_front() {
                let t_idx = match self.tiger.class_index.get(&cur) {
                    Some(v) => *v,
                    None => continue,
                };
                let tec = &self.tiger.eclasses[t_idx];
                for en in &tec.enodes {
                    let mut seen_eff = false;
                    for &ch_ti in &en.children {
                        let ch_ec = &self.tiger.eclasses[ch_ti];
                        let ch_cid = ch_ec.original.clone();
                        if ch_ec.is_effectful {
                            if Some(&ch_cid) == next.as_ref() {
                                members.insert(ch_cid);
                            } else if !seen_eff {
                                seen_eff = true;
                                if !members.contains(&ch_cid) {
                                    members.insert(ch_cid.clone());
                                    q.push_back(ch_cid);
                                }
                            }
                        } else {
                            if !members.contains(&ch_cid) {
                                members.insert(ch_cid.clone());
                                q.push_back(ch_cid);
                            }
                        }
                    }
                }
            }
            let (mut total, mut eff_en, mut pure_en) = (0, 0, 0);
            for cid in &members {
                if let Some(&ti) = self.tiger.class_index.get(cid) {
                    let ec = &self.tiger.eclasses[ti];
                    for _ in &ec.enodes {
                        total += 1;
                        if ec.is_effectful {
                            eff_en += 1;
                        } else {
                            pure_en += 1;
                        }
                    }
                }
            }
            regions.push(TigerRegion {
                anchor: anchor.clone(),
                next_anchor: next.clone(),
                members: members.clone(),
            });
            stats.push(TigerRegionStats {
                total_enodes: total,
                effectful_enodes: eff_en,
                pure_enodes: pure_en,
            });
        }
        (regions, stats)
    }

    // --- Walk-constrained extraction ---
    pub fn region_extraction_with_state_walk(
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
            let r_idx = *rsub.orig_to_region.get(cid)?;
            let orig_ec = &rsub.egraph.eclasses[r_idx];
            if *en_idx >= orig_ec.enodes.len() {
                return None;
            }
            let chosen = &orig_ec.enodes[*en_idx];
            let target = if orig_ec.is_effectful
                && new_classes[base_map[r_idx].unwrap()].enodes.is_empty()
            {
                base_map[r_idx].unwrap()
            } else if orig_ec.is_effectful {
                let dup = new_classes.len();
                new_classes.push(TigerEClass {
                    enodes: vec![],
                    is_effectful: true,
                    original: orig_ec.original.clone(),
                });
                dup
            } else {
                base_map[r_idx].unwrap()
            };
            let mut new_children = Vec::new();
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
            new_classes[target].enodes.push(TigerENode {
                head: chosen.head.clone(),
                eclass_idx: target,
                children: new_children,
                original_class: cid.clone(),
                original_node: chosen.original_node.clone(),
            });
            last_idx = Some(target);
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
        // SCost extraction on reconstructed graph
        let n = g.eclasses.len();
        let mut dis: Vec<IndexSet<usize>> = vec![IndexSet::new(); n];
        let mut pick: Vec<Option<usize>> = vec![None; n];
        let mut rev: Vec<Vec<(usize, usize)>> = vec![vec![]; n];
        let mut counters: Vec<Vec<(usize, IndexSet<usize>)>> = Vec::with_capacity(n);
        for (i, ec) in g.eclasses.iter().enumerate() {
            let mut ec_c = Vec::with_capacity(ec.enodes.len());
            for (j, en) in ec.enodes.iter().enumerate() {
                ec_c.push((en.children.len(), IndexSet::from([i])));
                for &ch in &en.children {
                    rev[ch].push((i, j));
                }
            }
            counters.push(ec_c);
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
            for &(p_ec, p_en) in &rev[i] {
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
        let mut in_ex = vec![false; n];
        let mut q = VecDeque::new();
        in_ex[root_new] = true;
        q.push_back(root_new);
        while let Some(c) = q.pop_front() {
            if let Some(pe) = pick[c] {
                let en = &g.eclasses[c].enodes[pe];
                for &ch in &en.children {
                    if !in_ex[ch] {
                        in_ex[ch] = true;
                        q.push_back(ch);
                    }
                }
            }
        }
        let mut ord: Vec<(usize, usize)> = in_ex
            .iter()
            .enumerate()
            .filter(|(_, f)| **f)
            .map(|(i, _)| (dis[i].len(), i))
            .collect();
        ord.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let mut extraction = TigerExtraction::new();
        let mut map: Vec<Option<usize>> = vec![None; n];
        for &(_sz, ec_idx) in &ord {
            let Some(chosen) = pick[ec_idx] else { continue };
            let en = &g.eclasses[ec_idx].enodes[chosen];
            let mut child_indices = Vec::new();
            for &ch in &en.children {
                if let Some(ci) = map[ch] {
                    child_indices.push(ci);
                }
            }
            let ex_idx = extraction.add_node(TigerExtractionENode {
                eclass: g.eclasses[ec_idx].original.clone(),
                enode_index: chosen,
                children: child_indices,
            });
            map[ec_idx] = Some(ex_idx);
        }
        extraction.root_index = map[root_new];
        Some(extraction)
    }

    // --- Unguided search ---
    // unguided_find_state_walk_region moved to tiger_extractor_statewalk

    pub fn saturate_pure_counters(&self, seed: &ExtractableSet) -> ExtractableSet {
        let n = self.tiger.eclasses.len();
        let mut ret = vec![false; n];
        let mut edges: Vec<Vec<(usize, usize)>> = vec![vec![]; n];
        let mut counters: Vec<Vec<usize>> = vec![vec![]; n];
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

    // --- Reachability & naive extraction ---
    pub fn effectful_reachable(&self, root: &ClassId) -> IndexSet<ClassId> {
        let mut seen = IndexSet::new();
        let mut q = VecDeque::new();
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
    pub fn naive_extraction(&self, root: &ClassId) -> TigerExtraction {
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
            let chosen = 0usize;
            let en = &tec.enodes[chosen];
            let mut child_indices = Vec::new();
            for &ch_ti in &en.children {
                let ch_ec = &this.tiger.eclasses[ch_ti];
                let ch_cid = &ch_ec.original;
                if let Some(ci) = rec(this, ch_cid, memo, ext) {
                    child_indices.push(ci);
                }
            }
            let idx = ext.add_node(TigerExtractionENode {
                eclass: cid.clone(),
                enode_index: chosen,
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
}
