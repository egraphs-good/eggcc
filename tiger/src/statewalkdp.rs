// Port of statewalkdp.h / statewalkdp.cpp — DP over statewalks.
// Direct line-by-line translation.

use std::collections::VecDeque;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use crate::egraphin::{EClass, EClassId, EGraph, ENode, ENodeId, UNEXTRACTABLE_ECLASS};
use crate::persistent_btree::{PBId, PersistentBitSet, PersistentDecArray, DataType};

pub type Cost = u64;

pub type Statewalk = Vec<(EClassId, ENodeId)>;

pub type StatewalkWidthStat = Vec<usize>;

type HashType = u64;

type DPId = i32;

#[derive(Clone)]
struct DPValue {
    c: Cost,
    root: PBId,
    prev: DPId,
    ec: EClassId,
    pick: ENodeId,
}

impl DPValue {
    fn new(c: Cost, root: PBId, prev: DPId, ec: EClassId, pick: ENodeId) -> Self {
        DPValue { c, root, prev, ec, pick }
    }
}

#[derive(Clone)]
struct BitsetExtraInfo {
    true_hash: HashType,
    masked_hash: HashType,
    array: PBId,
}

impl BitsetExtraInfo {
    fn new(true_hash: HashType, masked_hash: HashType, array: PBId) -> Self {
        BitsetExtraInfo { true_hash, masked_hash, array }
    }
}

// Bit-exact mt19937_64 via the rand_mt crate. `Mt64::new_unseeded()` uses the
// C++ standard default seed (5489), matching `std::mt19937_64()`.
use rand_mt::Mt64;

pub fn statewalkDP(
    g: &EGraph,
    root: EClassId,
    statewalk_cost: &Vec<Vec<Cost>>,
    use_liveness: bool,
    use_satellite_opt: bool,
    stat: Option<&mut StatewalkWidthStat>,
) -> Statewalk {
    let mut stat = stat;

    // file-scope statics in C++ — kept local here since `init()` resets them.
    let mut enode_cnt_pool: PersistentDecArray = PersistentDecArray::new();
    let mut true_extractable_pool: PersistentBitSet = PersistentBitSet::new();

    // find arg
    crate::debug_assert_tiger!(crate::debug::arg_check_regionalized_egraph(g));
    let mut argc: EClassId = UNEXTRACTABLE_ECLASS;
    let mut argn: ENodeId = UNEXTRACTABLE_ECLASS;
    {
        let mut i: EClassId = 0;
        while i < (g.neclasses() as EClassId) && argc == UNEXTRACTABLE_ECLASS {
            let c: &EClass = &g.eclasses[i as usize];
            if c.isEffectful {
                let mut j: ENodeId = 0;
                while j < (c.nenodes() as ENodeId) && argc == UNEXTRACTABLE_ECLASS {
                    let n: &ENode = &c.enodes[j as usize];
                    if n.ch.len() == 0 {
                        argc = i;
                        argn = j;
                    }
                    j += 1;
                }
            }
            i += 1;
        }
    }

    let init_cost: Cost = statewalk_cost[argc as usize][argn as usize];

    // prepare for heavy duty data structures
    let mut parent_edge_to_pure: Vec<Vec<(EClassId, ENodeId)>> = vec![Vec::new(); g.neclasses()];
    let mut parent_edge_to_effectful: Vec<Vec<(EClassId, ENodeId)>> = vec![Vec::new(); g.neclasses()];
    let mut enode_cnt: Vec<Vec<u32>> = vec![Vec::new(); g.neclasses()];

    for i in 0..(g.neclasses() as EClassId) {
        let c: &EClass = &g.eclasses[i as usize];
        if c.isEffectful {
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                for k in 0..n.ch.len() {
                    if g.eclasses[n.ch[k] as usize].isEffectful {
                        parent_edge_to_effectful[n.ch[k] as usize].push((i, j));
                    }
                }
            }
        } else {
            enode_cnt[i as usize].resize(c.nenodes(), 0);
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                enode_cnt[i as usize][j as usize] = n.ch.len() as u32;
                for k in 0..n.ch.len() {
                    parent_edge_to_pure[n.ch[k] as usize].push((i, j));
                }
            }
        }
    }

    let mut q: VecDeque<EClassId> = VecDeque::new();
    let mut init_extractable: Vec<bool> = vec![false; g.neclasses()];

    init_extractable[argc as usize] = true;
    q.push_back(argc);

    for i in 0..(g.neclasses() as EClassId) {
        let c: &EClass = &g.eclasses[i as usize];
        if !c.isEffectful {
            let mut extractable: bool = false;
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                if n.ch.len() == 0 {
                    extractable = true;
                    break;
                }
            }
            if extractable {
                init_extractable[i as usize] = true;
                q.push_back(i);
            }
        }
    }

    while q.len() > 0 {
        let u: EClassId = *q.front().unwrap();
        q.pop_front();
        for i in 0..parent_edge_to_pure[u as usize].len() {
            let vc: EClassId = parent_edge_to_pure[u as usize][i].0;
            let vn: ENodeId = parent_edge_to_pure[u as usize][i].1;
            enode_cnt[vc as usize][vn as usize] -= 1;
            if enode_cnt[vc as usize][vn as usize] == 0 {
                if !init_extractable[vc as usize] {
                    init_extractable[vc as usize] = true;
                    q.push_back(vc);
                }
            }
        }
    }

    // flatten egraphin format
    let mut rnk: Vec<i32> = vec![0; g.neclasses()];
    let mut init_cnt: Vec<u32> = Vec::new();
    let mut compressed_eclass_id: Vec<i32> = vec![-1; g.neclasses()];
    // be very careful that the compressed eclasses do not include the arg eclass
    let mut inv_compressed_eclass_id: Vec<EClassId> = Vec::new();
    for i in 0..(g.neclasses() as EClassId) {
        if init_extractable[i as usize] {
            enode_cnt[i as usize].clear();
            continue;
        }
        compressed_eclass_id[i as usize] = inv_compressed_eclass_id.len() as i32;
        inv_compressed_eclass_id.push(i);
        if enode_cnt[i as usize].len() > 0 {
            rnk[i as usize] = init_cnt.len() as i32;
            let to_extend: Vec<u32> = enode_cnt[i as usize].clone();
            init_cnt.extend(to_extend.into_iter());
        }
    }

    // crazy optimization to use only 2 bits for all in-degree counters
    for i in 0..init_cnt.len() {
        crate::debug_assert_tiger!(init_cnt[i] > 0);
        init_cnt[i] -= 1;
        crate::debug_assert_tiger!(init_cnt[i] <= 3);
    }
    let ncompressed_eclass: usize = inv_compressed_eclass_id.len();

    let init_cnt_id: PBId = enode_cnt_pool.init(&init_cnt);
    let init_extractable_id: PBId = true_extractable_pool.init(&vec![0 as DataType; ncompressed_eclass]);

    // crazy hash scheme
    let mut base_vectors: Vec<HashType> = vec![0; ncompressed_eclass];
    let mut rand_gen: Mt64 = Mt64::new_unseeded();
    for i in 0..base_vectors.len() {
        base_vectors[i] = rand_gen.next_u64();
    }
    let init_true_hash: HashType = 0;
    let init_masked_hash: HashType = 0;

    // liveness

    let mut liveness: Vec<Vec<u64>> = vec![Vec::new(); g.neclasses()];
    let mut liveness_delta: Vec<HashMap<EClassId, Vec<i32>>> = vec![HashMap::new(); g.neclasses()];

    if use_liveness {
        for i in 0..(g.neclasses() as EClassId) {
            let c: &EClass = &g.eclasses[i as usize];
            if c.isEffectful {
                liveness[i as usize].resize((g.neclasses() + 63) >> 6, 0);
                let mut q: VecDeque<EClassId> = VecDeque::new();
                q.push_back(i);
                while q.len() > 0 {
                    let u: EClassId = *q.front().unwrap();
                    q.pop_front();
                    if g.eclasses[u as usize].isEffectful && u != root {
                        for j in 0..parent_edge_to_effectful[u as usize].len() {
                            let v: EClassId = parent_edge_to_effectful[u as usize][j].0;
                            if !(((liveness[i as usize][(v >> 6) as usize] >> (v & 63)) & 1) != 0) {
                                liveness[i as usize][(v >> 6) as usize] |= 1u64 << (v & 63);
                                q.push_back(v);
                            }
                        }
                    }
                    if ((liveness[i as usize][(u >> 6) as usize] >> (u & 63)) & 1) != 0 {
                        let c: &EClass = &g.eclasses[u as usize];
                        for j in 0..(c.nenodes() as ENodeId) {
                            let n: &ENode = &c.enodes[j as usize];
                            for k in 0..n.ch.len() {
                                let v: EClassId = n.ch[k];
                                if !init_extractable[v as usize] && !g.eclasses[v as usize].isEffectful && !(((liveness[i as usize][(v >> 6) as usize] >> (v & 63)) & 1) != 0) {
                                    liveness[i as usize][(v >> 6) as usize] |= 1u64 << (v & 63);
                                    q.push_back(v);
                                }
                            }
                        }
                    }
                }
            }
        }

        for i in 0..(g.neclasses() as EClassId) {
            let c: &EClass = &g.eclasses[i as usize];
            if c.isEffectful && i != root {
                for j in 0..parent_edge_to_effectful[i as usize].len() {
                    let v: EClassId = parent_edge_to_effectful[i as usize][j].0;
                    if !liveness_delta[i as usize].contains_key(&v) {
                        liveness_delta[i as usize].insert(v, Vec::new());
                        // borrow immutably copies of liveness rows we need
                        let mut l: Vec<i32> = Vec::new();
                        for k in 0..liveness[i as usize].len() {
                            crate::debug_assert_tiger!((liveness[i as usize][k] & liveness[v as usize][k]) == liveness[v as usize][k]);
                            let mut delta: u64 = liveness[i as usize][k] ^ liveness[v as usize][k];
                            while delta != 0 {
                                let lb: u64 = delta & (delta.wrapping_neg());
                                delta ^= lb;
                                let w: EClassId = (lb.trailing_zeros() as EClassId) + ((k as EClassId) << 6);
                                if !g.eclasses[w as usize].isEffectful && !init_extractable[w as usize] {
                                    l.push(compressed_eclass_id[w as usize]);
                                }
                            }
                        }
                        liveness_delta[i as usize].insert(v, l);
                    }
                }
            }
        }
    }

    // AC - satellite eclasses
    let mut satellite_pa: Vec<EClassId> = vec![UNEXTRACTABLE_ECLASS; g.neclasses()];
    let mut satellite_chcnt: Vec<i32> = vec![0; g.neclasses()];
    const SATELLITE_BAR: i32 = 6;
    if use_satellite_opt {
        for i in 0..(g.neclasses() as EClassId) {
            let c: &EClass = &g.eclasses[i as usize];
            if c.isEffectful {
                let mut candidate: EClassId = satellite_pa[i as usize];
                for j in 0..(c.nenodes() as ENodeId) {
                    let n: &ENode = &c.enodes[j as usize];
                    let mut cp: EClassId = UNEXTRACTABLE_ECLASS;
                    for k in 0..n.ch.len() {
                        let ch: EClassId = n.ch[k];
                        if g.eclasses[ch as usize].isEffectful {
                            cp = ch;
                            break;
                        }
                    }
                    if cp == UNEXTRACTABLE_ECLASS {
                        candidate = UNEXTRACTABLE_ECLASS;
                        break;
                    } else if candidate == UNEXTRACTABLE_ECLASS {
                        candidate = cp;
                    } else if candidate != cp {
                        candidate = UNEXTRACTABLE_ECLASS;
                        break;
                    }
                }
                if candidate != UNEXTRACTABLE_ECLASS {
                    if parent_edge_to_effectful[i as usize].len() == 0 {
                        candidate = UNEXTRACTABLE_ECLASS;
                    } else {
                        for j in 0..parent_edge_to_effectful[i as usize].len() {
                            if parent_edge_to_effectful[i as usize][j].0 != candidate {
                                candidate = UNEXTRACTABLE_ECLASS;
                                break;
                            }
                        }
                    }
                }
                satellite_pa[i as usize] = candidate;
            }
        }

        for i in 0..(g.neclasses() as EClassId) {
            if satellite_pa[i as usize] != UNEXTRACTABLE_ECLASS {
                satellite_chcnt[satellite_pa[i as usize] as usize] += 1;
            }
        }
    }


    // main DP data structures

    let mut dpmap: Vec<HashMap<HashType, DPId>> = vec![HashMap::new(); g.neclasses()];
    let mut dp: Vec<DPValue> = Vec::new();
    let mut bitset_extra: HashMap<PBId, BitsetExtraInfo> = HashMap::new();
    let mut unifier: HashMap<HashType, PBId> = HashMap::new();
    let mut pure_saturation_cache: HashMap<HashType, PBId> = HashMap::new();

    dpmap[argc as usize].insert(init_true_hash, dp.len() as DPId);
    dp.push(DPValue::new(init_cost, init_extractable_id, -1, argc, argn));
    bitset_extra.insert(init_extractable_id, BitsetExtraInfo::new(init_true_hash, init_masked_hash, init_cnt_id));
    unifier.insert(init_true_hash, init_extractable_id);
    let mut maxheap: BinaryHeap<(Cost, DPId)> = BinaryHeap::new();
    maxheap.push((!init_cost, 0));
    let mut best_statewalk: DPId = if root == argc { 0 } else { -1 };
    while maxheap.len() > 0 {
        let c: Cost = !maxheap.peek().unwrap().0;
        let uid: DPId = maxheap.peek().unwrap().1;
        maxheap.pop();
        // let the dp saturate if in stat mode
        if stat.is_some() && (best_statewalk != -1 && dp[uid as usize].c == dp[best_statewalk as usize].c) {
            break;
        }
        if dp[uid as usize].c == c && dp[uid as usize].ec != root {
            let u: EClassId = dp[uid as usize].ec;
            let enable_satellite_opt: bool = satellite_chcnt[dp[uid as usize].ec as usize] > SATELLITE_BAR;
            let mut satellite_updated: bool = false;
            for i in 0..parent_edge_to_effectful[u as usize].len() {
                let v: EClassId = parent_edge_to_effectful[u as usize][i].0;
                let vn: ENodeId = parent_edge_to_effectful[u as usize][i].1;
                let is_satellite_update: bool = satellite_pa[v as usize] == dp[uid as usize].ec;
                if enable_satellite_opt && is_satellite_update && satellite_updated {
                    continue;
                }
                // test for validity
                let mut can_extend: bool = true;
                let n_ch: Vec<EClassId> = g.eclasses[v as usize].enodes[vn as usize].ch.clone();
                for j in 0..n_ch.len() {
                    let chc: EClassId = n_ch[j];
                    if !init_extractable[chc as usize] && (true_extractable_pool.getpos(dp[uid as usize].root, compressed_eclass_id[chc as usize]) == false) {
                        can_extend = false;
                        break;
                    }
                }
                if can_extend {
                    // find the new dp state
                    let info: BitsetExtraInfo = bitset_extra.get(&dp[uid as usize].root).unwrap().clone();
                    let nhash: HashType;
                    let mut nroot: PBId;
                    let nc: Cost = c + statewalk_cost[v as usize][vn as usize];
                    if best_statewalk != -1 && dp[best_statewalk as usize].c <= nc {
                        continue;
                    }
                    if init_extractable[v as usize] || true_extractable_pool.getpos(dp[uid as usize].root, compressed_eclass_id[v as usize]) == true {
                        nhash = info.masked_hash;
                        nroot = dp[uid as usize].root;
                    } else {
                        let pschash: HashType = ((dp[uid as usize].root as u64) << 32) | (v as u32 as u64);
                        if pure_saturation_cache.contains_key(&pschash) {
                            nroot = *pure_saturation_cache.get(&pschash).unwrap();
                            nhash = bitset_extra.get(&nroot).unwrap().masked_hash;
                        } else {
                            enode_cnt_pool.new_version();
                            true_extractable_pool.new_version();
                            nroot = dp[uid as usize].root;
                            let mut ninfo: BitsetExtraInfo = info.clone();
                            if use_liveness {
                                //liveness-1
                                let delta: Vec<i32> = liveness_delta[u as usize].get(&v).cloned().unwrap_or_else(Vec::new);
                                for j in 0..delta.len() {
                                    if true_extractable_pool.getpos(nroot, delta[j]) {
                                        ninfo.masked_hash ^= base_vectors[delta[j] as usize];
                                    }
                                }
                            }
                            let nliveness: Vec<u64> = liveness[v as usize].clone();
                            let mut q: VecDeque<EClassId> = VecDeque::new();

                            // saturate pure
                            q.push_back(v);
                            nroot = true_extractable_pool.setpos(nroot, compressed_eclass_id[v as usize]).0;
                            crate::debug_assert_tiger!(true_extractable_pool.getpos(nroot, compressed_eclass_id[v as usize]));
                            ninfo.true_hash ^= base_vectors[compressed_eclass_id[v as usize] as usize];
                            while q.len() > 0 {
                                let u: EClassId = *q.front().unwrap();
                                q.pop_front();
                                for j in 0..parent_edge_to_pure[u as usize].len() {
                                    let v: EClassId = parent_edge_to_pure[u as usize][j].0;
                                    if !init_extractable[v as usize] && true_extractable_pool.getpos(nroot, compressed_eclass_id[v as usize]) == false {
                                        let vn: ENodeId = parent_edge_to_pure[u as usize][j].1;
                                        let eid: i32 = rnk[v as usize] + vn;
                                        let res: (PBId, DataType) = enode_cnt_pool.dec(ninfo.array, eid);
                                        ninfo.array = res.0;
                                        if res.1 == 0 {
                                            let res: (PBId, bool) = true_extractable_pool.setpos(nroot, compressed_eclass_id[v as usize]);
                                            if !res.1 {
                                                nroot = res.0;
                                                ninfo.true_hash ^= base_vectors[compressed_eclass_id[v as usize] as usize];
                                                //liveness-2
                                                if !use_liveness || ((nliveness[(v >> 6) as usize] >> (v & 63)) & 1) != 0 {
                                                    ninfo.masked_hash ^= base_vectors[compressed_eclass_id[v as usize] as usize];
                                                }
                                                q.push_back(v);
                                            }
                                        }
                                    }
                                }
                            }
                            if unifier.contains_key(&ninfo.true_hash) {
                                nroot = *unifier.get(&ninfo.true_hash).unwrap();
                                crate::debug_assert_tiger!(bitset_extra.contains_key(&nroot));
                                let ninfop: BitsetExtraInfo = bitset_extra.get(&nroot).unwrap().clone();
                                crate::debug_assert_tiger!(ninfo.true_hash == ninfop.true_hash);
                                crate::debug_assert_tiger!(ninfo.masked_hash == ninfop.masked_hash);
                            } else {
                                unifier.insert(ninfo.true_hash, nroot);
                                bitset_extra.insert(nroot, ninfo.clone());
                            }
                            pure_saturation_cache.insert(pschash, nroot);
                            nhash = ninfo.masked_hash;
                        }
                    }
                    if enable_satellite_opt && is_satellite_update {
                        if nhash == info.masked_hash {
                            continue;
                        } else {
                            satellite_updated = true;
                        }
                    }
                    // update
                    if !dpmap[v as usize].contains_key(&nhash) {
                        let vid: DPId = dp.len() as DPId;
                        dpmap[v as usize].insert(nhash, vid);
                        dp.push(DPValue::new(nc, nroot, uid, v, vn));
                        maxheap.push((!nc, vid));
                        if v == root {
                            best_statewalk = vid;
                        }
                    } else {
                        let vid: DPId = *dpmap[v as usize].get(&nhash).unwrap();
                        if dp[vid as usize].c > nc {
                            dp[vid as usize].c = nc;
                            dp[vid as usize].root = nroot;
                            dp[vid as usize].prev = uid;
                            dp[vid as usize].pick = vn;
                            maxheap.push((!nc, vid));
                            if v == root {
                                best_statewalk = vid;
                            }
                        }
                    }
                }
            }
        }
    }
    crate::debug_assert_tiger!(best_statewalk != -1);
    // reconstruct statewalk
    let mut sw: Statewalk = Statewalk::new();
    let mut cur: DPId = best_statewalk;
    while cur != -1 {
        sw.push((dp[cur as usize].ec, dp[cur as usize].pick));
        cur = dp[cur as usize].prev;
    }
    crate::debug_assert_tiger!(crate::debug::is_valid_statewalk(g, root, &sw));
    // stats
    if let Some(s) = stat.as_deref_mut() {
        for i in 0..(g.neclasses() as EClassId) {
            if g.eclasses[i as usize].isEffectful {
                s.push(dpmap[i as usize].len());
            }
        }
    }
    sw
}
