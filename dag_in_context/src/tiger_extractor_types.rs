use crate::tiger_format::{TigerEClass, TigerEGraph, TigerENode};
use egraph_serialize::ClassId;
use indexmap::{IndexMap, IndexSet};

pub type ExtractableSet = Vec<bool>;

#[derive(Debug, Clone)]
pub struct TigerRegion {
    pub anchor: ClassId,
    pub next_anchor: Option<ClassId>,
    pub members: IndexSet<ClassId>,
}

#[derive(Debug, Clone)]
pub struct TigerRegionStats {
    pub total_enodes: usize,
    pub effectful_enodes: usize,
    pub pure_enodes: usize,
}

#[derive(Debug, Clone)]
pub struct TigerExtractionENode {
    pub eclass: ClassId,
    pub enode_index: usize,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct TigerExtraction {
    pub nodes: Vec<TigerExtractionENode>,
    pub root_index: Option<usize>,
}
impl TigerExtraction {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            root_index: None,
        }
    }
    pub fn add_node(&mut self, node: TigerExtractionENode) -> usize {
        let i = self.nodes.len();
        self.nodes.push(node);
        i
    }
}

#[derive(Debug, Clone)]
pub struct RegionSubEGraph {
    pub egraph: TigerEGraph,
    pub orig_to_region: IndexMap<ClassId, usize>,
    pub region_to_orig: Vec<ClassId>,
    pub n_subregion: Vec<Vec<usize>>, // mirrors nsubregion
}
impl RegionSubEGraph {
    pub fn size(&self) -> usize {
        self.egraph.eclasses.len()
    }
}

pub fn create_region_egraph(tiger: &TigerEGraph, region_root: &ClassId) -> RegionSubEGraph {
    use indexmap::IndexMap;
    let mut orig_to_region: IndexMap<ClassId, usize> = IndexMap::new();
    let mut region_to_orig: Vec<ClassId> = Vec::new();
    let mut n_subregion: Vec<Vec<usize>> = Vec::new();
    fn ensure_mapping(
        tiger: &TigerEGraph,
        cid: &ClassId,
        o2r: &mut IndexMap<ClassId, usize>,
        r2o: &mut Vec<ClassId>,
        nsub: &mut Vec<Vec<usize>>,
    ) {
        if o2r.contains_key(cid) {
            return;
        }
        if let Some(&t_idx) = tiger.class_index.get(cid) {
            let row_len = tiger.eclasses[t_idx].enodes.len();
            let new_idx = r2o.len();
            r2o.push(cid.clone());
            o2r.insert(cid.clone(), new_idx);
            nsub.push(vec![0; row_len]);
        }
    }
    ensure_mapping(
        tiger,
        region_root,
        &mut orig_to_region,
        &mut region_to_orig,
        &mut n_subregion,
    );
    let mut idx = 0;
    while idx < region_to_orig.len() {
        let orig = region_to_orig[idx].clone();
        let Some(&t_idx) = tiger.class_index.get(&orig) else {
            idx += 1;
            continue;
        };
        let tec = &tiger.eclasses[t_idx];
        let r_idx = *orig_to_region.get(&orig).unwrap();
        for (en_i, en) in tec.enodes.iter().enumerate() {
            let mut seen = false;
            for &ch in &en.children {
                let child_ec = &tiger.eclasses[ch];
                if child_ec.is_effectful {
                    if seen {
                        n_subregion[r_idx][en_i] += 1;
                        continue;
                    }
                    seen = true;
                    ensure_mapping(
                        tiger,
                        &child_ec.original,
                        &mut orig_to_region,
                        &mut region_to_orig,
                        &mut n_subregion,
                    );
                }
            }
        }
        idx += 1;
    }
    let mut idx2 = 0;
    while idx2 < region_to_orig.len() {
        let orig = region_to_orig[idx2].clone();
        let Some(&t_idx) = tiger.class_index.get(&orig) else {
            idx2 += 1;
            continue;
        };
        let tec = &tiger.eclasses[t_idx];
        for en in &tec.enodes {
            for &ch in &en.children {
                let child_ec = &tiger.eclasses[ch];
                if !child_ec.is_effectful {
                    ensure_mapping(
                        tiger,
                        &child_ec.original,
                        &mut orig_to_region,
                        &mut region_to_orig,
                        &mut n_subregion,
                    );
                }
            }
        }
        idx2 += 1;
    }
    let mut region_eclasses: Vec<TigerEClass> = Vec::with_capacity(region_to_orig.len());
    for (r_idx, orig) in region_to_orig.iter().enumerate() {
        let Some(&t_idx) = tiger.class_index.get(orig) else {
            continue;
        };
        let orig_class = &tiger.eclasses[t_idx];
        let mut new_class = TigerEClass {
            enodes: vec![],
            is_effectful: orig_class.is_effectful,
            original: orig.clone(),
        };
        for en in &orig_class.enodes {
            let mut new_children = Vec::new();
            let mut seen = false;
            for &ch in &en.children {
                let child_ec = &tiger.eclasses[ch];
                let child_cid = child_ec.original.clone();
                if child_ec.is_effectful {
                    if seen {
                        continue;
                    }
                    seen = true;
                }
                if let Some(&mapped) = orig_to_region.get(&child_cid) {
                    new_children.push(mapped);
                }
            }
            new_class.enodes.push(TigerENode {
                head: en.head.clone(),
                eclass_idx: r_idx,
                children: new_children,
                original_class: orig.clone(),
                original_node: en.original_node.clone(),
            });
        }
        region_eclasses.push(new_class);
    }
    let mut region_class_index: IndexMap<ClassId, usize> = IndexMap::new();
    for (i, cid) in region_to_orig.iter().enumerate() {
        region_class_index.insert(cid.clone(), i);
    }
    let region_graph = TigerEGraph {
        eclasses: region_eclasses,
        class_index: region_class_index,
    };
    RegionSubEGraph {
        egraph: region_graph,
        orig_to_region,
        region_to_orig,
        n_subregion,
    }
}

pub struct TigerExtractionResult {
    pub chosen_enodes: IndexMap<ClassId, usize>,
    pub state_walks: IndexMap<ClassId, Vec<ClassId>>,
    pub regions: IndexMap<ClassId, Vec<TigerRegion>>,
    pub region_stats: IndexMap<ClassId, Vec<TigerRegionStats>>,
    pub extractions: IndexMap<ClassId, TigerExtraction>,
    pub linearity_ok: IndexMap<ClassId, bool>,
    pub debug: String,
    pub guided_state_walks: IndexMap<ClassId, Vec<(ClassId, usize)>>,
    pub weak_linearity_excess: IndexMap<ClassId, usize>,
    pub weak_linearity_violation: IndexMap<ClassId, bool>,
    pub weak_linearity_counts: IndexMap<ClassId, IndexMap<ClassId, u32>>,
    pub state_walk_pure_ordering: IndexMap<ClassId, Vec<ClassId>>,
}
