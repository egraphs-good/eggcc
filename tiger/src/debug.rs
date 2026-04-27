// Port of debug.h / debug.cpp.
// The DEBUG_ASSERT and DEBUG_CERR macros are gated on the `debug` cargo feature
// (mirrors C++ `-DDEBUG`). Invariant checkers (is_wellformed_egraph etc.) are
// translated by the wave-1 debug agent below.

#[macro_export]
macro_rules! debug_assert_tiger {
    ($cond:expr) => {
        #[cfg(feature = "debug")]
        {
            assert!($cond);
        }
    };
}

#[macro_export]
macro_rules! debug_cerr {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug")]
        {
            eprintln!($($arg)*);
        }
    };
}

// TODO(wave-1 debug agent): port debug.cpp invariant checkers
// (is_wellformed_egraph, is_valid_egraph_mapping, arg_check_regionalized_egraph,
// is_valid_statewalk, is_valid_extraction, is_effect_safe_extraction,
// debug_print_egraph, debug_print_extraction).

use std::collections::VecDeque;

use crate::egraphin::{
    EClass, EClassId, EGraph, EGraphMapping, ENode, ENodeId, Extraction, ExtractionENode,
    ExtractionENodeId, UNEXTRACTABLE_ECLASS,
};

pub type Statewalk = Vec<(EClassId, ENodeId)>;

pub fn debug_print_egraph(g: &EGraph) {
    eprintln!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>begin:debug_print_egraph");
    let mut cnt: usize = 0;
    for i in 0..(g.neclasses() as EClassId) {
        cnt += g.eclasses[i as usize].nenodes();
    }
    eprintln!("# eclasses: {}\n# enodes: {}", g.neclasses(), cnt);
    for i in 0..(g.neclasses() as EClassId) {
        eprintln!("# eclass {}", i);
        let c: &EClass = &g.eclasses[i as usize];
        let f: i32 = if c.isEffectful { 1 } else { 0 };
        let m: i32 = c.enodes.len() as i32;
        eprintln!("{} {}", f, m);
        for j in 0..m {
            let n: &ENode = &c.enodes[j as usize];
            let l: usize = n.ch.len();
            eprint!("{}\n{}{}", n.head, l, if l == 0 { '\n' } else { ' ' });
            for k in 0..l {
                eprint!(
                    "{}{}{}",
                    if g.eclasses[n.ch[k] as usize].isEffectful { "!" } else { " " },
                    n.ch[k],
                    if k == l - 1 { '\n' } else { ' ' }
                );
            }
            //printf("%d\n", n.cost);
        }
        eprintln!();
    }
    eprintln!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<end:debug_print_egraph");
}

pub fn debug_print_extraction(g: &EGraph, e: &Extraction) {
    for i in 0..(e.len() as ExtractionENodeId) {
        eprint!(
            "#{} {}{} {} {}{}",
            i,
            e[i as usize].c,
            if g.eclasses[e[i as usize].c as usize].isEffectful { '!' } else { ' ' },
            e[i as usize].n,
            g.eclasses[e[i as usize].c as usize].enodes[e[i as usize].n as usize].head,
            if e[i as usize].ch.len() == 0 { '\n' } else { ' ' }
        );
        for j in 0..e[i as usize].ch.len() {
            eprint!(
                "#{}{}",
                e[i as usize].ch[j],
                if j == e[i as usize].ch.len() - 1 { '\n' } else { ' ' }
            );
        }
    }
}

pub fn is_wellformed_egraph(g: &EGraph, allow_unextractable_child: bool, allow_subregion_child: bool) -> bool {
    let mut ret: bool = true;
    for i in 0..(g.neclasses() as EClassId) {
        let c: &EClass = &g.eclasses[i as usize];
        if !(allow_unextractable_child || c.nenodes() > 0) {
            ret = false;
            eprintln!("Error: Found empty eclass");
        }
        for j in 0..(c.nenodes() as ENodeId) {
            let n: &ENode = &c.enodes[j as usize];
            if !(n.eclass == i) {
                ret = false;
                eprintln!("Error: Wrong eclass for an enode");
            }
            let mut effectful_ch_cnt: i32 = 0;
            for k in 0..n.ch.len() {
                let chc: EClassId = n.ch[k];
                if !((allow_unextractable_child && chc == UNEXTRACTABLE_ECLASS) || (0 <= chc && chc < g.neclasses() as EClassId)) {
                    ret = false;
                    eprintln!("Error: Invalid child edge {},{},{}", i, j, k);
                }
                // assuming unextractable eclasses are effectful
                if chc == UNEXTRACTABLE_ECLASS || g.eclasses[chc as usize].isEffectful {
                    effectful_ch_cnt += 1;
                }
            }
            if !allow_subregion_child && effectful_ch_cnt > 1 {
                ret = false;
                eprintln!("Error: Found subregion child at enode {},{}", i, j);
            }
        }
    }
    if !ret {
        debug_print_egraph(g);
    }
    ret
}

pub fn debug_print_egraph_mapping(g2gp: &EGraphMapping, g: &EGraph, gp: &EGraph) {
    eprintln!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>begin:debug_print_egraph_mapping");
    debug_print_egraph(g);
    debug_print_egraph(gp);
    eprintln!("{}", g2gp.eclassidmp.len());
    for i in 0..(g2gp.eclassidmp.len() as EClassId) {
        eprintln!("\t{} -> {}", i, g2gp.eclassidmp[i as usize]);
    }
    eprintln!("---");
    eprintln!("{}", g2gp.enodeidmp.len());
    for i in 0..(g2gp.enodeidmp.len() as EClassId) {
        eprintln!("\t{}: {}", i, g2gp.enodeidmp[i as usize].len());
        for j in 0..(g2gp.enodeidmp[i as usize].len() as ENodeId) {
            eprintln!("\t\t{} -> {}", j, g2gp.enodeidmp[i as usize][j as usize]);
        }
    }
    eprintln!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<end:debug_print_egraph_mapping");
}

pub fn is_valid_egraph_mapping_helper(g2gp: &EGraphMapping, g: &EGraph, gp: &EGraph, isPartial: bool, isInjective: bool, isSurjective: bool, checkChildrenConsistentcy: bool) -> bool {
    if g2gp.eclassidmp.len() != g.neclasses() || g2gp.enodeidmp.len() != g.neclasses() {
        eprintln!("Error: Wrong domain #eclasses");
        return false;
    }
    let mut vis_gp: Vec<Vec<bool>> = vec![Vec::new(); gp.neclasses()];
    for i in 0..(gp.neclasses() as EClassId) {
        vis_gp[i as usize].resize(gp.eclasses[i as usize].nenodes(), false);
    }
    for i in 0..(g.neclasses() as EClassId) {
        let c: &EClass = &g.eclasses[i as usize];
        if g2gp.enodeidmp[i as usize].len() != c.nenodes() {
            eprintln!("Error: Wrong domain #enodes in eclass #{}", i);
            return false;
        }
        let cpid: EClassId = g2gp.eclassidmp[i as usize];
        if isPartial && cpid == UNEXTRACTABLE_ECLASS {
            continue;
        }
        if !(0 <= cpid && cpid < gp.neclasses() as EClassId) {
            eprintln!("Error: Invalid codomain eclass for eclass #{} mapped to #{}", i, cpid);
            return false;
        }
        let cp: &EClass = &gp.eclasses[cpid as usize];
        if c.isEffectful != cp.isEffectful {
            eprintln!("Error: Mismatching effectful flags for eclass #{} mapped to #{}", i, cpid);
            return false;
        }
        for j in 0..(c.nenodes() as ENodeId) {
            let n: &ENode = &c.enodes[j as usize];
            let npid: ENodeId = g2gp.enodeidmp[i as usize][j as usize];
            if isPartial && npid == UNEXTRACTABLE_ECLASS {
                continue;
            }
            if !(0 <= npid && npid < cp.nenodes() as ENodeId) {
                eprintln!("Error: Invalid codomain enode for enode #{},{} mapped to #{},{}", i, j, cpid, npid);
                return false;
            }
            if isInjective && vis_gp[cpid as usize][npid as usize] {
                eprintln!("Error: egraph mapping not injective / multiple enodes map into the same enode");
                return false;
            }
            vis_gp[cpid as usize][npid as usize] = true;
            if checkChildrenConsistentcy {
                let np: &ENode = &cp.enodes[npid as usize];
                if n.ch.len() != np.ch.len() {
                    eprintln!("Error: Children inconsistency: different number of children of enode #{},{} mapped to #{},{}", i, j, cpid, npid);
                    return false;
                }
                for k in 0..n.ch.len() {
                    let chc: EClassId = n.ch[k];
                    let chcp: EClassId = np.ch[k];
                    if g2gp.eclassidmp[chc as usize] != chcp {
                        eprintln!("Error: Children inconsistency: bad child eclass of enode #{},{},{} mapped to #{},{},{}", i, j, chc, cpid, npid, chcp);
                        return false;
                    }
                }
            }
        }
    }
    if isSurjective {
        for i in 0..(gp.neclasses() as EClassId) {
            for j in 0..(gp.eclasses[i as usize].nenodes() as ENodeId) {
                if !vis_gp[i as usize][j as usize] {
                    eprintln!("Error: egraph mapping not surjective / enode in codomain not mapped {},{}", i, j);
                    return false;
                }
            }
        }
    }
    true
}

pub fn is_valid_egraph_mapping(g2gp: &EGraphMapping, g: &EGraph, gp: &EGraph, isPartial: bool, isInjective: bool, isSurjective: bool, checkChildrenConsistentcy: bool) -> bool {
    let ret: bool = is_valid_egraph_mapping_helper(g2gp, g, gp, isPartial, isInjective, isSurjective, checkChildrenConsistentcy);
    if !ret {
        debug_print_egraph_mapping(g2gp, g, gp);
    }
    ret
}

pub fn arg_check_regionalized_egraph(g: &EGraph) -> bool {
    let mut ret: bool = true;
    let mut cntn: i32 = 0;
    let mut cntc: i32 = 0;
    for i in 0..(g.neclasses() as EClassId) {
        let c: &EClass = &g.eclasses[i as usize];
        if c.isEffectful {
            let mut found_arg: bool = false;
            for j in 0..(c.nenodes() as ENodeId) {
                let n: &ENode = &c.enodes[j as usize];
                if n.ch.len() == 0 {
                    found_arg = true;
                    cntn += 1;
                }
            }
            if found_arg {
                cntc += 1;
            }
        }
    }
    if cntn == 0 {
        ret = false;
        eprintln!("Error: Found no arg in a regionalized egraph");
    }
    if cntn > 1 {
        ret = false;
        eprintln!("Error: Found multiple arg enodes in a regionalized egraph #{}", cntn);
        if cntc > 1 {
            eprintln!("Error: Found multiple arg eclasses in a regionalized egraph #{}", cntc);
        }
    }
    if !ret {
        debug_print_egraph(g);
    }
    ret
}

pub fn is_valid_statewalk(g: &EGraph, root: EClassId, sw: &Statewalk) -> bool {
    let mut ret: bool = true;
    if sw.len() == 0 || sw[0].0 != root {
        ret = false;
        eprintln!("Error: Statewalk does not start with the root eclass");
    }
    for i in 0..sw.len() {
        let cid: EClassId = sw[i].0;
        let nid: ENodeId = sw[i].1;
        if !(0 <= cid && cid < g.neclasses() as EClassId) {
            ret = false;
            eprintln!("Error: Invalid eclassid");
            break;
        }
        let c: &EClass = &g.eclasses[cid as usize];
        if !(0 <= nid && nid < c.nenodes() as ENodeId) {
            ret = false;
            eprintln!("Error: Invalid enodeid");
            break;
        }
        let n: &ENode = &g.eclasses[cid as usize].enodes[nid as usize];
        if i + 1 < sw.len() {
            let mut efchcid: EClassId = UNEXTRACTABLE_ECLASS;
            for j in 0..n.ch.len() {
                let chc: EClassId = n.ch[j];
                if g.eclasses[chc as usize].isEffectful {
                    efchcid = chc;
                }
            }
            if efchcid == UNEXTRACTABLE_ECLASS {
                ret = false;
                eprintln!("Error: Invalid prefix with no connection");
                break;
            }
            if efchcid != sw[i + 1].0 {
                ret = false;
                eprintln!("Error: Mismatched child eclass");
                break;
            }
        } else {
            if n.ch.len() != 0 {
                ret = false;
                eprintln!("Error: Statewalk does not end with an arg");
            }
        }
    }
    if !ret {
        debug_print_egraph(g);
        for i in 0..sw.len() {
            eprint!("{} {}", sw[i].0, sw[i].1);
        }
    }
    ret
}

pub fn is_valid_extraction_strict_helper(g: &EGraph, root: EClassId, e: &Extraction) -> bool {
    if e.len() == 0 || e.last().unwrap().c != root {
        // root
        eprintln!("Error: The last element of the extraction must be the root.");
        return false;
    }
    let mut i: ExtractionENodeId = (e.len() as ExtractionENodeId) - 1;
    while i >= 0 {
        let n: &ExtractionENode = &e[i as usize];
        if n.c < 0 || n.c >= g.eclasses.len() as EClassId {
            eprintln!("Error: Extraction referring to an eclass outside of bounds.");
            return false;
        }
        if n.n < 0 || n.n >= g.eclasses[n.c as usize].enodes.len() as ENodeId {
            eprintln!("Error: Extraction referring to an enode outside of bounds.");
            return false;
        }
        if n.ch.len() != g.eclasses[n.c as usize].enodes[n.n as usize].ch.len() {
            eprintln!("Error: Extraction referring to a wrong number of children.");
            return false;
        }
        for j in 0..n.ch.len() {
            let ch: ExtractionENodeId = n.ch[j];
            if ch < 0 || ch >= e.len() as ExtractionENodeId {
                // child present
                eprintln!("Error: Extraction referring to an index outside of bounds.");
                eprintln!("Found: {}", ch);
                return false;
            }
            let expected_child: EClassId = g.eclasses[n.c as usize].enodes[n.n as usize].ch[j];
            if e[ch as usize].c != expected_child {
                eprintln!("Error: Extraction referring to a child of wrong eclass.");
                return false;
            }
            if ch >= i {
                // acyclicity
                eprintln!("Error: Extraction may contain a loop.");
                return false;
            }
        }
        i -= 1;
    }
    // reachability does not really matter
    // unique choice not required
    true
}

pub fn is_valid_extraction(g: &EGraph, root: EClassId, e: &Extraction) -> bool {
    let ret: bool = is_valid_extraction_strict_helper(g, root, e);
    if !ret {
        debug_print_egraph(g);
        debug_print_extraction(g, e);
    }
    ret
}

pub fn is_effect_safe_extraction_helper(g: &EGraph, rootid: ExtractionENodeId, e: &Extraction, subregion_checked: &mut Vec<bool>) -> bool {
    let mut statewalk: Vec<ExtractionENodeId> = Vec::new();
    let mut vis: Vec<bool> = vec![false; e.len()];
    let mut onpath: Vec<bool> = vec![false; e.len()];
    let mut q: VecDeque<ExtractionENodeId> = VecDeque::new();
    statewalk.push(rootid);
    onpath[rootid as usize] = true;
    let mut i: usize = 0;
    while i < statewalk.len() {
        let u: ExtractionENodeId = statewalk[i];
        let mut nxt: ExtractionENodeId = -1;
        for j in 0..e[u as usize].ch.len() {
            let che: ExtractionENodeId = e[u as usize].ch[j];
            if g.eclasses[e[che as usize].c as usize].isEffectful {
                if nxt == -1 {
                    nxt = che;
                    statewalk.push(nxt);
                    onpath[nxt as usize] = true;
                } else if !is_effect_safe_extraction_helper(g, che, e, subregion_checked) {
                    return false;
                }
            } else {
                if !vis[che as usize] {
                    vis[che as usize] = true;
                    q.push_back(che);
                }
            }
        }
        i += 1;
    }
    // Check pure enodes only depend on the effectful walk in this region
    while q.len() > 0 {
        let u: ExtractionENodeId = *q.front().unwrap();
        q.pop_front();
        for i in 0..e[u as usize].ch.len() {
            let v: ExtractionENodeId = e[u as usize].ch[i];
            // assuming pure enodes can only have one effectful child
            if g.eclasses[e[v as usize].c as usize].isEffectful {
                if !onpath[v as usize] {
                    // using a effectul enode not in this region
                    eprintln!("Error: Using an effectul node not on the region statewalk");
                    return false;
                }
            } else {
                if !vis[v as usize] {
                    vis[v as usize] = true;
                    q.push_back(v);
                }
            }
        }
    }
    subregion_checked[rootid as usize] = true;
    true
}

pub fn is_effect_safe_extraction(g: &EGraph, root: EClassId, e: &Extraction) -> bool {
    if !is_valid_extraction(g, root, e) {
        return false;
    }
    // prevent double-checking each subregion
    let mut subregion_checked: Vec<bool> = vec![false; e.len()];
    let ret: bool = is_effect_safe_extraction_helper(g, (e.len() - 1) as ExtractionENodeId, e, &mut subregion_checked);
    if !ret {
        debug_print_egraph(g);
        debug_print_extraction(g, e);
    }
    ret
}
