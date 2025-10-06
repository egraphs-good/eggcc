use crate::greedy_dag_extractor::get_root;
use crate::tiger_extractor_types::{
    TigerExtraction, TigerExtractionResult, TigerRegion, TigerRegionStats,
};
use crate::tiger_format::{build_tiger_egraph, TigerEGraph};
use egraph_serialize::{ClassId, EGraph};
use indexmap::IndexMap;

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
        let mut chosen_enodes: IndexMap<ClassId, usize> = IndexMap::new();
        let mut state_walks: IndexMap<ClassId, Vec<ClassId>> = IndexMap::new();
        let mut regions: IndexMap<ClassId, Vec<TigerRegion>> = IndexMap::new();
        let mut region_stats: IndexMap<ClassId, Vec<TigerRegionStats>> = IndexMap::new();
        let mut extractions: IndexMap<ClassId, TigerExtraction> = IndexMap::new();
        let mut linearity_ok: IndexMap<ClassId, bool> = IndexMap::new();
        let mut weak_linearity_excess: IndexMap<ClassId, usize> = IndexMap::new();
        let mut weak_linearity_violation: IndexMap<ClassId, bool> = IndexMap::new();
        let mut weak_linearity_counts: IndexMap<ClassId, IndexMap<ClassId, u32>> = IndexMap::new();
        let mut state_walk_pure_ordering: IndexMap<ClassId, Vec<ClassId>> = IndexMap::new();
        let mut debug_lines: Vec<String> = Vec::new();
        let guided_state_walks: IndexMap<ClassId, Vec<(ClassId, usize)>> = IndexMap::new();

        for func in functions {
            if let Some(root_body) = self.function_body_root(func) {
                let mut used_strategy = String::new();
                let mut wl_flag = false;
                let mut best: Option<(TigerExtraction, Vec<ClassId>, IndexMap<ClassId, u32>)> =
                    None;
                let root = root_body.clone();
                // Parity with legacy C++: disable multi/recursive region strategies.
                // if let Some((ex, walk, wl, wlcounts)) = self
                //     .advanced_recursive_multi_region_extraction(&root)
                //     .map(|(a, b, _c, d, e)| (a, b, d, e))
                // {
                //     wl_flag = wl;
                //     best = Some((ex, walk, wlcounts));
                //     used_strategy = "recursive-multi-region".into();
                // } else if let Some((ex, walk, wl, wlcounts)) = self
                //     .advanced_multi_region_extraction(&root)
                //     .map(|(a, b, _c, d, e)| (a, b, d, e))
                // {
                //     wl_flag = wl;
                //     best = Some((ex, walk, wlcounts));
                //     used_strategy = "multi-region".into();
                // } else
                if let Some((ex, walk, wl, wlcounts)) = self
                    .advanced_region_extraction(&root)
                    .map(|(a, b, _c, d, e)| (a, b, d, e))
                {
                    wl_flag = wl;
                    best = Some((ex, walk, wlcounts));
                    used_strategy = "single-region".into();
                }
                let (extraction, walk_ids, wlcounts) = if let Some(b) = best {
                    b
                } else {
                    used_strategy = "fallback-naive".into();
                    let walk_ids = self.build_state_walk(root.clone());
                    (self.naive_extraction(&root), walk_ids, IndexMap::new())
                };
                let lin_ok = self.region_linearity_check(&extraction)
                    && self.valid_extraction(&extraction, &root);
                if let Some(ridx) = extraction.root_index {
                    chosen_enodes.insert(root.clone(), extraction.nodes[ridx].enode_index);
                }
                state_walks.insert(root.clone(), walk_ids.clone());
                let eff_reach = self.effectful_reachable(&root);
                let excess = eff_reach.len().saturating_sub(state_walks[&root].len());
                weak_linearity_excess.insert(root.clone(), excess);
                weak_linearity_violation.insert(root.clone(), wl_flag || excess > 0);
                // Previously conditional on guided_state_walks; now always empty
                let rs: Vec<TigerRegion> = Vec::new();
                let rs_stats: Vec<TigerRegionStats> = Vec::new();
                regions.insert(root.clone(), rs);
                region_stats.insert(root.clone(), rs_stats);
                extractions.insert(root.clone(), extraction.clone());
                linearity_ok.insert(root.clone(), lin_ok);
                weak_linearity_counts.insert(root.clone(), wlcounts);
                // Derive pure ordering diagnostic (use guided if available else unguided walk mapping with dummy enode index 0)
                let walk_pairs: Vec<(ClassId, usize)> = Vec::new();
                let pure_ord = self.analyze_state_walk_ordering(&walk_pairs, None);
                state_walk_pure_ordering.insert(root.clone(), pure_ord);
                debug_lines.push(format!(
                    "func={} strategy={} lin_ok={} wl_violation={} excess={} regions={} nodes={}",
                    func,
                    used_strategy,
                    lin_ok,
                    weak_linearity_violation[&root],
                    weak_linearity_excess[&root],
                    regions[&root].len(),
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
            weak_linearity_counts,
            state_walk_pure_ordering,
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
