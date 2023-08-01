use std::collections::HashMap;

use petgraph::{
    algo::dominators::{simple_fast, Dominators},
    prelude::NodeIndex,
    visit::EdgeRef,
};

use crate::EggCCError;

use super::{structured::StructuredBlock, BlockName, Cfg};

enum ContainingHistory {
    ThenBranch,
    LoopWithLabel(String),
    BlockFollowedBy(String),
}

pub(crate) struct StructuredCfgBuilder<'a> {
    context: Vec<ContainingHistory>,
    rpostorder: HashMap<BlockName, usize>,
    dominators: Dominators<NodeIndex>,
    cfg: &'a Cfg,
}

impl<'a> StructuredCfgBuilder<'a> {
    fn new(cfg: &'a Cfg) -> Self {
        let rpostorder = cfg.reverse_posorder();
        let dominators = simple_fast(&cfg.graph, cfg.entry);
        StructuredCfgBuilder {
            context: vec![],
            rpostorder,
            dominators,
            cfg,
        }
    }

    fn to_structured(&self) -> Result<StructuredBlock, EggCCError> {
        self.check_reducible()?;
        Ok(StructuredBlock::Sequence(vec![]))
    }

    fn check_reducible(&self) -> Result<(), EggCCError> {
        for edge in self.cfg.graph.edge_references() {
            let source = edge.source();
            let target = edge.target();
            // check if this is a back edge
            if self.rpostorder[&self.cfg.graph[source].name]
                > self.rpostorder[&self.cfg.graph[target].name]
            {
                // check if the target dominates the source
                if self
                    .dominators
                    .dominators(source)
                    .map(|mut dominators| !dominators.any(|a| a == target))
                    .unwrap_or(false)
                {
                    return Err(EggCCError::UnstructuredControlFlow);
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn to_structured(cfg: &Cfg) -> Result<StructuredBlock, EggCCError> {
    StructuredCfgBuilder::new(cfg).to_structured()
}
