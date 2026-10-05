// Port of egraphin.h / egraphin.cpp — minimal egraph types and helpers.
// Direct line-by-line translation.

use std::collections::VecDeque;

use egglog::ast::Literal;

pub type EClassId = i32;
pub type ENodeId = i32;
pub type ExtractionENodeId = i32;

// -1 used to denote an unextractable eclass
pub const UNEXTRACTABLE_ECLASS: EClassId = -1;

#[derive(Clone, Default)]
pub struct ENode {
    /// Constructor name, or the literal's text for primitive leaves.
    pub op: String,
    /// `Some` for primitive leaves (ints, bools, floats, strings).
    pub lit: Option<Literal>,
    pub eclass: EClassId,
    pub ch: Vec<EClassId>,
}

impl ENode {
    pub fn is_primitive(&self) -> bool {
        self.lit.is_some()
    }
}

#[derive(Clone, Default)]
pub struct EClass {
    pub enodes: Vec<ENode>,
    pub isEffectful: bool,
}

impl EClass {
    pub fn nenodes(&self) -> usize {
        self.enodes.len()
    }
}

#[derive(Clone, Default)]
pub struct EGraph {
    pub eclasses: Vec<EClass>,
}

impl EGraph {
    pub fn neclasses(&self) -> usize {
        self.eclasses.len()
    }
}

// An extraction corespondes to a particular egraph

#[derive(Clone, Default)]
pub struct ExtractionENode {
    pub c: EClassId,
    pub n: ENodeId,
    pub ch: Vec<ExtractionENodeId>,
}

pub type Extraction = Vec<ExtractionENode>;

// A mapping from egraph A to B

#[derive(Clone, Default)]
pub struct EGraphMapping {
    pub eclassidmp: Vec<EClassId>,
    pub enodeidmp: Vec<Vec<ENodeId>>,
}

impl EGraphMapping {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_egraph(g: &EGraph) -> Self {
        let mut m = EGraphMapping {
            eclassidmp: vec![UNEXTRACTABLE_ECLASS; g.neclasses()],
            enodeidmp: vec![Vec::new(); g.neclasses()],
        };
        for i in 0..(g.neclasses() as EClassId) {
            let c = &g.eclasses[i as usize];
            m.enodeidmp[i as usize] = vec![UNEXTRACTABLE_ECLASS; c.nenodes()];
        }
        m
    }
}

// A reverse index for speed up things

pub type EClassParents = Vec<Vec<(EClassId, ENodeId)>>;

// An accompanying counter for optimizations

pub type ENodeCounters = Vec<Vec<i32>>;

pub fn compute_reverse_index(g: &EGraph) -> EClassParents {
    let mut ret: EClassParents = vec![Vec::new(); g.neclasses()];
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        for j in 0..(c.nenodes() as ENodeId) {
            let n = &c.enodes[j as usize];
            for k in 0..n.ch.len() {
                if n.ch[k] != UNEXTRACTABLE_ECLASS {
                    ret[n.ch[k] as usize].push((i, j));
                }
            }
        }
    }
    ret
}

pub fn initialize_enode_counters(g: &EGraph) -> ENodeCounters {
    let mut ret: ENodeCounters = vec![Vec::new(); g.neclasses()];
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        ret[i as usize].resize(c.nenodes(), 0);
        for j in 0..(c.nenodes() as ENodeId) {
            ret[i as usize][j as usize] = c.enodes[j as usize].ch.len() as i32;
        }
    }
    ret
}

pub fn inverse_egraph_mapping(gp: &EGraph, g2gp: &EGraphMapping) -> EGraphMapping {
    let mut gp2g = EGraphMapping::from_egraph(gp);
    for i in 0..(g2gp.eclassidmp.len() as EClassId) {
        if g2gp.eclassidmp[i as usize] != UNEXTRACTABLE_ECLASS {
            crate::debug_assert_tiger!(
                0 <= g2gp.eclassidmp[i as usize]
                    && g2gp.eclassidmp[i as usize] < gp.neclasses() as EClassId
            );
            gp2g.eclassidmp[g2gp.eclassidmp[i as usize] as usize] = i;
        }
    }
    for i in 0..(g2gp.eclassidmp.len() as EClassId) {
        for j in 0..(g2gp.enodeidmp[i as usize].len() as ENodeId) {
            if g2gp.enodeidmp[i as usize][j as usize] != UNEXTRACTABLE_ECLASS {
                crate::debug_assert_tiger!(
                    0 <= g2gp.enodeidmp[i as usize][j as usize]
                        && g2gp.enodeidmp[i as usize][j as usize]
                            < gp.eclasses[g2gp.eclassidmp[i as usize] as usize].nenodes()
                                as ENodeId
                );
                gp2g.enodeidmp[g2gp.eclassidmp[i as usize] as usize]
                    [g2gp.enodeidmp[i as usize][j as usize] as usize] = j;
            }
        }
    }
    gp2g
}

pub fn project_extraction(f: &EGraphMapping, e: &Extraction) -> Extraction {
    let mut ne: Extraction = e.clone();
    for i in 0..(e.len() as ExtractionENodeId) {
        ne[i as usize].n = f.enodeidmp[ne[i as usize].c as usize][ne[i as usize].n as usize];
        ne[i as usize].c = f.eclassidmp[ne[i as usize].c as usize];
    }
    ne
}

// return a mapping from the old egraph ids to the new one
pub fn prune_unextractable_enodes(g: &EGraph, root: EClassId) -> (EGraph, EGraphMapping) {
    let mut extractable: Vec<bool> = vec![false; g.neclasses()];
    let parents: EClassParents = compute_reverse_index(g);
    let mut cnts: ENodeCounters = initialize_enode_counters(g);
    let mut q: VecDeque<EClassId> = VecDeque::new();
    for i in 0..(g.neclasses() as EClassId) {
        let c = &g.eclasses[i as usize];
        for j in 0..(c.nenodes() as ENodeId) {
            if cnts[i as usize][j as usize] == 0 {
                if !extractable[i as usize] {
                    extractable[i as usize] = true;
                    q.push_back(i);
                }
            }
        }
    }
    while q.len() > 0 {
        let u = *q.front().unwrap();
        q.pop_front();
        for i in 0..parents[u as usize].len() {
            let vc: EClassId = parents[u as usize][i].0;
            let vn: ENodeId = parents[u as usize][i].1;
            cnts[vc as usize][vn as usize] -= 1;
            if cnts[vc as usize][vn as usize] == 0 {
                if !extractable[vc as usize] {
                    extractable[vc as usize] = true;
                    q.push_back(vc);
                }
            }
        }
    }
    let mut reachable: Vec<bool> = vec![if root == -1 { true } else { false }; g.neclasses()];
    if root != -1 {
        reachable[root as usize] = true;
        q.push_back(root);
        while q.len() > 0 {
            let u = *q.front().unwrap();
            q.pop_front();
            let c = &g.eclasses[u as usize];
            for j in 0..(c.nenodes() as ENodeId) {
                let n = &c.enodes[j as usize];
                let mut isExtractable = true;
                for k in 0..n.ch.len() {
                    let v: EClassId = n.ch[k];
                    if v == UNEXTRACTABLE_ECLASS || !extractable[v as usize] {
                        isExtractable = false;
                        break;
                    }
                }
                if isExtractable {
                    for k in 0..n.ch.len() {
                        let v: EClassId = n.ch[k];
                        if !reachable[v as usize] {
                            reachable[v as usize] = true;
                            q.push_back(v);
                        }
                    }
                }
            }
        }
    }
    let mut gp: EGraph = EGraph::default();
    let mut mp: EGraphMapping = EGraphMapping::from_egraph(g);
    for i in 0..(g.neclasses() as EClassId) {
        if reachable[i as usize] && extractable[i as usize] {
            let c = &g.eclasses[i as usize];
            let mut nc = EClass::default();
            nc.isEffectful = c.isEffectful;
            mp.eclassidmp[i as usize] = gp.neclasses() as EClassId;
            gp.eclasses.push(nc);
        }
    }
    for i in 0..(g.neclasses() as EClassId) {
        if mp.eclassidmp[i as usize] != UNEXTRACTABLE_ECLASS {
            let c = &g.eclasses[i as usize];
            // Take immutable info we need before borrowing gp mutably below.
            let target_idx = mp.eclassidmp[i as usize] as usize;
            for j in 0..(c.nenodes() as ENodeId) {
                let n = &c.enodes[j as usize];
                let mut isExtractable = true;
                for k in 0..n.ch.len() {
                    let v: EClassId = n.ch[k];
                    if v == UNEXTRACTABLE_ECLASS
                        || mp.eclassidmp[v as usize] == UNEXTRACTABLE_ECLASS
                    {
                        isExtractable = false;
                        break;
                    }
                }
                if isExtractable {
                    let n = &c.enodes[j as usize];
                    let mut nn = ENode::default();
                    nn.op = n.op.clone();
                    nn.lit = n.lit.clone();
                    nn.ch.resize(n.ch.len(), 0);
                    for k in 0..n.ch.len() {
                        nn.ch[k] = mp.eclassidmp[n.ch[k] as usize];
                    }
                    nn.eclass = mp.eclassidmp[i as usize];
                    let nc: &mut EClass = &mut gp.eclasses[target_idx];
                    mp.enodeidmp[i as usize][j as usize] = nc.nenodes() as ENodeId;
                    nc.enodes.push(nn);
                }
            }
        }
    }
    crate::debug_assert_tiger!(crate::debug::is_wellformed_egraph(&gp, false, true));
    crate::debug_assert_tiger!(crate::debug::is_valid_egraph_mapping(
        &mp, g, &gp, true, true, true, true
    ));
    (gp, mp)
}
