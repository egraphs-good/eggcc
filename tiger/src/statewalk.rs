//! The statewalk dynamic program.
//!
//! A *region* is an e-graph in which effectful e-classes form a chain from the
//! region's root down to its `Arg`. Extracting the region means choosing one
//! effectful e-node per e-class on that chain (the *statewalk*) so that every
//! pure term the chosen e-nodes need is extractable from the chosen state, and
//! the total cost is minimal.
//!
//! The DP walks the chain upwards from the `Arg`. A DP state is an effectful
//! e-class together with the set of pure e-classes that are extractable given
//! the statewalk so far (kept in a persistent bit set and hashed, so states
//! with the same extractable set are merged).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use rand_mt::Mt64;
use rustc_hash::FxHashMap;

use crate::cost::Cost;
use crate::egraph::{EClass, EClassId, EGraph, EGraphMapping, ENodeId, Extraction};
use crate::greedy::statewalk_greedy_extraction;
use crate::persistent::{Id as VersionId, PersistentBitSet, PersistentCounters};

/// The chosen effectful e-nodes of a region, from the root down to the `Arg`.
pub type Statewalk = Vec<(EClassId, ENodeId)>;

/// Optimizations of the DP. Both are on by default; the flags exist for experiments.
#[derive(Clone, Copy, Debug)]
pub struct StatewalkOptions {
    /// Ignore pure e-classes that no later part of the statewalk can use when
    /// comparing DP states.
    pub liveness: bool,
    /// Only visit one *satellite* (an effectful e-class whose only effectful
    /// parent and child are the same e-class) per DP state unless it changes
    /// the extractable set.
    pub satellite: bool,
}

impl Default for StatewalkOptions {
    fn default() -> Self {
        StatewalkOptions {
            liveness: true,
            satellite: true,
        }
    }
}

type Hash = u64;
type DpId = usize;

struct DpState {
    cost: Cost,
    /// Version of the extractable-set bit set for this state.
    extractable: VersionId,
    prev: Option<DpId>,
    class: EClassId,
    pick: ENodeId,
}

#[derive(Clone)]
struct VersionInfo {
    /// Hash of the full extractable set.
    true_hash: Hash,
    /// Hash of the extractable set restricted to live e-classes.
    masked_hash: Hash,
    /// Version of the child counters for this extractable set.
    counters: VersionId,
}

/// Satellites are only skipped for e-classes with more than this many of them.
const SATELLITE_BAR: usize = 6;

fn bit(bits: &[u64], i: usize) -> bool {
    (bits[i >> 6] >> (i & 63)) & 1 != 0
}

fn set_bit(bits: &mut [u64], i: usize) {
    bits[i >> 6] |= 1 << (i & 63);
}

/// Everything about a region the DP needs that does not depend on the DP state.
struct Region {
    arg: (EClassId, ENodeId),
    /// Pure e-nodes with the e-class as a child.
    parents_pure: Vec<Vec<(EClassId, ENodeId)>>,
    /// Effectful e-nodes with the e-class as a child.
    parents_effectful: Vec<Vec<(EClassId, ENodeId)>>,
    /// E-classes extractable from the `Arg` alone.
    init_extractable: Vec<bool>,
    /// Dense index of the e-classes that are not initially extractable.
    compressed: Vec<Option<usize>>,
    /// Offset of a pure e-class's e-nodes in the counter array.
    rank: Vec<usize>,
    /// Random hash contribution of each compressed e-class.
    base: Vec<Hash>,
    /// Per effectful e-class, the e-classes live at it (effectful ancestors and
    /// the pure e-classes they use). Empty when liveness is off.
    live: Vec<Vec<u64>>,
    /// `live_delta[c][p]`: compressed pure e-classes live at `c` but not at its
    /// effectful parent `p`.
    live_delta: Vec<FxHashMap<EClassId, Vec<usize>>>,
    /// The e-class an effectful e-class is a satellite of, if any.
    satellite_of: Vec<Option<EClassId>>,
    satellite_count: Vec<usize>,
}

impl Region {
    fn new(g: &EGraph, root: EClassId, opts: StatewalkOptions) -> (Self, Vec<u32>) {
        debug_assert!(crate::checks::has_single_arg(g));
        let arg = g
            .class_ids()
            .filter(|&c| g.is_effectful(c))
            .find_map(|c| {
                g.classes[c]
                    .enodes
                    .iter()
                    .position(|n| n.is_leaf())
                    .map(|n| (c, n))
            })
            .expect("region has no Arg");

        let n = g.len();
        let mut parents_pure = vec![Vec::new(); n];
        let mut parents_effectful = vec![Vec::new(); n];
        let mut child_count: Vec<Vec<u32>> = vec![Vec::new(); n];
        for (c, node) in g.enode_ids() {
            let enode = g.enode(c, node);
            if g.is_effectful(c) {
                for &child in &enode.children {
                    if g.is_effectful(child) {
                        parents_effectful[child].push((c, node));
                    }
                }
            } else {
                child_count[c].push(enode.children.len() as u32);
                for &child in &enode.children {
                    parents_pure[child].push((c, node));
                }
            }
        }

        // Extractable from the Arg alone: the Arg, pure leaves, and their closure.
        let mut init_extractable = vec![false; n];
        let mut queue = VecDeque::new();
        init_extractable[arg.0] = true;
        queue.push_back(arg.0);
        for c in g.class_ids() {
            if !g.is_effectful(c) && g.classes[c].enodes.iter().any(|e| e.is_leaf()) {
                init_extractable[c] = true;
                queue.push_back(c);
            }
        }
        while let Some(u) = queue.pop_front() {
            for &(pc, pn) in &parents_pure[u] {
                child_count[pc][pn] -= 1;
                if child_count[pc][pn] == 0 && !init_extractable[pc] {
                    init_extractable[pc] = true;
                    queue.push_back(pc);
                }
            }
        }

        // Dense numbering of the remaining e-classes, and the flat counter array
        // of the remaining pure e-nodes' children.
        let mut compressed = vec![None; n];
        let mut rank = vec![0; n];
        let mut counts: Vec<u32> = Vec::new();
        let mut n_compressed = 0;
        for c in g.class_ids() {
            if init_extractable[c] {
                continue;
            }
            compressed[c] = Some(n_compressed);
            n_compressed += 1;
            if !child_count[c].is_empty() {
                rank[c] = counts.len();
                counts.extend_from_slice(&child_count[c]);
            }
        }
        // The counters are two bits wide and store (children - 1).
        for count in &mut counts {
            debug_assert!(*count > 0);
            *count -= 1;
            debug_assert!(*count <= 3);
        }

        // Random hash contributions; mt19937_64 with its default seed, like the C++.
        let mut rng = Mt64::new_unseeded();
        let base: Vec<Hash> = (0..n_compressed).map(|_| rng.next_u64()).collect();

        let mut live: Vec<Vec<u64>> = vec![Vec::new(); n];
        let mut live_delta: Vec<FxHashMap<EClassId, Vec<usize>>> = vec![FxHashMap::default(); n];
        if opts.liveness {
            let words = n.div_ceil(64);
            for i in g.class_ids().filter(|&c| g.is_effectful(c)) {
                live[i] = vec![0; words];
                let mut queue = VecDeque::from([i]);
                while let Some(u) = queue.pop_front() {
                    if g.is_effectful(u) && u != root {
                        for &(v, _) in &parents_effectful[u] {
                            if !bit(&live[i], v) {
                                set_bit(&mut live[i], v);
                                queue.push_back(v);
                            }
                        }
                    }
                    if bit(&live[i], u) {
                        for enode in &g.classes[u].enodes {
                            for &v in &enode.children {
                                if !init_extractable[v] && !g.is_effectful(v) && !bit(&live[i], v) {
                                    set_bit(&mut live[i], v);
                                    queue.push_back(v);
                                }
                            }
                        }
                    }
                }
            }
            for i in g.class_ids().filter(|&c| g.is_effectful(c) && c != root) {
                for &(v, _) in &parents_effectful[i] {
                    live_delta[i].entry(v).or_insert_with(|| {
                        let mut delta = Vec::new();
                        for k in 0..words {
                            debug_assert!(live[i][k] & live[v][k] == live[v][k]);
                            let mut diff = live[i][k] ^ live[v][k];
                            while diff != 0 {
                                let w = (k << 6) + diff.trailing_zeros() as usize;
                                diff &= diff - 1;
                                if !g.is_effectful(w) && !init_extractable[w] {
                                    delta.push(compressed[w].unwrap());
                                }
                            }
                        }
                        delta
                    });
                }
            }
        }

        let mut satellite_of = vec![None; n];
        let mut satellite_count = vec![0; n];
        if opts.satellite {
            for i in g.class_ids().filter(|&c| g.is_effectful(c)) {
                // A satellite's e-nodes all share one effectful child, which is
                // also its only effectful parent.
                let mut candidate: Option<EClassId> = None;
                let mut consistent = true;
                for enode in &g.classes[i].enodes {
                    match g.effectful_child(enode) {
                        None => {
                            consistent = false;
                            break;
                        }
                        Some(child) => {
                            if candidate.is_none() {
                                candidate = Some(child);
                            } else if candidate != Some(child) {
                                consistent = false;
                                break;
                            }
                        }
                    }
                }
                let candidate = candidate.filter(|&c| {
                    consistent
                        && !parents_effectful[i].is_empty()
                        && parents_effectful[i].iter().all(|&(p, _)| p == c)
                });
                satellite_of[i] = candidate;
            }
            for c in satellite_of.iter().flatten() {
                satellite_count[*c] += 1;
            }
        }

        let region = Region {
            arg,
            parents_pure,
            parents_effectful,
            init_extractable,
            compressed,
            rank,
            base,
            live,
            live_delta,
            satellite_of,
            satellite_count,
        };
        (region, counts)
    }
}

/// The DP's mutable state: the versioned extractable sets and counters, and
/// the hash-consing tables over them.
struct Versions {
    sets: PersistentBitSet,
    counters: PersistentCounters,
    info: FxHashMap<VersionId, VersionInfo>,
    /// Version with a given full extractable set.
    by_true_hash: FxHashMap<Hash, VersionId>,
    /// Result of extending a version with an effectful e-class.
    saturated: FxHashMap<(VersionId, EClassId), VersionId>,
}

impl Versions {
    fn is_extractable(&self, region: &Region, version: VersionId, class: EClassId) -> bool {
        region.init_extractable[class]
            || self.sets.contains(
                version,
                region.compressed[class].expect("class is compressed"),
            )
    }

    /// Extend `version` by making the effectful e-class `v` extractable (coming
    /// from its effectful child `u`), then saturate: every pure e-node whose
    /// children are all extractable makes its e-class extractable too.
    /// Returns the new version and its masked hash.
    fn saturate(
        &mut self,
        region: &Region,
        opts: StatewalkOptions,
        version: VersionId,
        u: EClassId,
        v: EClassId,
    ) -> (VersionId, Hash) {
        if let Some(&cached) = self.saturated.get(&(version, v)) {
            return (cached, self.info[&cached].masked_hash);
        }
        self.sets.new_version();
        self.counters.new_version();
        let mut info = self.info[&version].clone();
        let mut new_version = version;
        if opts.liveness {
            // E-classes that are live at u but dead at v drop out of the masked hash.
            if let Some(delta) = region.live_delta[u].get(&v) {
                for &d in delta {
                    if self.sets.contains(new_version, d) {
                        info.masked_hash ^= region.base[d];
                    }
                }
            }
        }
        let live_at_v = &region.live[v];
        let cv = region.compressed[v].expect("v is not initially extractable");
        new_version = self.sets.insert(new_version, cv).0;
        info.true_hash ^= region.base[cv];
        let mut queue = VecDeque::from([v]);
        while let Some(w) = queue.pop_front() {
            for &(pc, pn) in &region.parents_pure[w] {
                if region.init_extractable[pc] {
                    continue;
                }
                let cpc = region.compressed[pc].unwrap();
                if self.sets.contains(new_version, cpc) {
                    continue;
                }
                let (counters, before) =
                    self.counters.decrement(info.counters, region.rank[pc] + pn);
                info.counters = counters;
                // The counter stores (children - 1), so hitting zero means every child is extractable.
                if before == 0 {
                    let (set, already) = self.sets.insert(new_version, cpc);
                    if !already {
                        new_version = set;
                        info.true_hash ^= region.base[cpc];
                        if !opts.liveness || bit(live_at_v, pc) {
                            info.masked_hash ^= region.base[cpc];
                        }
                        queue.push_back(pc);
                    }
                }
            }
        }
        match self.by_true_hash.get(&info.true_hash) {
            Some(&existing) => {
                debug_assert!(self.info[&existing].true_hash == info.true_hash);
                debug_assert!(self.info[&existing].masked_hash == info.masked_hash);
                new_version = existing;
            }
            None => {
                self.by_true_hash.insert(info.true_hash, new_version);
                self.info.insert(new_version, info.clone());
            }
        }
        self.saturated.insert((version, v), new_version);
        (new_version, info.masked_hash)
    }
}

/// The cheapest statewalk of the region `g` from `root` down to its `Arg`.
/// `costs[c][n]` is the statewalk cost of effectful e-node `(c, n)`.
pub fn statewalk_dp(
    g: &EGraph,
    root: EClassId,
    costs: &[Vec<Cost>],
    opts: StatewalkOptions,
) -> Statewalk {
    let (region, counts) = Region::new(g, root, opts);
    let n_compressed = region.compressed.iter().flatten().count();

    let mut versions = Versions {
        sets: PersistentBitSet::default(),
        counters: PersistentCounters::default(),
        info: FxHashMap::default(),
        by_true_hash: FxHashMap::default(),
        saturated: FxHashMap::default(),
    };
    let init_counters = versions.counters.init(&counts);
    let init_set = versions.sets.init(&vec![0; n_compressed]);
    versions.info.insert(
        init_set,
        VersionInfo {
            true_hash: 0,
            masked_hash: 0,
            counters: init_counters,
        },
    );
    versions.by_true_hash.insert(0, init_set);

    let (arg_class, arg_node) = region.arg;
    let mut states: Vec<DpState> = vec![DpState {
        cost: costs[arg_class][arg_node],
        extractable: init_set,
        prev: None,
        class: arg_class,
        pick: arg_node,
    }];
    let mut state_by_hash: Vec<FxHashMap<Hash, DpId>> = vec![FxHashMap::default(); g.len()];
    state_by_hash[arg_class].insert(0, 0);
    let mut heap: BinaryHeap<(Reverse<Cost>, DpId)> = BinaryHeap::new();
    heap.push((Reverse(states[0].cost), 0));
    let mut best: Option<DpId> = (root == arg_class).then_some(0);

    while let Some((Reverse(cost), uid)) = heap.pop() {
        if states[uid].cost != cost || states[uid].class == root {
            continue;
        }
        let u = states[uid].class;
        let u_version = states[uid].extractable;
        let skip_satellites = region.satellite_count[u] > SATELLITE_BAR;
        let mut satellite_visited = false;
        for &(v, vn) in &region.parents_effectful[u] {
            let is_satellite = region.satellite_of[v] == Some(u);
            if skip_satellites && is_satellite && satellite_visited {
                continue;
            }
            let enode = g.enode(v, vn);
            if !enode
                .children
                .iter()
                .all(|&c| versions.is_extractable(&region, u_version, c))
            {
                continue;
            }
            let new_cost = cost + costs[v][vn];
            if best.is_some_and(|b| states[b].cost <= new_cost) {
                continue;
            }
            let old_hash = versions.info[&u_version].masked_hash;
            let (new_version, new_hash) = if versions.is_extractable(&region, u_version, v) {
                (u_version, old_hash)
            } else {
                versions.saturate(&region, opts, u_version, u, v)
            };
            if skip_satellites && is_satellite {
                if new_hash == old_hash {
                    continue;
                }
                satellite_visited = true;
            }
            match state_by_hash[v].get(&new_hash) {
                None => {
                    let vid = states.len();
                    state_by_hash[v].insert(new_hash, vid);
                    states.push(DpState {
                        cost: new_cost,
                        extractable: new_version,
                        prev: Some(uid),
                        class: v,
                        pick: vn,
                    });
                    heap.push((Reverse(new_cost), vid));
                    if v == root {
                        best = Some(vid);
                    }
                }
                Some(&vid) => {
                    if states[vid].cost > new_cost {
                        states[vid] = DpState {
                            cost: new_cost,
                            extractable: new_version,
                            prev: Some(uid),
                            class: v,
                            pick: vn,
                        };
                        heap.push((Reverse(new_cost), vid));
                        if v == root {
                            best = Some(vid);
                        }
                    }
                }
            }
        }
    }

    let mut statewalk = Statewalk::new();
    let mut cur = Some(best.expect("no statewalk reaches the root"));
    while let Some(id) = cur {
        statewalk.push((states[id].class, states[id].pick));
        cur = states[id].prev;
    }
    debug_assert!(crate::checks::is_valid_statewalk(g, root, &statewalk));
    statewalk
}

/// Linearize a region along a statewalk: every effectful e-class keeps only its
/// chosen e-node, whose effectful child is redirected to the next e-class on the
/// walk. An e-class visited more than once gets a fresh copy per visit.
/// Returns the linearized e-graph and the mapping back into `g`.
pub fn linearize(g: &EGraph, statewalk: &Statewalk) -> (EGraph, EGraphMapping) {
    let mut lin = EGraph {
        classes: g
            .classes
            .iter()
            .map(|class| {
                if class.is_effectful {
                    EClass {
                        enodes: Vec::new(),
                        is_effectful: true,
                    }
                } else {
                    class.clone()
                }
            })
            .collect(),
    };
    let mut to_g = EGraphMapping {
        classes: g.class_ids().map(Some).collect(),
        enodes: g
            .classes
            .iter()
            .map(|class| {
                if class.is_effectful {
                    Vec::new()
                } else {
                    (0..class.enodes.len()).map(Some).collect()
                }
            })
            .collect(),
    };
    let mut prev: Option<EClassId> = None;
    for &(c, n) in statewalk.iter().rev() {
        let mut enode = g.enode(c, n).clone();
        for child in &mut enode.children {
            if g.is_effectful(*child) {
                *child = prev.expect("the statewalk ends at the Arg");
            }
        }
        let lin_class = if lin.classes[c].enodes.is_empty() {
            lin.classes[c].enodes.push(enode);
            to_g.enodes[c].push(Some(n));
            c
        } else {
            lin.classes.push(EClass {
                enodes: vec![enode],
                is_effectful: true,
            });
            to_g.classes.push(Some(c));
            to_g.enodes.push(vec![Some(n)]);
            lin.len() - 1
        };
        prev = Some(lin_class);
    }
    debug_assert!(crate::checks::is_wellformed(&lin, true, false));
    debug_assert!(crate::checks::is_valid_mapping(
        &to_g, &lin, g, false, false, false, true
    ));
    (lin, to_g)
}

/// Extract `root` from the region `g`: find the cheapest statewalk, linearize
/// along it, and greedily extract the pure terms it needs.
pub fn extract_region(
    g: &EGraph,
    root: EClassId,
    costs: &[Vec<Cost>],
    opts: StatewalkOptions,
) -> Extraction {
    let statewalk = statewalk_dp(g, root, costs, opts);
    let (lin, lin_to_g) = linearize(g, &statewalk);
    // The root keeps its id in the linearized e-graph.
    let (pruned, lin_to_pruned) = lin.prune_unextractable(Some(root));
    let extraction = statewalk_greedy_extraction(&pruned, lin_to_pruned.class(root));
    let extraction = lin_to_pruned
        .inverse(&pruned)
        .then(&lin_to_g)
        .apply(&extraction);
    debug_assert!(crate::checks::is_effect_safe(g, root, &extraction));
    extraction
}
