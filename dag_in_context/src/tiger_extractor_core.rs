use crate::greedy_dag_extractor::get_root;
use crate::tiger_extractor_types::{
    create_region_egraph, TigerExtraction, TigerExtractionENode, TigerExtractionResult,
    TigerRegion, TigerRegionStats,
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
        let mut guided_state_walks: IndexMap<ClassId, Vec<(ClassId, usize)>> = IndexMap::new();
        let mut function_roots: IndexMap<String, ClassId> = IndexMap::new();
        let mut debug_lines: Vec<String> = Vec::new();

        for func in functions {
            let Some(root_cid) = self.function_body_root(func) else {
                debug_lines.push(format!("func={} missing_root", func));
                continue;
            };
            let Some(&root_idx) = self.tiger.class_index.get(&root_cid) else {
                debug_lines.push(format!("func={} missing_tiger_class", func));
                continue;
            };
            if !self.tiger.eclasses[root_idx].is_effectful {
                debug_lines.push(format!("func={} skipped_pure_root", func));
                continue;
            }

            let mut region_root_id: Vec<Option<usize>> = vec![None; self.tiger.eclasses.len()];
            let mut region_roots: Vec<ClassId> = Vec::new();
            region_roots.push(root_cid.clone());
            region_root_id[root_idx] = Some(0);

            for ec in self.tiger.eclasses.iter() {
                if !ec.is_effectful {
                    continue;
                }
                for en in &ec.enodes {
                    let mut subregion_root = false;
                    for &child_idx in &en.children {
                        let child_ec = &self.tiger.eclasses[child_idx];
                        if child_ec.is_effectful {
                            if subregion_root {
                                if region_root_id[child_idx].is_none() {
                                    let new_id = region_roots.len();
                                    region_root_id[child_idx] = Some(new_id);
                                    region_roots.push(child_ec.original.clone());
                                }
                            } else {
                                subregion_root = true;
                            }
                        }
                    }
                }
            }

            let mut extracted_roots: Vec<Option<usize>> = vec![None; region_roots.len()];
            let mut extraction = TigerExtraction::new();
            let Some(root_region_id) = region_root_id[root_idx] else {
                debug_lines.push(format!("func={} missing_region_root_id", func));
                continue;
            };
            let Some(root_global_idx) = self.reconstruct_extraction(
                &region_roots,
                &region_root_id,
                &mut extracted_roots,
                &mut extraction,
                root_region_id,
            ) else {
                debug_lines.push(format!("func={} reconstruction_failed", func));
                continue;
            };
            extraction.root_index = Some(root_global_idx);

            assert!(self.valid_extraction(&extraction, &root_cid));
            assert!(self.region_linearity_check(&extraction));

            if let Some(ridx) = extraction.root_index {
                chosen_enodes.insert(root_cid.clone(), extraction.nodes[ridx].enode_index);
            }

            extractions.insert(root_cid.clone(), extraction.clone());
            linearity_ok.insert(root_cid.clone(), true);
            state_walks.insert(root_cid.clone(), Vec::new());
            weak_linearity_counts.insert(root_cid.clone(), IndexMap::new());
            weak_linearity_excess.insert(root_cid.clone(), 0);
            weak_linearity_violation.insert(root_cid.clone(), false);
            regions.insert(root_cid.clone(), Vec::new());
            region_stats.insert(root_cid.clone(), Vec::new());
            state_walk_pure_ordering.insert(root_cid.clone(), Vec::new());
            guided_state_walks.insert(root_cid.clone(), Vec::new());
            function_roots.insert(func.clone(), root_cid.clone());

            debug_lines.push(format!(
                "Function root: {} (idx={}) extracted_nodes={}",
                root_cid,
                root_idx,
                extraction.nodes.len()
            ));
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
            function_roots,
        }
    }

    fn reconstruct_extraction(
        &self,
        region_roots: &[ClassId],
        region_root_id: &[Option<usize>],
        extracted_roots: &mut Vec<Option<usize>>,
        extraction: &mut TigerExtraction,
        cur_region: usize,
    ) -> Option<usize> {
        if let Some(existing) = extracted_roots.get(cur_region).and_then(|v| *v) {
            return Some(existing);
        }

        let region_root = region_roots.get(cur_region)?;
        let rsub = create_region_egraph(&self.tiger, region_root);
        let (walk_pairs, _wl_flag, _wl_counts) = self.unguided_find_state_walk_region(&rsub);
        let region_extraction = if walk_pairs.is_empty() {
            self.scost_region_extraction(&rsub, region_root)?
        } else {
            self.region_extraction_with_state_walk(&rsub, &walk_pairs)?
        };

        let mut local_to_global: Vec<usize> = vec![usize::MAX; region_extraction.nodes.len()];
        for (idx, node) in region_extraction.nodes.iter().enumerate() {
            let &tiger_idx = self.tiger.class_index.get(&node.eclass)?;
            let original_enode = &self.tiger.eclasses[tiger_idx].enodes[node.enode_index];
            let mut child_iter = node.children.iter().copied();
            let mut final_children: Vec<usize> = Vec::with_capacity(original_enode.children.len());
            let mut seen_effectful = false;
            for &child_idx in &original_enode.children {
                let child_ec = &self.tiger.eclasses[child_idx];
                if child_ec.is_effectful {
                    if seen_effectful {
                        let region_idx = region_root_id.get(child_idx).copied().flatten()?;
                        let child_global = self.reconstruct_extraction(
                            region_roots,
                            region_root_id,
                            extracted_roots,
                            extraction,
                            region_idx,
                        )?;
                        final_children.push(child_global);
                    } else {
                        seen_effectful = true;
                        let local_child_idx = child_iter.next()?;
                        let mapped = *local_to_global.get(local_child_idx)?;
                        if mapped == usize::MAX {
                            return None;
                        }
                        final_children.push(mapped);
                    }
                } else {
                    let local_child_idx = child_iter.next()?;
                    let mapped = *local_to_global.get(local_child_idx)?;
                    if mapped == usize::MAX {
                        return None;
                    }
                    final_children.push(mapped);
                }
            }
            if child_iter.next().is_some() {
                return None;
            }

            let new_idx = extraction.add_node(TigerExtractionENode {
                eclass: node.eclass.clone(),
                enode_index: node.enode_index,
                children: final_children,
                original_node: node.original_node.clone(),
            });
            local_to_global[idx] = new_idx;
        }

        let root_local = region_extraction.root_index?;
        let root_global = local_to_global[root_local];
        extracted_roots[cur_region] = Some(root_global);
        Some(root_global)
    }
}

impl<'a> TigerExtractor<'a> {
    pub(crate) fn function_body_root(&self, func: &str) -> Option<ClassId> {
        use egraph_serialize::NodeId;
        let root_nid: NodeId = get_root(self.serialized, func);
        Some(self.serialized.nid_to_cid(&root_nid).clone())
    }
}
