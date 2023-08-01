use std::collections::HashMap;

use petgraph::{
    algo::dominators::{simple_fast, Dominators},
    visit::{DfsPostOrder, EdgeRef, Walker},
};

use crate::EggCCError;

use super::{structured::StructuredBlock, BlockName, Cfg};

pub(crate) struct StructuredCfgBuilder;

impl StructuredCfgBuilder {
    fn new() -> Self {
        StructuredCfgBuilder
    }

    fn to_structured(&self, cfg: &Cfg) -> Result<StructuredBlock, EggCCError> {
        let rpostorder = self.reverse_posorder(cfg);
        let dominators = simple_fast(&cfg.graph, cfg.entry);
        for edge in cfg.graph.edge_references() {
            let source = edge.source();
            let target = edge.target();
            // check if this is a back edge
            if rpostorder[&cfg.graph[source].name] > rpostorder[&cfg.graph[target].name] {
                // check if the target dominates the source
                if dominators
                    .dominators(source)
                    .map(|mut dominators| !dominators.any(|a| a == target))
                    .unwrap_or(false)
                {
                    return Err(EggCCError::UnstructuredControlFlow);
                }
            }
        }

        Ok(StructuredBlock::Sequence(vec![]))
    }

    fn reverse_posorder(&self, cfg: &Cfg) -> HashMap<BlockName, usize> {
        let mut reverse_postorder = HashMap::<BlockName, usize>::new();
        let mut post_counter = 0;
        DfsPostOrder::new(&cfg.graph, cfg.entry)
            .iter(&cfg.graph)
            .for_each(|node| {
                reverse_postorder.insert(cfg.graph[node].name.clone(), post_counter);
                post_counter += 1;
            });

        reverse_postorder
    }
}

pub(crate) fn to_structured(cfg: &Cfg) -> Result<StructuredBlock, EggCCError> {
    StructuredCfgBuilder::new().to_structured(cfg)
}
