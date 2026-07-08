// Port of regionalize.h / regionalize.cpp — partition into regions and recurse.
// Direct line-by-line translation.

use std::cell::RefCell;

use crate::egraphin::{
    inverse_egraph_mapping, project_extraction, prune_unextractable_enodes, EClassId, EGraph,
    EGraphMapping, ENode, ENodeId, EClass, Extraction, ExtractionENode, ExtractionENodeId,
    UNEXTRACTABLE_ECLASS,
};
use crate::greedy::{compute_statewalk_cost, project_statewalk_cost, Cost};
use crate::tiger::extract_regionalized_egraph_tiger;

type RegionId = i32;

// static int timestamp = 0;
// vector<int> region_vis;
// vector<EClassId> region_eclass_inv;
thread_local! {
    static TIMESTAMP: RefCell<i32> = RefCell::new(0);
    static REGION_VIS: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    static REGION_ECLASS_INV: RefCell<Vec<EClassId>> = RefCell::new(Vec::new());
}

pub fn construct_regionalized_egraph(g: &EGraph, root: EClassId) -> (EGraph, (EClassId, EGraphMapping)) {
    REGION_VIS.with(|c| {
        let mut v = c.borrow_mut();
        v.resize(g.neclasses(), 0);
    });
    REGION_ECLASS_INV.with(|c| {
        let mut v = c.borrow_mut();
        v.resize(g.neclasses(), 0);
    });
    TIMESTAMP.with(|t| {
        *t.borrow_mut() += 1;
    });
    let ts: i32 = TIMESTAMP.with(|t| *t.borrow());
    let mut gr2g: Vec<EClassId> = Vec::new();
    REGION_VIS.with(|c| {
        c.borrow_mut()[root as usize] = ts;
    });
    REGION_ECLASS_INV.with(|c| {
        c.borrow_mut()[root as usize] = gr2g.len() as EClassId;
    });
    gr2g.push(root);
    {
        let mut _idx: usize = 0;
        while _idx < gr2g.len() {
            let i: EClassId = gr2g[_idx];
            let c: &EClass = &g.eclasses[i as usize];
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                let mut isSubregionChild: bool = false;
                for k in 0..n.ch.len() {
                    let v: EClassId = n.ch[k];
                    if g.eclasses[v as usize].isEffectful && !isSubregionChild {
                        let vis_v: i32 = REGION_VIS.with(|c2| c2.borrow()[v as usize]);
                        if vis_v != ts {
                            REGION_VIS.with(|c2| {
                                c2.borrow_mut()[v as usize] = ts;
                            });
                            REGION_ECLASS_INV.with(|c2| {
                                c2.borrow_mut()[v as usize] = gr2g.len() as EClassId;
                            });
                            gr2g.push(v);
                        }
                        isSubregionChild = true;
                    }
                }
            }
            _idx += 1;
        }
    }
    {
        let mut _idx: usize = 0;
        while _idx < gr2g.len() {
            let i: EClassId = gr2g[_idx];
            let c: &EClass = &g.eclasses[i as usize];
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                for k in 0..n.ch.len() {
                    let v: EClassId = n.ch[k];
                    if !g.eclasses[v as usize].isEffectful {
                        let vis_v: i32 = REGION_VIS.with(|c2| c2.borrow()[v as usize]);
                        if vis_v != ts {
                            REGION_VIS.with(|c2| {
                                c2.borrow_mut()[v as usize] = ts;
                            });
                            REGION_ECLASS_INV.with(|c2| {
                                c2.borrow_mut()[v as usize] = gr2g.len() as EClassId;
                            });
                            gr2g.push(v);
                        }
                    }
                }
            }
            _idx += 1;
        }
    }
    let mut gr: EGraph = EGraph::default();
    for i in 0..(gr2g.len() as EClassId) {
        let u: EClassId = gr2g[i as usize];
        let c: &EClass = &g.eclasses[u as usize];
        let mut nc: EClass = EClass::default();
        nc.isEffectful = c.isEffectful;
        nc.enodes.resize(c.nenodes(), ENode::default());
        for j in 0..(c.nenodes() as ENodeId) {
            let n: &ENode = &c.enodes[j as usize];
            let nn: &mut ENode = &mut nc.enodes[j as usize];
            nn.head = n.head.clone();
            nn.eclass = i;
            let mut isSubregionchild: bool = false;
            for k in 0..n.ch.len() {
                let v: EClassId = n.ch[k];
                if g.eclasses[v as usize].isEffectful {
                    if isSubregionchild {
                        continue;
                    }
                    isSubregionchild = true;
                }
                let vis_v: i32 = REGION_VIS.with(|c2| c2.borrow()[v as usize]);
                if vis_v != ts {
                    nn.ch.push(UNEXTRACTABLE_ECLASS);
                } else {
                    let inv_v: EClassId = REGION_ECLASS_INV.with(|c2| c2.borrow()[v as usize]);
                    nn.ch.push(inv_v);
                }
            }
        }
        gr.eclasses.push(nc);
    }
    crate::debug_assert_tiger!(crate::debug::is_wellformed_egraph(&gr, true, false));
    // region_eclass_inv[root] = 0
    let res: (EGraph, EGraphMapping) = prune_unextractable_enodes(&gr, 0);
    let grp: EGraph = res.0;
    let gr2grp: EGraphMapping = res.1;
    let nroot: EClassId = gr2grp.eclassidmp[0];
    let mut grp2gr: EGraphMapping = inverse_egraph_mapping(&grp, &gr2grp);
    for i in 0..(grp.neclasses() as EClassId) {
        //composes to grp2g
        grp2gr.eclassidmp[i as usize] = gr2g[grp2gr.eclassidmp[i as usize] as usize];
    }
    crate::debug_assert_tiger!(crate::debug::is_valid_egraph_mapping(&grp2gr, &grp, g, false, true, false, false));
    (grp, (nroot, grp2gr))
}

pub fn extract_region_tiger(
    g: &EGraph,
    root: EClassId,
    e: &mut Extraction,
    region_root_id: &Vec<RegionId>,
    region_extraction_cache: &mut Vec<(Extraction, ExtractionENodeId)>,
    statewalk_cost: &Vec<Vec<Cost>>,
) -> ExtractionENodeId {
    let rid: RegionId = region_root_id[root as usize];
    if region_extraction_cache[rid as usize].1 != -1 {
        return region_extraction_cache[rid as usize].1;
    }
    if region_extraction_cache[rid as usize].0.len() == 0 {
        // if has not been computed yet
        let res: (EGraph, (EClassId, EGraphMapping)) = construct_regionalized_egraph(g, root);
        let gr: &EGraph = &res.0;
        let nroot: EClassId = res.1.0;
        let gr2g: &EGraphMapping = &res.1.1;
        let tmpe: Extraction = extract_regionalized_egraph_tiger(gr, nroot, &project_statewalk_cost(gr2g, statewalk_cost), true, true);
        region_extraction_cache[rid as usize].0 = project_extraction(gr2g, &tmpe);
    }

    let mut subregions: Vec<ExtractionENodeId> = Vec::new();
    {
        let cache_len: ExtractionENodeId = region_extraction_cache[rid as usize].0.len() as ExtractionENodeId;
        let mut i: ExtractionENodeId = 0;
        while i < cache_len {
            // Pull out the fields we need so we don't hold a borrow during the recursive call.
            let en_c: EClassId = region_extraction_cache[rid as usize].0[i as usize].c;
            let en_n: ENodeId = region_extraction_cache[rid as usize].0[i as usize].n;
            let n_ch: Vec<EClassId> = g.eclasses[en_c as usize].enodes[en_n as usize].ch.clone();
            let mut isSubregionChild: bool = false;
            for j in 0..n_ch.len() {
                let v: EClassId = n_ch[j];
                if g.eclasses[v as usize].isEffectful {
                    if isSubregionChild {
                        let sub: ExtractionENodeId = extract_region_tiger(g, v, e, region_root_id, region_extraction_cache, statewalk_cost);
                        subregions.push(sub);
                    } else {
                        isSubregionChild = true;
                    }
                }
            }
            i += 1;
        }
    }
    let base: ExtractionENodeId = e.len() as ExtractionENodeId;
    let new_len: usize = base as usize + region_extraction_cache[rid as usize].0.len();
    e.resize(new_len, ExtractionENode::default());
    {
        let cache_len: ExtractionENodeId = region_extraction_cache[rid as usize].0.len() as ExtractionENodeId;
        let mut i: ExtractionENodeId = 0;
        let mut l: usize = 0;
        while i < cache_len {
            // C++: ExtractionENode &nen = e[base + i], &en = region_extraction_cache[rid].first[i];
            let en_n: ENodeId;
            let en_c: EClassId;
            let en_ch: Vec<ExtractionENodeId>;
            {
                let en: &ExtractionENode = &region_extraction_cache[rid as usize].0[i as usize];
                en_n = en.n;
                en_c = en.c;
                en_ch = en.ch.clone();
            }
            {
                let nen: &mut ExtractionENode = &mut e[(base + i) as usize];
                nen.n = en_n;
                nen.c = en_c;
            }
            let n_ch: Vec<EClassId> = g.eclasses[en_c as usize].enodes[en_n as usize].ch.clone();
            {
                let nen: &mut ExtractionENode = &mut e[(base + i) as usize];
                nen.ch.resize(n_ch.len(), 0);
            }
            let mut isSubregionChild: bool = false;
            let mut k: usize = 0;
            for j in 0..n_ch.len() {
                let v: EClassId = n_ch[j];
                if g.eclasses[v as usize].isEffectful {
                    if isSubregionChild {
                        let val: ExtractionENodeId = subregions[l];
                        l += 1;
                        let nen: &mut ExtractionENode = &mut e[(base + i) as usize];
                        nen.ch[j] = val;
                    } else {
                        let val: ExtractionENodeId = base + en_ch[k];
                        k += 1;
                        let nen: &mut ExtractionENode = &mut e[(base + i) as usize];
                        nen.ch[j] = val;
                        isSubregionChild = true;
                    }
                } else {
                    let val: ExtractionENodeId = base + en_ch[k];
                    k += 1;
                    let nen: &mut ExtractionENode = &mut e[(base + i) as usize];
                    nen.ch[j] = val;
                }
            }
            i += 1;
        }
    }
    region_extraction_cache[rid as usize].1 = (e.len() as ExtractionENodeId) - 1;
    region_extraction_cache[rid as usize].1
}

pub fn find_all_region_roots(g: &EGraph, fun_roots: &Vec<EClassId>) -> Vec<EClassId> {
    REGION_VIS.with(|c| {
        let mut v = c.borrow_mut();
        v.resize(g.neclasses(), 0);
    });
    TIMESTAMP.with(|t| {
        *t.borrow_mut() += 1;
    });
    let ts: i32 = TIMESTAMP.with(|t| *t.borrow());
    let mut ret: Vec<EClassId> = Vec::new();
    for i in 0..fun_roots.len() {
        let v: EClassId = fun_roots[i];
        let vis_v: i32 = REGION_VIS.with(|c| c.borrow()[v as usize]);
        if vis_v != ts {
            REGION_VIS.with(|c| {
                c.borrow_mut()[v as usize] = ts;
            });
            ret.push(v);
        }
    }
    for i in 0..(g.neclasses() as EClassId) {
        let c: &EClass = &g.eclasses[i as usize];
        if c.isEffectful {
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                let mut isSubregionroot: bool = false;
                for k in 0..n.ch.len() {
                    let v: EClassId = n.ch[k];
                    if g.eclasses[v as usize].isEffectful {
                        if !isSubregionroot {
                            isSubregionroot = true;
                        } else {
                            let vis_v: i32 = REGION_VIS.with(|c2| c2.borrow()[v as usize]);
                            if vis_v != ts {
                                REGION_VIS.with(|c2| {
                                    c2.borrow_mut()[v as usize] = ts;
                                });
                                ret.push(v);
                            }
                        }
                    }
                }
            }
        }
    }
    ret
}

pub fn extract_all_fun_roots_tiger(g: &EGraph, fun_roots: &Vec<EClassId>) -> Vec<Extraction> {
    let region_roots: Vec<EClassId> = find_all_region_roots(g, fun_roots);

    let mut region_root_id: Vec<RegionId> = vec![-1; g.neclasses()];
    for i in 0..region_roots.len() {
        region_root_id[region_roots[i] as usize] = i as RegionId;
    }

    let statewalk_cost: Vec<Vec<Cost>> = compute_statewalk_cost(g);

    let mut region_extraction_cache: Vec<(Extraction, ExtractionENodeId)> =
        vec![(Extraction::new(), 0); region_roots.len()];
    let mut ret: Vec<Extraction> = vec![Extraction::new(); fun_roots.len()];
    for i in 0..fun_roots.len() {
        let fun_root: EClassId = fun_roots[i];
        for j in 0..(region_roots.len() as RegionId) {
            region_extraction_cache[j as usize].1 = -1;
        }
        extract_region_tiger(g, fun_root, &mut ret[i], &region_root_id, &mut region_extraction_cache, &statewalk_cost);
        crate::debug_assert_tiger!(crate::debug::is_effect_safe_extraction(g, fun_root, &ret[i]));
    }
    ret
}
