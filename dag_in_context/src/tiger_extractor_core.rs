use crate::greedy_dag_extractor::get_root;
use crate::tiger_extractor_types::{
    TigerExtraction, TigerExtractionResult, TigerRegion, TigerRegionStats,
};
use crate::tiger_format::{build_tiger_egraph, TigerEGraph};
use egraph_serialize::{ClassId, EGraph};
use indexmap::{IndexMap, IndexSet};

/// Core Tiger extractor struct (split from monolithic implementation).
pub struct TigerExtractor<'a> {
    pub(crate) serialized: &'a EGraph,
    pub(crate) tiger: TigerEGraph,
}

impl<'a> TigerExtractor<'a> {
    pub fn new(serialized: &'a EGraph) -> Self {
        let tiger = build_tiger_egraph(serialized);
        Self { serialized, tiger }
    }
}

impl<'a> TigerExtractor<'a> {
    // Orchestrator kept here; methods it calls are defined across split modules.
    pub fn extract(&self, functions: &[String]) -> TigerExtractionResult {
        use crate::tiger_extractor_types::RegionSubEGraph;
        use crate::tiger_extractor_types::TigerExtractionENode; // not directly used but keeps parity
        use std::collections::HashMap;
        let mut chosen_enodes: IndexMap<ClassId, usize> = IndexMap::new();
        let mut state_walks: IndexMap<ClassId, Vec<ClassId>> = IndexMap::new();
        let mut regions: IndexMap<ClassId, Vec<TigerRegion>> = IndexMap::new();
        let mut region_stats: IndexMap<ClassId, Vec<TigerRegionStats>> = IndexMap::new();
        let mut extractions: IndexMap<ClassId, TigerExtraction> = IndexMap::new();
        let mut linearity_ok: IndexMap<ClassId, bool> = IndexMap::new();
        let mut guided_state_walks: IndexMap<ClassId, Vec<(ClassId, usize)>> = IndexMap::new();
        let mut weak_linearity_excess: IndexMap<ClassId, usize> = IndexMap::new();
        let mut weak_linearity_violation: IndexMap<ClassId, bool> = IndexMap::new();
        let mut debug_lines: Vec<String> = Vec::new();

        for func in functions {
            if let Some(root_body) = self.function_body_root(func) {
                let mut used_strategy = String::new();
                let mut wl_flag = false;
                let mut best: Option<(TigerExtraction, Vec<ClassId>, Vec<(ClassId, usize)>)> = None;
                if let Some((ex, walk, guided, wl)) =
                    self.advanced_recursive_multi_region_extraction(&root_body)
                {
                    wl_flag = wl;
                    best = Some((ex, walk, guided));
                    used_strategy = "recursive-multi-region".into();
                } else if let Some((ex, walk, guided, wl)) =
                    self.advanced_multi_region_extraction(&root_body)
                {
                    wl_flag = wl;
                    best = Some((ex, walk, guided));
                    used_strategy = "multi-region".into();
                } else if let Some((ex, walk, guided, wl)) =
                    self.advanced_region_extraction(&root_body)
                {
                    wl_flag = wl;
                    best = Some((ex, walk, guided));
                    used_strategy = "single-region".into();
                }
                let (extraction, walk_ids, guided_pairs) = if let Some(b) = best {
                    b
                } else {
                    used_strategy = "fallback-naive".into();
                    let walk_ids = self.build_state_walk(root_body.clone());
                    (self.naive_extraction(&root_body), walk_ids, Vec::new())
                };
                let lin_ok = self.region_linearity_check(&extraction)
                    && self.valid_extraction(&extraction, &root_body);
                if let Some(ridx) = extraction.root_index {
                    chosen_enodes.insert(root_body.clone(), extraction.nodes[ridx].enode_index);
                }
                state_walks.insert(root_body.clone(), walk_ids.clone());
                if !guided_pairs.is_empty() {
                    guided_state_walks.insert(root_body.clone(), guided_pairs.clone());
                }
                let eff_reach = self.effectful_reachable(&root_body);
                let excess = eff_reach
                    .len()
                    .saturating_sub(state_walks[&root_body].len());
                weak_linearity_excess.insert(root_body.clone(), excess);
                weak_linearity_violation.insert(root_body.clone(), wl_flag || excess > 0);
                let (rs, rs_stats) = self.build_regions_for_walk(&state_walks[&root_body]);
                regions.insert(root_body.clone(), rs);
                region_stats.insert(root_body.clone(), rs_stats);
                extractions.insert(root_body.clone(), extraction.clone());
                linearity_ok.insert(root_body.clone(), lin_ok);
                debug_lines.push(format!(
                    "func={} strategy={} lin_ok={} wl_violation={} excess={} regions={} nodes={}",
                    func,
                    used_strategy,
                    lin_ok,
                    weak_linearity_violation[&root_body],
                    weak_linearity_excess[&root_body],
                    regions[&root_body].len(),
                    extraction.nodes.len()
                ));
            }
        }
        TigerExtractionResult {
            chosen_enodes,
            state_walks,
            regions,
            region_stats,
            extractions,
            linearity_ok,
            debug: debug_lines.join("\n"),
            guided_state_walks,
            weak_linearity_excess,
            weak_linearity_violation,
        }
    }
}

impl<'a> TigerExtractor<'a> {
    pub(crate) fn function_body_root(&self, func: &str) -> Option<ClassId> {
        use egraph_serialize::NodeId;
        let root_nid: NodeId = get_root(self.serialized, func);
        Some(self.serialized.nid_to_cid(&root_nid).clone())
    }
}
