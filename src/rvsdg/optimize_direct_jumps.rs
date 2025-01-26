//! This module takes as input a [`SimpleCfgProgram`] and gets rid of any
//! direct jumps between blocks.
//! The block that is jumped to must have only one predecessor.
//! This is used by `to_cfg` to clean up
//! the output.

use bril_rs::{EffectOps, Instruction, ValueOps};
use indexmap::IndexMap;
use petgraph::{
    stable_graph::{NodeIndex, StableDiGraph, StableGraph},
    visit::{DfsPostOrder, EdgeRef},
    Direction,
};

use crate::cfg::{Annotation, BasicBlock, Branch, Simple, SimpleCfgFunction, SimpleCfgProgram};

#[cfg(test)]
use crate::Optimizer;

impl SimpleCfgFunction {
    /// Optimize a CFG by collapsing blocks
    pub fn optimize_jumps(&self) -> Self {
        // fusing down only needs to happen once
        // however, the other passes may need to be run until fixed point
        // currently we run them twice and call it good
        let mut res = self
            .clone()
            .remove_footers()
            .add_explicit_return()
            .fuse_down()
            .fuze_up()
            .fuze_up()
            .return_early()
            .return_early()
            .collapse_empty_blocks()
            .collapse_empty_blocks();
        res.remove_unreachable();
        res
    }

    /// Converts footers into bril expressions to simplify this pass
    fn remove_footers(mut self) -> SimpleCfgFunction {
        for node in self.graph.node_indices().collect::<Vec<_>>() {
            for annotation in self.graph[node].footer.clone() {
                match annotation {
                    Annotation::AssignRet { src } => {
                        self.graph[node].instrs.push(Instruction::Effect {
                            op: EffectOps::Return,
                            args: vec![src.to_string()],
                            funcs: vec![],
                            labels: vec![],
                            pos: None,
                        });
                    }
                    Annotation::AssignCond { .. } => {
                        panic!("No assigncond should be present for simple cfgs");
                    }
                }
            }
            self.graph[node].footer.clear();
        }
        self
    }

    fn add_explicit_return(mut self) -> SimpleCfgFunction {
        // if the exit node has no return instructions
        if !self.has_return(self.exit) {
            // add a return instruction
            self.graph[self.exit].instrs.push(Instruction::Effect {
                op: EffectOps::Return,
                args: vec![],
                funcs: vec![],
                labels: vec![],
                pos: None,
            });
        }
        self
    }

    fn has_return(&self, node: NodeIndex) -> bool {
        // nodes can return through instructions, annotations, or by being the exit node implicitly
        self.graph[node].instrs.iter().any(|instr| {
            matches!(
                instr,
                Instruction::Effect {
                    op: EffectOps::Return,
                    ..
                }
            )
        })
    }

    /// Find blocks that return and fuze them with parent blocks that jump unconditionaly to them.
    /// All parents must jump unconditionally to the block.
    fn return_early(mut self) -> SimpleCfgFunction {
        // for each node
        for node in self.graph.node_indices().collect::<Vec<_>>() {
            // find all parents
            let parents = self
                .graph
                .edges_directed(node, Direction::Incoming)
                .map(|edge| edge.source())
                .collect::<Vec<_>>();

            // if the node contains a return, and all parents jump unconditionally to this node
            // and more than one parent
            if self.has_return(node)
                && parents.iter().all(|parent| {
                    self.graph
                        .edges_directed(*parent, Direction::Outgoing)
                        .count()
                        == 1
                })
                && !parents.is_empty()
            {
                // for each parent, fuze up
                for parent in &parents {
                    // if the parent doesn't contain return instructions
                    if !self.has_return(*parent) {
                        // if the parent jumps unconditionally to this node
                        if self
                            .graph
                            .edges_directed(*parent, Direction::Outgoing)
                            .count()
                            == 1
                        {
                            let instrs = self.graph[node].instrs.clone();
                            // move instructions from node up to parent
                            self.graph[*parent].instrs.extend(instrs);
                        };
                    }
                }
                // clear the instructions in this node
                self.graph[node].instrs.clear();
            }
        }

        self
    }

    /// Finds blocks with only id instructions and fuses them with their parents
    /// The parent must jump directly to the block
    /// A -> B -> C
    ///    /
    /// D
    ///
    /// If B has only id instructions, we can fuse it into both A and D.
    /// These id instructions are optimized away by register allocation in LLVM, so this is fine.
    /// If there are non-id instructions, this causes code blowup. Id instructions are "free"
    fn fuze_up(mut self) -> SimpleCfgFunction {
        for node in self.graph.node_indices().collect::<Vec<_>>() {
            let parents = self
                .graph
                .edges_directed(node, Direction::Incoming)
                .map(|edge| edge.source())
                .collect::<Vec<_>>();

            // check if fusing up is possible- instructions are all id
            // and parents directly jump to this block
            // Also needs at least one parent
            let should_apply = self.graph[node].instrs.iter().all(|instr| {
                matches!(
                    instr,
                    Instruction::Value {
                        op: ValueOps::Id,
                        ..
                    }
                )
            }) && parents.iter().all(|parent| {
                let parent_outgoing = self
                    .graph
                    .edges_directed(*parent, Direction::Outgoing)
                    .count();
                parent_outgoing == 1
            }) && !parents.is_empty();

            let new_instrs = self.graph[node].instrs.clone();
            // move instructions from node up to parents
            if should_apply {
                for parent in parents {
                    self.graph[parent].instrs.extend(new_instrs.clone());
                }

                // delete instructions from node
                self.graph[node].instrs.clear();
            }
        }

        self
    }

    /// Find cases where a block jumps directly to another block A -> B where
    /// A has only one outgoing edge and B has one incoming edge
    /// Turn it into one block AB
    fn fuse_down(&self) -> SimpleCfgFunction {
        let mut resulting_graph: StableGraph<BasicBlock, Branch> = StableDiGraph::new();

        // a map from nodes in the old graph to nodes in the
        // new graph
        // if a node was fused into another node,
        // it points to the new, fused node
        let mut node_mapping: IndexMap<NodeIndex, NodeIndex> = IndexMap::new();

        // we use a dfs post order
        // so dependencies are visited before parents
        // This ensures that `node_mapping[&next]` succeeds most of the time (can still fail with
        // some loops).
        let mut dfs = DfsPostOrder::new(&self.graph, self.entry);

        let mut edges_to_add = vec![];

        // copy the graph without the edges
        // also choose which nodes get fused to which
        // by re-assigning in the node map
        while let Some(node) = dfs.next(&self.graph) {
            let outgoing_from_node = self
                .graph
                .edges_directed(node, Direction::Outgoing)
                .collect::<Vec<_>>();
            let target = if let &[single_edge] = outgoing_from_node.as_slice() {
                let target = single_edge.target();
                let incoming_to_next = self
                    .graph
                    .edges_directed(target, Direction::Incoming)
                    .count();
                if incoming_to_next == 1 && target != node {
                    Some(target)
                } else {
                    None
                }
            } else {
                None
            };
            // single outgoing edge
            if target.is_some() && node_mapping.contains_key(&target.unwrap()) {
                let next = target.unwrap();
                let new_target = node_mapping[&next];
                // this node will be mapped to the previous
                node_mapping.insert(node, new_target);

                // add instructions to the beginning of the next node
                let mut new_instrs = self.graph[node].instrs.to_vec();
                new_instrs.extend(resulting_graph[new_target].instrs.to_vec());

                resulting_graph[new_target].instrs = new_instrs;
            } else {
                // add the node
                let new_node = resulting_graph.add_node(self.graph[node].clone());
                node_mapping.insert(node, new_node);

                edges_to_add.extend(self.graph.edges_directed(node, Direction::Outgoing));
            }
        }

        for edge in edges_to_add {
            let source = node_mapping[&edge.source()];
            let target = node_mapping[&edge.target()];
            resulting_graph.add_edge(source, target, edge.weight().clone());
        }

        SimpleCfgFunction {
            name: self.name.clone(),
            args: self.args.clone(),
            graph: resulting_graph,
            entry: node_mapping[&self.entry],
            exit: node_mapping[&self.exit],
            _phantom: Simple,
            return_ty: self.return_ty.clone(),
        }
    }

    // this function looks for all CFG empty blocks with a direct jump out
    // and makes sure they are removed by having the parents skip them
    fn collapse_empty_blocks(mut self) -> SimpleCfgFunction {
        let mut to_remove = vec![];
        for node in self.graph.node_indices().collect::<Vec<_>>() {
            // empty block with a single direct jump out
            if node != self.entry && self.graph[node].instrs.is_empty() {
                if let Some(single_child) = single_exit(&self.graph, node) {
                    let parents = self
                        .graph
                        .edges_directed(node, Direction::Incoming)
                        .map(|parent| {
                            let source = parent.source();
                            let weight = parent.weight().clone();
                            to_remove.push(parent.id());
                            (source, weight)
                        })
                        .collect::<Vec<_>>();

                    // for every parent edge, point to child instead of node
                    for (source, weight) in parents {
                        self.graph.add_edge(source, single_child, weight);
                    }
                }
            }
        }

        for edge in to_remove {
            self.graph.remove_edge(edge);
        }
        while self.graph[self.entry].instrs.is_empty() {
            if let Some(child) = single_exit(&self.graph, self.entry) {
                self.graph.remove_node(self.entry);
                self.entry = child;
            }
        }
        self
    }
}

fn single_exit(graph: &StableDiGraph<BasicBlock, Branch>, node: NodeIndex) -> Option<NodeIndex> {
    let mut iter = graph
        .edges_directed(node, Direction::Outgoing)
        .map(|edge| edge.target());
    let target = iter.next()?;
    if iter.next().is_some() || target == node {
        None
    } else {
        Some(target)
    }
}

impl SimpleCfgProgram {
    pub fn optimize_jumps(&self) -> Self {
        SimpleCfgProgram {
            functions: self
                .functions
                .iter()
                .map(|f| {
                    // NB: We could avoid this copy by having `optimize_jumps` take `self` by value.
                    let mut res = f.optimize_jumps();
                    res.simplify_branches();
                    res
                })
                .collect(),
        }
    }
}

#[test]
fn single_node() {
    // TODO these imports are very bad
    use crate::cfg::BlockName;
    use crate::test_util::*;

    let mut graph = StableDiGraph::new();
    let node = graph.add_node(BasicBlock {
        name: BlockName::Entry,
        instrs: vec![],
        footer: vec![],
        pos: None,
    });
    let input_cfg = SimpleCfgFunction {
        args: vec![],
        graph,
        entry: node,
        exit: node,
        name: "test".to_string(),
        _phantom: Simple,
        return_ty: None,
    };

    cfg_test_equiv!(input_cfg.optimize_jumps(), []);
}

#[test]
fn empty_entry_empty_exit() {
    // TODO these imports are very bad
    use crate::cfg::BlockName;
    use crate::test_util::*;

    let mut graph = StableDiGraph::new();
    let n1 = graph.add_node(BasicBlock {
        name: BlockName::Entry,
        instrs: vec![],
        footer: vec![],
        pos: None,
    });
    let n2 = graph.add_node(BasicBlock {
        name: BlockName::Exit,
        instrs: vec![],
        footer: vec![],
        pos: None,
    });

    graph.add_edge(
        n1,
        n2,
        Branch {
            op: crate::cfg::BranchOp::Jmp,
            pos: None,
        },
    );
    let input_cfg = SimpleCfgFunction {
        args: vec![],
        graph,
        entry: n1,
        exit: n2,
        name: "test".to_string(),
        _phantom: Simple,
        return_ty: None,
    };

    cfg_test_equiv!(input_cfg.optimize_jumps(), []);
}

#[test]
fn loops_to_self() {
    // TODO you have to import both of these
    //  is there a way to package up some macros all together?
    use crate::cfg::BlockName;
    use crate::test_util::*;
    let mut graph = StableDiGraph::new();
    let node = graph.add_node(BasicBlock {
        name: BlockName::Entry,
        instrs: vec![],
        footer: vec![],
        pos: None,
    });
    graph.add_edge(
        node,
        node,
        Branch {
            op: crate::cfg::BranchOp::Jmp,
            pos: None,
        },
    );

    let input_cfg = SimpleCfgFunction {
        args: vec![],
        graph,
        entry: node,
        exit: node,
        name: "test".to_string(),
        _phantom: Simple,
        return_ty: None,
    };

    cfg_test_equiv!(input_cfg.optimize_jumps(), [ENTRY = (Jmp)=> ENTRY,]);
}

#[test]
fn add_block_ind_test_snapshot() {
    let prog = include_str!("../../tests/passing/small/add_block_indirection.bril");
    let cfg = Optimizer::program_to_cfg(&Optimizer::parse_bril(prog).unwrap());
    insta::assert_snapshot!(cfg.optimize_jumps().to_bril().to_string());
}
