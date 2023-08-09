use std::collections::HashMap;

use petgraph::{
    algo::dominators::{simple_fast, Dominators},
    graph::EdgeReference,
    prelude::NodeIndex,
    visit::EdgeRef,
};

use crate::{cfg::BasicBlock, EggCCError};

use super::{
    structured::{StructuredBlock, StructuredFunction},
    BlockName, Branch, BranchOp, Cfg,
};

#[derive(Debug)]
enum ContainingHistory {
    ThenBranch,
    LoopWithLabel(BlockName),
    BlockFollowedBy(BlockName),
}

pub(crate) struct StructuredCfgBuilder<'a> {
    context: Vec<ContainingHistory>,
    postorder: HashMap<BlockName, usize>,
    dominators: Dominators<NodeIndex>,
    cfg: &'a Cfg,
}

impl<'a> StructuredCfgBuilder<'a> {
    fn new(cfg: &'a Cfg) -> Self {
        let postorder = cfg.reverse_posorder();
        let dominators = simple_fast(&cfg.graph, cfg.entry);
        StructuredCfgBuilder {
            context: vec![],
            postorder,
            dominators,
            cfg,
        }
    }

    fn convert_structured(&mut self) -> Result<StructuredFunction, EggCCError> {
        self.check_reducible()?;
        let result = self.do_tree(self.cfg.entry);
        Ok(StructuredFunction {
            name: self.cfg.name.clone(),
            args: self.cfg.args.clone(),
            block: result,
        })
    }

    fn do_tree(&mut self, node: NodeIndex) -> StructuredBlock {
        if self.is_loop_header(node) {
            self.context
                .push(ContainingHistory::LoopWithLabel(self.name(node)));
            let body = StructuredBlock::Loop(Box::new(self.code_for_node(node)));
            self.context.pop();
            body
        } else {
            self.code_for_node(node)
        }
    }

    fn name(&self, node: NodeIndex) -> BlockName {
        self.cfg.graph[node].name.clone()
    }

    fn code_for_node(&mut self, node: NodeIndex) -> StructuredBlock {
        let mut merge_nodes = self
            .dominators
            .immediately_dominated_by(node)
            .filter(|n| self.is_merge_node(*n))
            .collect::<Vec<_>>();
        merge_nodes.sort_by_key(|n| self.postorder[&self.cfg.graph[*n].name]);
        self.node_within(node, merge_nodes)
    }

    fn node_within(&mut self, node: NodeIndex, merge_nodes: Vec<NodeIndex>) -> StructuredBlock {
        if node == self.cfg.exit {
            return StructuredBlock::Basic(Box::new(self.cfg.graph[node].clone()));
        }

        let edges = self
            .cfg
            .graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .collect::<Vec<_>>();
        match merge_nodes.as_slice() {
            [] => {
                StructuredBlock::Sequence(vec![
                    StructuredBlock::Basic(Box::new(self.cfg.graph[node].clone())),
                    match edges.as_slice() {
                        [] => {
                            panic!("handled above");
                        }
                        // Unconditional
                        [out] => self.do_branch(out),
                        [branch1, branch2] => {
                            if let (
                                Branch {
                                    op:
                                        BranchOp::Cond {
                                            val: val1,
                                            arg: arg1,
                                        },
                                    ..
                                },
                                Branch {
                                    op: BranchOp::Cond { val: val2, .. },
                                    ..
                                },
                            ) = (branch1.weight(), branch2.weight())
                            {
                                assert!(val1 != val2);
                                self.context.push(ContainingHistory::ThenBranch);
                                let then_block =
                                    self.do_branch(if *val1 { branch1 } else { branch2 });
                                let else_block =
                                    self.do_branch(if !*val1 { branch1 } else { branch2 });
                                self.context.pop();
                                StructuredBlock::Ite(
                                    arg1.to_string(),
                                    Box::new(then_block),
                                    Box::new(else_block),
                                )
                            } else {
                                panic!(
                                    "Expected two conditional branches. Got {:?} and {:?}",
                                    branch1, branch2
                                );
                            }
                        }
                        _ => {
                            panic!("Expected at most two outgoing edges. Got {:?}", edges);
                        }
                    },
                ])
            }
            [first, ..] => {
                self.context
                    .push(ContainingHistory::BlockFollowedBy(self.name(*first)));
                let rest = self.node_within(node, merge_nodes[1..].to_vec());
                self.context.pop();
                StructuredBlock::Sequence(vec![
                    StructuredBlock::Block(Box::new(rest)),
                    self.do_tree(*first),
                ])
            }
        }
    }
    fn do_branch(&mut self, edge: &EdgeReference<Branch>) -> StructuredBlock {
        let source = edge.source();
        let target = edge.target();
        let target_block = self.cfg.graph[target].clone();
        if target_block.name == BlockName::Exit {
            assert!(target_block.instrs.is_empty());

            match &edge.weight().op {
                BranchOp::Jmp => StructuredBlock::Return(None),
                BranchOp::RetVal { arg } => StructuredBlock::Return(Some(arg.clone())),
                _ => {
                    panic!("Unexpected branch op {:?}", edge.weight().op);
                }
            }
        } else if self.is_backward_edge(source, target) || self.is_merge_node(target) {
            let index = self.context_index(self.cfg.graph[target].name.clone());
            StructuredBlock::Break(index)
        } else {
            self.do_tree(target)
        }
    }

    fn context_index(&self, target: BlockName) -> i64 {
        for (index, context) in self.context.iter().rev().enumerate() {
            match context {
                ContainingHistory::ThenBranch => {}
                ContainingHistory::LoopWithLabel(label) => {
                    if label == &target {
                        return index.try_into().unwrap();
                    }
                }
                ContainingHistory::BlockFollowedBy(label) => {
                    if label == &target {
                        return index.try_into().unwrap();
                    }
                }
            }
        }
        panic!(
            "Could not find target {:?} in context {:?}",
            target, self.context
        );
    }

    fn is_backward_edge(&self, source: NodeIndex, target: NodeIndex) -> bool {
        self.postorder[&self.cfg.graph[target].name] > self.postorder[&self.cfg.graph[source].name]
    }

    fn is_merge_node(&self, node: NodeIndex) -> bool {
        self.cfg
            .graph
            .neighbors_directed(node, petgraph::Direction::Incoming)
            .take(1)
            .next()
            .is_some()
    }

    fn is_loop_header(&self, node: NodeIndex) -> bool {
        self.cfg
            .graph
            .edges_directed(node, petgraph::Direction::Incoming)
            .any(|edge| self.is_backward_edge(edge.source(), node))
    }

    fn check_reducible(&self) -> Result<(), EggCCError> {
        for edge in self.cfg.graph.edge_references() {
            let source = edge.source();
            let target = edge.target();
            // check if this is a back edge
            if self.is_backward_edge(source, target) {
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

pub(crate) fn to_structured(cfg: &Cfg) -> Result<StructuredFunction, EggCCError> {
    StructuredCfgBuilder::new(cfg).convert_structured()
}
