// Port of greedy.h / greedy.cpp — bag-based greedy cost + statewalk greedy extraction.
// Direct line-by-line translation.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use indexmap::IndexMap;

use crate::egraphin::{
    compute_reverse_index, EClassId, EGraph, EGraphMapping, ENode, ENodeId, Extraction,
    ExtractionENode, ExtractionENodeId, UNEXTRACTABLE_ECLASS,
};

pub type Cost = u64;

pub const INF: Cost = u64::MAX;

// bag-based greedy cost
// it is unstable because it does not guarantee the lowest cost
// but that is ok for getting an estimate

#[derive(Clone)]
pub struct SCost {
    pub sum: Cost,
    pub bag: IndexMap<EClassId, Cost>,
}

impl SCost {
    pub fn new(v: Cost) -> Self {
        SCost {
            sum: v,
            bag: IndexMap::new(),
        }
    }

    // Mirror of C++ `SCost& operator += (SCost &a, const pair<EClassId, SCost> &cb)`.
    pub fn add_assign_pair(&mut self, cb: &(EClassId, SCost)) {
        let mut overhead: Cost = cb.1.sum;
        for (cid_ref, c_ref) in cb.1.bag.iter() {
            let cid: EClassId = *cid_ref;
            let c: Cost = *c_ref;
            overhead -= c;
            if self.bag.contains_key(&cid) {
                if self.bag[&cid] > c {
                    self.sum -= self.bag[&cid] - c;
                    self.bag.insert(cid, c);
                }
            } else {
                self.bag.insert(cid, c);
                self.sum += c;
            }
        }
        if self.bag.contains_key(&cb.0) {
            if self.bag[&cb.0] > overhead {
                self.sum -= self.bag[&cb.0] - overhead;
                self.bag.insert(cb.0, overhead);
            }
        } else {
            self.bag.insert(cb.0, overhead);
            self.sum += overhead;
        }
    }
}

impl PartialEq for SCost {
    fn eq(&self, other: &Self) -> bool {
        self.sum == other.sum
    }
}

impl Eq for SCost {}

impl PartialOrd for SCost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SCost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sum.cmp(&other.sum)
    }
}

pub fn isPrimitive(name: &str) -> bool {
    name.len() > 9 && &name[..9] == "primitive"
}

pub fn isType(op: &str) -> bool {
    op == "IntT" || op == "BoolT" || op == "FloatT"
        || op == "PointerT" || op == "StateT" || op == "Base"
        || op == "TupleT" || op == "TNil" || op == "TCons"
}

pub fn get_enode_cost(n: &ENode) -> Cost {
    let name: String = n.get_name();
    let op: String = n.get_op();
    if op == "Const" {
        return 10;
    } else if op == "Arg" || isPrimitive(&name) || isType(&op) || op == "Int" || op == "Bool" || op == "Float" {
        return 0;
    } else if op == "Empty" || op == "Single" || op == "Concat" || op == "Nil" || op == "Cons" {
        return 0;
    } else if op == "Get" {
        return 1;
    } else if op == "Abs" || op == "Bitand" || op == "Neg" || op == "Add" || op == "PtrAdd" || op == "Sub" || op == "And" || op == "Or" || op == "Not" || op == "Shl" || op == "Shr" {
        return 100;
    } else if op == "FAdd" || op == "FSub" || op == "Fmax" || op == "Fmin" {
        return 500;
    } else if op == "Mul" {
        return 300;
    } else if op == "FMul" {
        return 1500;
    } else if op == "Div" {
        return 500;
    } else if op == "FDiv" {
        return 2500;
    } else if op == "Eq" || op == "LessThan" || op == "GreaterThan" || op == "LessEq" || op == "GreaterEq" {
        return 100;
    } else if op == "Select" {
        return 1500;
    } else if op == "Smax" || op == "Smin" || op == "FEq" {
        return 100;
    } else if op == "FLessThan" || op == "FGreaterThan" || op == "FLessEq" || op == "FGreaterEq" {
        return 1000;
    } else if op == "Print" || op == "Write" || op == "Load" {
        return 500;
    } else if op == "Alloc" || op == "Free" {
        return 1000;
    } else if op == "Call" {
        return 500000;
    } else if op == "Program" || op == "Function" {
        return 0;
    // special treatment for loops and ifs somewhere else
    } else if op == "DoWhile" {
        return 1;
    } else if op == "If" || op == "Switch" {
        return 250;
    } else if op == "Uop" || op == "Bop" || op == "Top" {
        return 0;
    } else {
        crate::debug_cerr!("Encountered op of unknown cost: {} {}", name, op);
        crate::debug_assert_tiger!(false);
        return 0;
    }
}

// extract all eclasses if root is -1
pub fn greedy_extract_compute_eclasses_pick(g: &EGraph, root: EClassId) -> (Vec<ENodeId>, Vec<Cost>) {
    let mut pick: Vec<ENodeId> = vec![-1; g.neclasses()];
    let mut dis: Vec<SCost> = vec![SCost::new(INF); g.neclasses()];
    // C++ uses a max-heap on `~sum` (bitwise NOT). Mirror that exactly with
    // `Cost::MAX - sum` (i.e. `!sum` for u64) so .peek() returns the entry
    // with the smallest `sum`.
    let mut maxheap: BinaryHeap<(Cost, EClassId)> = BinaryHeap::new();
    // Optimization
    let parents: Vec<Vec<(EClassId, ENodeId)>> = compute_reverse_index(g);
    let mut cnt: Vec<Vec<i32>> = vec![Vec::new(); g.neclasses()];
    // Initialize nodes
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        cnt[i as usize].resize(c.nenodes(), 0);
        for j in 0..(c.nenodes() as ENodeId) {
            let n: &ENode = &c.enodes[j as usize];
            cnt[i as usize][j as usize] = n.ch.len() as i32;
            if cnt[i as usize][j as usize] == 0 {
                let ndis: SCost = SCost::new(get_enode_cost(n));
                if ndis < dis[i as usize] {
                    dis[i as usize] = ndis;
                    pick[i as usize] = j;
                    maxheap.push((!dis[i as usize].sum, i));
                }
            }
        }
    }
    while maxheap.len() > 0 {
        let d: Cost = !maxheap.peek().unwrap().0;
        let i: EClassId = maxheap.peek().unwrap().1;
        if i == root {
            break;
        }
        maxheap.pop();
        if d == dis[i as usize].sum {
            for j in 0..parents[i as usize].len() {
                let pc: EClassId = parents[i as usize][j].0;
                let pn: ENodeId = parents[i as usize][j].1;
                cnt[pc as usize][pn as usize] -= 1;
                if cnt[pc as usize][pn as usize] == 0 {
                    let n: &ENode = &g.eclasses[pc as usize].enodes[pn as usize];
                    let mut ndis: SCost = SCost::new(0);
                    if n.get_op() == "If" {
                        crate::debug_assert_tiger!(n.ch.len() == 4);
                        ndis = SCost::new(get_enode_cost(n));
                        let pair0: (EClassId, SCost) = (n.ch[0], dis[n.ch[0] as usize].clone());
                        ndis.add_assign_pair(&pair0);
                        let pair1: (EClassId, SCost) = (n.ch[1], dis[n.ch[1] as usize].clone());
                        ndis.add_assign_pair(&pair1);
                        let then_cost: Cost = dis[n.ch[2] as usize].sum;
                        let else_cost: Cost = dis[n.ch[3] as usize].sum;
                        // Heuristics for computing cost of an If
                        ndis.sum += std::cmp::max(then_cost, else_cost) + (std::cmp::min(then_cost, else_cost) >> 2);
                    } else if n.get_op() == "DoWhile" {
                        crate::debug_assert_tiger!(n.ch.len() == 2);
                        ndis = dis[n.ch[0] as usize].clone();
                        let body_cost: Cost = dis[n.ch[1] as usize].sum;
                        // Heuristics for computing cost of a loop
                        // Can be improved further by plumbing the information from the iter analysis
                        // This can potentially overflow due to deeply nested loops
                        ndis.sum += body_cost * 500;
                    } else {
                        // Otherwise, just the bag cost
                        ndis = SCost::new(get_enode_cost(n));
                        for k in 0..n.ch.len() {
                            let pair: (EClassId, SCost) = (n.ch[k], dis[n.ch[k] as usize].clone());
                            ndis.add_assign_pair(&pair);
                        }
                    }
                    if ndis < dis[pc as usize] {
                        dis[pc as usize] = ndis;
                        pick[pc as usize] = pn;
                        maxheap.push((!dis[pc as usize].sum, pc));
                    }
                }
            }
        }
    }
    let mut dis_sum: Vec<Cost> = vec![0; g.neclasses()];
    for i in 0..(g.neclasses() as EClassId) {
        dis_sum[i as usize] = dis[i as usize].sum;
    }
    (pick, dis_sum)
}

pub fn greedy_extract_estimate_all_eclasses_cost(g: &EGraph) -> Vec<Cost> {
    greedy_extract_compute_eclasses_pick(g, -1).1
}

pub fn get_statewalk_enode_cost(g: &EGraph, eclass_cost: &Vec<Cost>, n: &ENode) -> Cost {
    let mut ret: Cost = 0;
    if n.get_op() == "If" {
        crate::debug_assert_tiger!(n.ch.len() == 4);
        ret = get_enode_cost(n);
        if !g.eclasses[n.ch[0] as usize].isEffectful {
            ret += eclass_cost[n.ch[0] as usize];
        }
        if !g.eclasses[n.ch[1] as usize].isEffectful {
            ret += eclass_cost[n.ch[1] as usize];
        }
        let then_cost: Cost = eclass_cost[n.ch[2] as usize];
        let else_cost: Cost = eclass_cost[n.ch[3] as usize];
        ret += std::cmp::max(then_cost, else_cost) + (std::cmp::min(then_cost, else_cost) >> 2);
    } else if n.get_op() == "DoWhile" {
        crate::debug_assert_tiger!(n.ch.len() == 2);
        let body_cost: Cost = eclass_cost[n.ch[1] as usize];
        ret = body_cost * 500;
    } else {
        ret = get_enode_cost(n);
        for k in 0..n.ch.len() {
            let cid: EClassId = n.ch[k];
            if !g.eclasses[cid as usize].isEffectful {
                ret += eclass_cost[cid as usize];
            }
        }
    }
    ret
}

pub fn compute_statewalk_cost(g: &EGraph) -> Vec<Vec<Cost>> {
    let eclass_cost: Vec<Cost> = greedy_extract_estimate_all_eclasses_cost(g);

    let mut statewalk_cost: Vec<Vec<Cost>> = vec![Vec::new(); g.neclasses()];
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        if c.isEffectful {
            statewalk_cost[i as usize].resize(c.nenodes(), 0);
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                statewalk_cost[i as usize][j as usize] = get_statewalk_enode_cost(g, &eclass_cost, n);
            }
        }
    }
    statewalk_cost
}

pub fn project_statewalk_cost(gr2g: &EGraphMapping, statewalk_cost: &Vec<Vec<Cost>>) -> Vec<Vec<Cost>> {
    let mut rstatewalk_cost: Vec<Vec<Cost>> = vec![Vec::new(); gr2g.eclassidmp.len()];
    for i in 0..(gr2g.eclassidmp.len() as EClassId) {
        let cid: EClassId = gr2g.eclassidmp[i as usize];
        if statewalk_cost[cid as usize].len() > 0 {
            rstatewalk_cost[i as usize].resize(gr2g.enodeidmp[i as usize].len(), 0);
            for j in 0..(rstatewalk_cost[i as usize].len() as ENodeId) {
                rstatewalk_cost[i as usize][j as usize] =
                    statewalk_cost[gr2g.eclassidmp[i as usize] as usize][gr2g.enodeidmp[i as usize][j as usize] as usize];
            }
        }
    }
    rstatewalk_cost
}

pub fn statewalk_greedy_extraction(g: &EGraph, root: EClassId) -> Extraction {
    let mut pick: Vec<ENodeId> = vec![-1; g.neclasses()];
    let mut dis: Vec<SCost> = vec![SCost::new(INF); g.neclasses()];
    let mut maxheap: BinaryHeap<(Cost, EClassId)> = BinaryHeap::new();
    // Optimization
    let parents: Vec<Vec<(EClassId, ENodeId)>> = compute_reverse_index(g);
    let mut cnt: Vec<Vec<i32>> = vec![Vec::new(); g.neclasses()];
    // Initialize nodes
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        cnt[i as usize].resize(c.nenodes(), 0);
        for j in 0..(c.nenodes() as ENodeId) {
            let n: &ENode = &c.enodes[j as usize];
            cnt[i as usize][j as usize] = n.ch.len() as i32;
            if cnt[i as usize][j as usize] == 0 {
                let ndis: SCost = SCost::new(get_enode_cost(n));
                if ndis < dis[i as usize] {
                    dis[i as usize] = ndis;
                    pick[i as usize] = j;
                    maxheap.push((!dis[i as usize].sum, i));
                }
            }
        }
    }
    let mut e: Extraction = Extraction::new();
    let mut extracted: Vec<ExtractionENodeId> = vec![UNEXTRACTABLE_ECLASS; g.neclasses()];
    let mut buf_id: Vec<i32> = vec![-1; g.neclasses()];
    let mut processed: Vec<bool> = vec![false; g.neclasses()];
    while maxheap.len() > 0 {
        let d: Cost = !maxheap.peek().unwrap().0;
        let i: EClassId = maxheap.peek().unwrap().1;
        maxheap.pop();
        if d != dis[i as usize].sum {
            continue;
        }
        if g.eclasses[i as usize].isEffectful {
            // grow the extraction
            let mut buf: Vec<EClassId> = Vec::new();
            let mut edges: Vec<Vec<i32>> = Vec::new();
            let mut bcnt: Vec<i32> = Vec::new();
            buf_id[i as usize] = 0;
            buf.push(i);
            edges.push(Vec::new());
            bcnt.push(0);
            // Note: the C++ inner loop shadows `i` with the loop index. Mirror that
            // by using the same name here — Rust's shadowing handles it cleanly.
            {
                let mut i: usize = 0;
                while i < buf.len() {
                    let u: EClassId = buf[i];
                    let c = &g.eclasses[u as usize];
                    let n: &ENode = &c.enodes[pick[u as usize] as usize];
                    for k in 0..n.ch.len() {
                        let v: EClassId = n.ch[k];
                        if extracted[v as usize] == UNEXTRACTABLE_ECLASS {
                            if buf_id[v as usize] == -1 {
                                buf_id[v as usize] = buf.len() as i32;
                                buf.push(v);
                                edges.push(Vec::new());
                                bcnt.push(0);
                            }
                            edges[buf_id[v as usize] as usize].push(i as i32);
                            bcnt[i] += 1;
                        }
                    }
                    i += 1;
                }
            }
            let mut q: Vec<i32> = Vec::new();
            for i2 in 0..buf.len() {
                if bcnt[i2] == 0 {
                    q.push(i2 as i32);
                }
            }
            {
                let mut i: usize = 0;
                while i < q.len() {
                    let u: i32 = q[i];
                    for j in 0..edges[u as usize].len() {
                        let v: i32 = edges[u as usize][j];
                        bcnt[v as usize] -= 1;
                        if bcnt[v as usize] == 0 {
                            q.push(v);
                        }
                    }
                    i += 1;
                }
            }
            crate::debug_assert_tiger!(q.len() == buf.len());
            let base: i32 = e.len() as i32;
            e.resize(e.len() + q.len(), ExtractionENode::default());
            for i2 in 0..q.len() {
                let u: i32 = buf[q[i2] as usize];
                extracted[u as usize] = base + i2 as i32;
                let en: &mut ExtractionENode = &mut e[extracted[u as usize] as usize];
                en.c = u;
                en.n = pick[u as usize];
            }
            for i2 in 0..q.len() {
                let u: i32 = buf[q[i2] as usize];
                let n_ch_len: usize = g.eclasses[u as usize].enodes[pick[u as usize] as usize].ch.len();
                let n_ch: Vec<EClassId> = g.eclasses[u as usize].enodes[pick[u as usize] as usize].ch.clone();
                let en: &mut ExtractionENode = &mut e[extracted[u as usize] as usize];
                en.ch.resize(n_ch_len, 0);
                for j in 0..n_ch_len {
                    en.ch[j] = extracted[n_ch[j] as usize];
                }
            }
            // processed optimization
            for j in 0..q.len() {
                let u: i32 = buf[q[j] as usize];
                if dis[u as usize].sum > 0 {
                    dis[u as usize].sum = 0;
                    dis[u as usize].bag.clear();
                    if u != i {
                        maxheap.push((!0u64, u));
                    }
                }
            }
        }
        if i == root {
            break;
        }
        for j in 0..parents[i as usize].len() {
            let pc: EClassId = parents[i as usize][j].0;
            let pn: ENodeId = parents[i as usize][j].1;
            let trigger: bool = if processed[i as usize] {
                cnt[pc as usize][pn as usize] == 0
            } else {
                cnt[pc as usize][pn as usize] -= 1;
                cnt[pc as usize][pn as usize] == 0
            };
            if trigger {
                let n: &ENode = &g.eclasses[pc as usize].enodes[pn as usize];
                let mut ndis: SCost = SCost::new(0);
                if !g.eclasses[pc as usize].isEffectful {
                    ndis = SCost::new(get_enode_cost(n));
                }
                for k in 0..n.ch.len() {
                    let pair: (EClassId, SCost) = (n.ch[k], dis[n.ch[k] as usize].clone());
                    ndis.add_assign_pair(&pair);
                }
                if ndis < dis[pc as usize] {
                    dis[pc as usize] = ndis;
                    pick[pc as usize] = pn;
                    maxheap.push((!dis[pc as usize].sum, pc));
                }
            }
        }
        processed[i as usize] = true;
    }
    crate::debug_assert_tiger!(extracted[root as usize] != UNEXTRACTABLE_ECLASS);
    crate::debug_assert_tiger!(crate::debug::is_effect_safe_extraction(g, root, &e));
    e
}

// Suppress unused-import warning for `Reverse` (kept for documentation: the C++
// max-heap on `~sum` is equivalent to a min-heap on `sum`, but we mirror the
// C++ formulation directly with a plain `BinaryHeap` keyed on `!sum`).
#[allow(dead_code)]
fn _unused_reverse(_: Reverse<u64>) {}
