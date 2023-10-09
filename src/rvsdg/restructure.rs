//! Convert a potentially irreducible CFG to a reducible one.

use hashbrown::{HashMap, HashSet};
use petgraph::{
    algo::{dominators, tarjan_scc},
    dot::Dot,
    graph::NodeIndex,
    stable_graph::EdgeIndex,
    visit::{EdgeRef, NodeFiltered, VisitMap},
    Direction,
};

use crate::cfg::{
    Annotation, BasicBlock, BlockName, Branch, BranchOp, CondVal, Identifier, NodeSet,
    SwitchCfgFunction,
};

fn node_set(nodes: impl IntoIterator<Item = NodeIndex>) -> NodeSet {
    let mut set = NodeSet::default();
    for node in nodes {
        if set.len() <= node.index() {
            set.grow(node.index() + 1);
        }
        set.visit(node);
    }
    set
}

struct RestructureState {
    n_names: usize,
}

impl RestructureState {
    fn fresh(&mut self) -> Identifier {
        let n = self.n_names;
        self.n_names += 1;
        Identifier::Num(n)
    }
}

impl SwitchCfgFunction {
    fn fresh_block(&mut self) -> NodeIndex {
        let placeholder = self.graph.node_count();
        self.graph
            .add_node(BasicBlock::empty(BlockName::Placeholder(placeholder)))
    }

    pub(crate) fn restructure(&mut self) {
        let mut state = RestructureState { n_names: 0 };
        let mut all = NodeSet::with_capacity(self.graph.node_count());
        self.graph.node_indices().for_each(|node| {
            all.visit(node);
        });
        self.restructure_loops(&all, &mut state);

        eprintln!("After restructuring loops: {:#?}", Dot::new(&self.graph));
        self.restructure_branches(&mut state);
    }

    fn branch_if(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        id: &Identifier,
        cv: CondVal,
    ) -> EdgeIndex {
        self.graph.add_edge(
            from,
            to,
            Branch {
                op: BranchOp::Cond {
                    arg: id.clone(),
                    val: cv,
                },
                pos: None,
            },
        )
    }

    fn rewrite_arcs(
        &mut self,
        arcs: impl Iterator<Item = EdgeIndex>,
        mut annotate: impl FnMut(NodeIndex, &mut Vec<Annotation>) -> NodeIndex,
    ) {
        for edge_id in arcs {
            let (src, target) = self.graph.edge_endpoints(edge_id).unwrap();
            let branch = self.graph.remove_edge(edge_id).unwrap();
            match &branch.op {
                BranchOp::Jmp
                | BranchOp::Cond {
                    val: CondVal { val: 0, of: 1 },
                    ..
                } => {
                    // For unconditional jumps, simply annotate the
                    // source block and reroute the edge
                    let new_target = annotate(target, &mut self.graph[src].footer);
                    self.graph.add_edge(src, new_target, branch);
                }
                BranchOp::Cond { .. } => {
                    // For conditional jumps, create a new node and
                    // route flow through it before jumping to the entry
                    // node.
                    let intermediate = self.fresh_block();
                    let new_target = annotate(target, &mut self.graph[intermediate].footer);
                    self.graph.add_edge(src, intermediate, branch);
                    self.graph.add_edge(intermediate, new_target, JMP);
                }
            }
        }
    }

    fn restructure_loops(&mut self, filter: &NodeSet, state: &mut RestructureState) {
        let base = NodeFiltered::from_fn(&self.graph, |node| filter.is_visited(&node));
        let sccs = tarjan_scc(&base);
        for scc in sccs {
            if scc.len() < 2 {
                // Any SCC with this few nodes must be structured.
                continue;
            }

            // The following follows the paper fairly literally.

            let scc_set = node_set(scc.iter().copied());
            let mut entry_arcs = HashSet::new();
            let mut entry_vertices = HashSet::new();
            for edge_ref in scc
                .iter()
                .flat_map(|node| self.graph.edges_directed(*node, Direction::Incoming))
                .filter(|e| !scc_set.is_visited(&e.source()))
            {
                entry_arcs.insert(edge_ref.id());
                entry_vertices.insert(edge_ref.target());
            }

            let mut exit_arcs = HashSet::new();
            let mut exit_vertices = HashSet::new();

            for edge_ref in scc
                .iter()
                .flat_map(|node| self.graph.edges_directed(*node, Direction::Outgoing))
                .filter(|e| !scc_set.is_visited(&e.target()))
            {
                exit_arcs.insert(edge_ref.id());
                exit_vertices.insert(edge_ref.target());
            }

            let repetition_arcs: HashSet<EdgeIndex> = entry_vertices
                .iter()
                .flat_map(|node| self.graph.edges_directed(*node, Direction::Incoming))
                .filter(|e| scc_set.is_visited(&e.source()))
                .map(|e| e.id())
                .collect();

            let entry_node = if entry_vertices.len() == 1 {
                *entry_vertices.iter().next().unwrap()
            } else {
                self.fresh_block()
            };

            let exit_node = if exit_vertices.len() == 1 {
                *exit_vertices.iter().next().unwrap()
            } else {
                self.fresh_block()
            };

            let rep_id = state.fresh();
            let loop_tail = if !repetition_arcs.is_empty() {
                let tail_node = self.fresh_block();
                self.branch_if(tail_node, exit_node, &rep_id, CondVal { val: 0, of: 2 });
                self.branch_if(tail_node, entry_node, &rep_id, CondVal { val: 1, of: 2 });
                tail_node
            } else {
                exit_node
            };

            // demux through the entry block.
            let (block_map, cond_id) =
                self.make_demux_node(entry_node, entry_vertices.iter().copied(), state);

            self.rewrite_arcs(entry_arcs.iter().copied(), |target, anns| {
                anns.push(Annotation::AssignCond {
                    dst: cond_id.clone(),
                    cond: block_map[&target],
                });
                entry_node
            });

            self.rewrite_arcs(repetition_arcs.iter().copied(), |target, anns| {
                anns.push(Annotation::AssignCond {
                    dst: cond_id.clone(),
                    cond: block_map[&target],
                });
                anns.push(Annotation::AssignCond {
                    dst: rep_id.clone(),
                    cond: 1,
                });
                loop_tail
            });

            // Now demux exit paths through the exit node:

            let (block_map, cond_id) =
                self.make_demux_node(exit_node, exit_vertices.iter().copied(), state);

            self.rewrite_arcs(exit_arcs.iter().copied(), |target, anns| {
                anns.push(Annotation::AssignCond {
                    dst: cond_id.clone(),
                    cond: block_map[&target],
                });
                anns.push(Annotation::AssignCond {
                    dst: rep_id.clone(),
                    cond: 0,
                });
                loop_tail
            });

            if exit_arcs.is_empty() {
                // Infinite loop
                self.graph.remove_node(exit_node);
            }

            // Recursively restructure the inner loop.
            self.restructure_loops(&scc_set, state);
        }
    }

    fn split_arc(&mut self, edge: EdgeIndex) -> NodeIndex {
        let (src, dst) = self.graph.edge_endpoints(edge).unwrap();
        let middle = self.fresh_block();
        let weight = self.graph.remove_edge(edge).unwrap();
        self.graph.add_edge(src, middle, weight);
        self.graph.add_edge(
            middle,
            dst,
            Branch {
                op: BranchOp::Jmp,
                pos: None,
            },
        );
        middle
    }

    fn split_edges(&mut self, node: NodeIndex) {
        let mut has_jmp = false;
        let mut walker = self
            .graph
            .neighbors_directed(node, Direction::Outgoing)
            .detach();
        while let Some((edge, other)) = walker.next(&self.graph) {
            assert!(!has_jmp);
            match &self.graph.edge_weight(edge).unwrap().op {
                BranchOp::Jmp
                | BranchOp::Cond {
                    val: CondVal { of: 1, .. },
                    ..
                } => {
                    has_jmp = true;
                    continue;
                }
                BranchOp::Cond { .. } => {}
            }

            // We have a conditional branch. Reroute through a placeholder.
            let weight = self.graph.remove_edge(edge).unwrap();
            let placeholder = self.fresh_block();

            // We had  node => other
            // We want node => placeholder => other
            assert_ne!(other, self.entry);
            self.graph.add_edge(node, placeholder, weight);
            self.graph.add_edge(placeholder, other, JMP);
        }
    }

    fn make_demux_node(
        &mut self,
        node: NodeIndex,
        targets: impl IntoIterator<Item = NodeIndex>,
        state: &mut RestructureState,
    ) -> (HashMap<NodeIndex, u32>, Identifier) {
        let mut blocks = HashMap::new();
        for node in targets {
            let cur_len = u32::try_from(blocks.len()).unwrap();
            blocks.entry(node).or_insert(cur_len);
        }
        let cond = state.fresh();

        let n_blocks = u32::try_from(blocks.len()).unwrap();
        for (block, val) in blocks.iter() {
            if *block == node {
                continue;
            }
            self.branch_if(
                node,
                *block,
                &cond,
                CondVal {
                    val: *val,
                    of: n_blocks,
                },
            );
        }
        (blocks, cond)
    }

    fn restructure_branches(&mut self, state: &mut RestructureState) {
        let please_use_nodeset_instead = 1;
        // Credit to optir for structuring the loop in this way; this is pretty different than the paper.
        let dom = dominators::simple_fast(&self.graph, self.entry);
        let dominates = |x: NodeIndex, y| {
            dom.dominators(y)
                .map(|mut ds| ds.any(|d| x == d))
                .unwrap_or(false)
        };

        // The "Perfect Reconstructability" paper uses "continuation point" to
        // refer to the targets of a branch _not_ part of a structured region.
        //
        // We want to group these continuations by their immediate dominators
        // (called the "Head" in the paper), then add a mux node in front of the
        // continuations if there is more than one.

        let mut tail_continuations = HashMap::<NodeIndex, Vec<NodeIndex>>::new();

        for ix in self.graph.node_indices() {
            if let Some(idom) = dom.immediate_dominator(ix) {
                // Continuations have more than one non-loop incoming edge
                if self
                    .graph
                    .neighbors_directed(ix, Direction::Incoming)
                    .filter(|pred| !dominates(ix, *pred))
                    .nth(1)
                    .is_some()
                {
                    tail_continuations.entry(idom).or_default().push(ix);
                }
            }
        }

        for (idom, conts) in tail_continuations.into_iter() {
            // Split any direct edges from the idom to the continuation. If we
            // have more than one continuation then split _all_ edges.
            for cont in &conts {
                let mut walker = self
                    .graph
                    .neighbors_directed(*cont, Direction::Incoming)
                    .detach();
                while let Some((e, src)) = walker.next(&self.graph) {
                    if src == idom && conts.len() == 1 {
                        self.split_arc(e);
                    } else if conts.len() > 1 {
                        self.split_edges(src);
                    }
                }
            }

            // The rest of this loop is focused on mux/demux for multiple
            // continuations. If we only have one, we are done.
            if conts.len() == 1 {
                continue;
            }
            let mux = self.fresh_block();
            let (preds, cond_var) = self.make_demux_node(mux, conts.iter().copied(), state);

            for cont in &conts {
                let mut walker = self
                    .graph
                    .neighbors_directed(*cont, Direction::Incoming)
                    .detach();

                // NB: there's some extra filtering that happens here in optir, do we need it?
                while let Some((edge, src)) = walker.next(&self.graph) {
                    if src == mux {
                        continue;
                    }
                    if conts.iter().any(|&c| c == src) {
                        continue;
                    }
                    let branch = self.graph.remove_edge(edge).unwrap();
                    self.graph.add_edge(src, mux, branch);
                    self.graph[src].footer.push(Annotation::AssignCond {
                        dst: cond_var.clone(),
                        cond: preds[cont],
                    });
                }
            }
        }
    }
}

const JMP: Branch = Branch {
    op: BranchOp::Jmp,
    pos: None,
};
