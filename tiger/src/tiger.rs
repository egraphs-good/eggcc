// Port of tiger.h / tiger.cpp — top-level orchestration of statewalk -> rebuild -> prune -> greedy.
// Direct line-by-line translation.

use crate::egraphin::{
    inverse_egraph_mapping, project_extraction, prune_unextractable_enodes, EClass, EClassId,
    EGraph, EGraphMapping, ENodeId, Extraction,
};
use crate::greedy::statewalk_greedy_extraction;
use crate::statewalkdp::{statewalkDP, Statewalk, StatewalkWidthStat};

pub type Cost = u64;

pub fn rebuild_egraph_statewalk(g: &EGraph, sw: &Statewalk) -> (EGraph, EGraphMapping) {
    let mut gp: EGraph = EGraph::default();
    let mut gp2g: EGraphMapping = EGraphMapping::default();
    gp.eclasses.resize(g.neclasses(), EClass::default());
    gp2g.eclassidmp.resize(g.neclasses(), 0);
    gp2g.enodeidmp.resize(g.neclasses(), Vec::new());
    for i in 0..(g.neclasses() as EClassId) {
        gp2g.eclassidmp[i as usize] = i;
        let c: &EClass = &g.eclasses[i as usize];
        if !c.isEffectful {
            gp.eclasses[i as usize] = c.clone();
            gp2g.enodeidmp[i as usize].resize(c.nenodes(), 0);
            for j in 0..(c.nenodes() as ENodeId) {
                gp2g.enodeidmp[i as usize][j as usize] = j;
            }
        } else {
            gp.eclasses[i as usize].isEffectful = true;
        }
    }
    let mut last: EClassId = -1;
    let mut i: i32 = (sw.len() as i32) - 1;
    while i >= 0 {
        let uc: EClassId = sw[i as usize].0;
        let vc: EClassId;
        let un: ENodeId = sw[i as usize].1;
        if gp.eclasses[uc as usize].nenodes() == 0 {
            let n_clone = g.eclasses[uc as usize].enodes[un as usize].clone();
            gp.eclasses[uc as usize].enodes.push(n_clone);
            vc = uc;
            gp2g.enodeidmp[vc as usize].push(un);
        } else {
            vc = gp.neclasses() as EClassId;
            let mut c: EClass = EClass::default();
            c.isEffectful = true;
            let n_clone = g.eclasses[uc as usize].enodes[un as usize].clone();
            c.enodes.push(n_clone);
            c.enodes[0].eclass = vc;
            gp.eclasses.push(c);
            gp2g.eclassidmp.push(uc);
            gp2g.enodeidmp.push(vec![un]);
        }
        let ch_len: usize = gp.eclasses[vc as usize].enodes[0].ch.len();
        for j in 0..ch_len {
            let chc: EClassId = gp.eclasses[vc as usize].enodes[0].ch[j];
            if g.eclasses[chc as usize].isEffectful {
                gp.eclasses[vc as usize].enodes[0].ch[j] = last;
            }
        }
        last = vc;
        i -= 1;
    }
    crate::debug_assert_tiger!(crate::debug::is_wellformed_egraph(&gp, true, false));
    crate::debug_assert_tiger!(crate::debug::is_valid_egraph_mapping(&gp2g, &gp, g, false, false, false, true));
    (gp, gp2g)
}

pub fn extract_regionalized_egraph_tiger(
    g: &EGraph,
    root: EClassId,
    statewalk_cost: &Vec<Vec<Cost>>,
    use_liveness: bool,
    use_satellite_opt: bool,
) -> Extraction {
    let sw: Statewalk = statewalkDP(g, root, statewalk_cost, use_liveness, use_satellite_opt, None);

    let res: (EGraph, EGraphMapping) = rebuild_egraph_statewalk(g, &sw);
    let gp: &EGraph = &res.0;
    let gp2g: &EGraphMapping = &res.1;
    let res2: (EGraph, EGraphMapping) = prune_unextractable_enodes(gp, root);
    let gpp: &EGraph = &res2.0;
    let gp2gpp: &EGraphMapping = &res2.1;
    // root eclass id is unchanged from g to gp
    let nroot: EClassId = gp2gpp.eclassidmp[root as usize];
    let e: Extraction = project_extraction(
        gp2g,
        &project_extraction(&inverse_egraph_mapping(gpp, gp2gpp), &statewalk_greedy_extraction(gpp, nroot)),
    );
    crate::debug_assert_tiger!(crate::debug::is_effect_safe_extraction(g, root, &e));
    e
}

pub struct StatewalkWidthReport {
    pub max_width: usize,
    pub avg_width: f64,
}

impl StatewalkWidthReport {
    pub fn new(data: &Vec<usize>) -> Self {
        let max_width = *data.iter().max().unwrap();
        let avg_width = data.iter().map(|x| *x as i64).sum::<i64>() as f64 / data.len() as f64;
        Self { max_width, avg_width }
    }
}

pub struct StatewalkWidthReports {
    pub liveon_satelliteon: StatewalkWidthReport,
    pub liveon_satelliteoff: StatewalkWidthReport,
    pub liveoff_satelliteon: StatewalkWidthReport,
    pub liveoff_satelliteoff: StatewalkWidthReport,
}

impl StatewalkWidthReports {
    pub fn new(
        liveon_sat_on: StatewalkWidthReport,
        liveon_sat_off: StatewalkWidthReport,
        liveoff_sat_on: StatewalkWidthReport,
        liveoff_sat_off: StatewalkWidthReport,
    ) -> Self {
        Self {
            liveon_satelliteon: liveon_sat_on,
            liveon_satelliteoff: liveon_sat_off,
            liveoff_satelliteon: liveoff_sat_on,
            liveoff_satelliteoff: liveoff_sat_off,
        }
    }
}

pub fn get_stat_regionalized_egraph_tiger(
    g: &EGraph,
    root: EClassId,
    statewalk_cost: &Vec<Vec<Cost>>,
) -> StatewalkWidthReports {
    let mut liveness_satelliteon: StatewalkWidthStat = StatewalkWidthStat::new();
    let mut liveness_satelliteoff: StatewalkWidthStat = StatewalkWidthStat::new();
    let mut noliveness_satelliteon: StatewalkWidthStat = StatewalkWidthStat::new();
    let mut noliveness_satelliteoff: StatewalkWidthStat = StatewalkWidthStat::new();

    statewalkDP(g, root, statewalk_cost, true, true, Some(&mut liveness_satelliteon));
    statewalkDP(g, root, statewalk_cost, true, false, Some(&mut liveness_satelliteoff));
    statewalkDP(g, root, statewalk_cost, false, true, Some(&mut noliveness_satelliteon));
    statewalkDP(g, root, statewalk_cost, false, false, Some(&mut noliveness_satelliteoff));

    StatewalkWidthReports::new(
        StatewalkWidthReport::new(&liveness_satelliteon),
        StatewalkWidthReport::new(&liveness_satelliteoff),
        StatewalkWidthReport::new(&noliveness_satelliteon),
        StatewalkWidthReport::new(&noliveness_satelliteoff),
    )
}
