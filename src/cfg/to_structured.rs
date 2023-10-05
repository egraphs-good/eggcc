use std::collections::HashMap;

use petgraph::{
    algo::dominators::{self, Dominators},
    prelude::NodeIndex,
    stable_graph::EdgeReference,
    visit::{EdgeRef, IntoEdgeReferences},
};

use crate::{
    cfg::{Annotation, CondVal, Identifier},
    EggCCError,
};

use super::{
    structured::{StructuredBlock, StructuredFunction, StructuredProgram},
    BlockName, Branch, BranchOp, SimpleCfgFunction, SimpleCfgProgram,
};

/// Records the history of the current node in the CFG
/// being processed.
/// For example, BlockFollowedBy(BlockName) means that the current
/// cfg block being proessed is in a structured block followed by code for
/// the block with the given name.
#[derive(Debug)]
enum ContainingHistory {
    ThenBranch,
    LoopWithLabel(BlockName),
    BlockFollowedBy(BlockName),
}

#[derive(Debug)]
struct Context {
    enclosing: ContainingHistory,
    fallthrough: Option<BlockName>,
}

pub(crate) struct StructuredCfgBuilder<'a> {
    context: Vec<Context>, // last element is newest context
    postorder: HashMap<BlockName, usize>,
    dominators: Dominators<NodeIndex>,
    cfg: &'a SimpleCfgFunction,
}

impl<'a> StructuredCfgBuilder<'a> {
    fn new(cfg: &'a SimpleCfgFunction) -> Self {
        let postorder = cfg.reverse_postorder();
        let dominators = dominators::simple_fast(&cfg.graph, cfg.entry);
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

    /// Convert a node and all it's children in the dominator tree
    /// to a structured representation.
    fn do_tree(&mut self, node: NodeIndex) -> StructuredBlock {
        if self.is_loop_header(node) {
            self.context.push(Context {
                enclosing: ContainingHistory::LoopWithLabel(self.name(node)),
                fallthrough: Some(self.name(node)),
            });
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
        assert!(
            !edges.is_empty(),
            "edges should not be empty for non-exit block {:?}",
            self.name(node)
        );
        match merge_nodes.as_slice() {
            [] => {
                let first = StructuredBlock::Basic(Box::new(self.cfg.graph[node].clone()));
                let second = match edges.as_slice() {
                    [] => {
                        panic!("handled above");
                    }
                    // Unconditionally jumps to out
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
                            self.context.push(Context {
                                enclosing: ContainingHistory::ThenBranch,
                                fallthrough: None,
                            });
                            let then_block = self
                                .do_branch(if val1 == &CondVal::from(true) {
                                    branch1
                                } else {
                                    branch2
                                })
                                .unwrap();
                            let else_block = self
                                .do_branch(if val1 == &CondVal::from(false) {
                                    branch1
                                } else {
                                    branch2
                                })
                                .unwrap();
                            self.context.pop();
                            Some(StructuredBlock::Ite(
                                arg1.to_string(),
                                Box::new(then_block),
                                Box::new(else_block),
                            ))
                        } else {
                            panic!(
                                "Expected two conditional branches for node {}. Got {:?} and {:?}",
                                self.name(node),
                                branch1,
                                branch2
                            );
                        }
                    }
                    _ => {
                        panic!("Expected at most two outgoing edges. Got {:?}", edges);
                    }
                };
                if let Some(block) = second {
                    StructuredBlock::Sequence(vec![first, block])
                } else {
                    first
                }
            }
            [first, ..] => {
                self.context.push(Context {
                    enclosing: ContainingHistory::BlockFollowedBy(self.name(*first)),
                    fallthrough: Some(self.name(*first)),
                });
                let rest = self.node_within(node, merge_nodes[1..].to_vec());
                self.context.pop();
                StructuredBlock::Sequence(vec![
                    StructuredBlock::Block(Box::new(rest)),
                    self.do_tree(*first),
                ])
            }
        }
    }

    fn do_branch(&mut self, edge: &EdgeReference<Branch>) -> Option<StructuredBlock> {
        let source = edge.source();
        let target = edge.target();
        let target_block = self.cfg.graph[target].clone();
        if target_block.name == BlockName::Exit {
            assert!(target_block.instrs.is_empty());

            match &edge.weight().op {
                BranchOp::Jmp => {
                    if let Some(ret_val) =
                        self.cfg.graph[source]
                            .footer
                            .iter()
                            .find_map(|ann| match ann {
                                Annotation::AssignRet { src: Identifier::Name(src) } => Some(&**src),
                                Annotation::AssignRet { src: Identifier::Num(_) } => panic!("using placeholder identifier as return value (unsupported for structured IR)"),
                                Annotation::AssignCond {..} => None,
                            })
                    {
                        Some(StructuredBlock::Return(Some(ret_val.into())))
                    } else {
                        Some(StructuredBlock::Return(None))
                    }
                }
                _ => {
                    panic!("Unexpected branch op {:?}", edge.weight().op);
                }
            }
        } else if self.is_backward_edge(source, target) || self.is_merge_node(target) {
            self.break_out_to(self.name(target))
        } else {
            Some(self.do_tree(target))
        }
    }

    fn break_out_to(&self, target: BlockName) -> Option<StructuredBlock> {
        assert!(!self.context.is_empty(), "context should not be empty");
        let top_context = self.context.last().unwrap();
        for (index, context) in self.context.iter().rev().enumerate() {
            match &context.enclosing {
                ContainingHistory::ThenBranch => {}
                ContainingHistory::LoopWithLabel(label)
                | ContainingHistory::BlockFollowedBy(label) => {
                    if label == &target {
                        if let Some(true) = top_context
                            .fallthrough
                            .as_ref()
                            .map(|fallthrough_label| fallthrough_label == &target)
                        {
                            return None;
                        } else {
                            return Some(StructuredBlock::Break(index));
                        }
                    }
                }
            }
        }
        panic!(
            "Could not find target {:?} in context {:?}. Options are {:?}",
            target, self.context, self.context
        );
    }

    fn is_backward_edge(&self, source: NodeIndex, target: NodeIndex) -> bool {
        self.postorder[&self.cfg.graph[target].name] >= self.postorder[&self.cfg.graph[source].name]
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

    /// Check if this cfg is reducible,
    /// which means that it can be represented as a StructuredBlock
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

pub(crate) fn cfg_to_structured(cfg: &SimpleCfgProgram) -> Result<StructuredProgram, EggCCError> {
    let mut functions = vec![];
    for func in &cfg.functions {
        functions.push(StructuredCfgBuilder::new(func).convert_structured()?)
    }

    Ok(StructuredProgram { functions })
}
