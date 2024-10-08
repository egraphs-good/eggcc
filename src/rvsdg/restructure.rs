//! Convert a potentially irreducible CFG to a reducible one.
//! Important resources for the implementation here are Optir[^1], and the relevant RVSDG paper[^2].
//!
//! [^1]: <https://github.com/jameysharp/optir>
//!
//! [^2]: ["Perfect Reconstructability of Control Flow from Demand Dependence
//! Graphs"](https://dl.acm.org/doi/10.1145/2693261). See also the accompanying
//! jlm repo.
use std::collections::VecDeque;

use bril_rs::Type;
use indexmap::{IndexMap, IndexSet};
use petgraph::{
    algo::{dominators, tarjan_scc},
    graph::NodeIndex,
    stable_graph::EdgeIndex,
    visit::{EdgeRef, IntoEdgeReferences, NodeFiltered, VisitMap},
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

        self.restructure_branches(&mut state);
    }

    /// Using a boolean predicate,
    /// add a branch to the graph that jumps from `from` and to
    /// `to` when the predicate has value `cv`.
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
                    bril_type: Type::Bool,
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
            let mut entry_arcs = IndexSet::new();
            let mut entry_vertices = IndexSet::new();
            for edge_ref in scc
                .iter()
                .flat_map(|node| self.graph.edges_directed(*node, Direction::Incoming))
                .filter(|e| !scc_set.is_visited(&e.source()))
            {
                entry_arcs.insert(edge_ref.id());
                entry_vertices.insert(edge_ref.target());
            }

            let mut exit_arcs = IndexSet::new();
            let mut exit_vertices = IndexSet::new();

            for edge_ref in scc
                .iter()
                .flat_map(|node| self.graph.edges_directed(*node, Direction::Outgoing))
                .filter(|e| !scc_set.is_visited(&e.target()))
            {
                exit_arcs.insert(edge_ref.id());
                exit_vertices.insert(edge_ref.target());
            }

            let repetition_arcs: IndexSet<EdgeIndex> = entry_vertices
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

    fn make_demux_node(
        &mut self,
        node: NodeIndex,
        targets: impl IntoIterator<Item = NodeIndex>,
        state: &mut RestructureState,
    ) -> (IndexMap<NodeIndex, u32>, Identifier) {
        let mut blocks = IndexMap::default();
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

    /// Compute the subgraph of the CFG dominated by the given edge.
    fn dominator_graph(&self, edge: EdgeIndex) -> IndexSet<NodeIndex> {
        let mut nodes = IndexSet::default();
        let mut edges = IndexSet::new();
        edges.insert(edge);
        let mut frontier = VecDeque::with_capacity(1);
        let (_, target) = self.graph.edge_endpoints(edge).unwrap();
        frontier.push_back(target);
        while let Some(node) = frontier.pop_front() {
            if nodes.contains(&node) {
                continue;
            }
            let all_known = self
                .graph
                .edges_directed(node, Direction::Incoming)
                .all(|e| edges.contains(&e.id()));
            if all_known {
                nodes.insert(node);
                for edge_ref in self.graph.edges_directed(node, Direction::Outgoing) {
                    edges.insert(edge_ref.id());
                    frontier.push_back(edge_ref.target());
                }
            }
        }
        nodes
    }

    fn get_continuation(&self, branch: NodeIndex) -> Continuation {
        let mut cont = Continuation::default();
        for (edge_ref, dgraph) in self
            .graph
            .edges_directed(branch, Direction::Outgoing)
            .map(|edge_ref| (edge_ref, self.dominator_graph(edge_ref.id())))
        {
            if dgraph.is_empty() {
                cont.exit_arcs
                    .entry(edge_ref.id())
                    .or_default()
                    .insert(edge_ref.id());
                cont.reentry_nodes.insert(edge_ref.target());
                continue;
            }

            for node in &dgraph {
                for exit_edge in self
                    .graph
                    .edges_directed(*node, Direction::Outgoing)
                    .filter(|x| !dgraph.contains(&x.target()))
                {
                    cont.exit_arcs
                        .entry(edge_ref.id())
                        .or_default()
                        .insert(exit_edge.id());
                    cont.reentry_nodes.insert(exit_edge.target());
                }
            }
        }
        cont
    }
    fn traverse_linear_region(&self, mut cur: NodeIndex, exit: NodeIndex) -> NodeIndex {
        while cur != exit {
            let mut neighbors = self.graph.neighbors_directed(cur, Direction::Outgoing);
            let Some(next) = neighbors.next() else {
                break;
            };
            if neighbors.next().is_some() {
                break;
            }
            cur = next;
        }
        cur
    }
    /// Reassigns an edge to a new target destination, but the same source.
    fn retarget(&mut self, edge: EdgeIndex, new_target: NodeIndex) -> EdgeIndex {
        let (src, target) = self.graph.edge_endpoints(edge).unwrap();
        if target == new_target {
            return edge;
        }
        let weight = self.graph.remove_edge(edge).unwrap();
        self.graph.add_edge(src, new_target, weight)
    }
    fn restructure_branches_inner(
        &mut self,
        entry: NodeIndex,
        exit: NodeIndex,
        state: &mut RestructureState,
    ) {
        // Use the original algorithm (RVSDG Paper, including the accompanying JLM code).
        let start = self.traverse_linear_region(entry, exit);
        if start == exit {
            return;
        }
        let cont = self.get_continuation(start);

        if cont.reentry_nodes.is_empty() {
            // Nothing to do.
            return;
        }

        // First, some special cases that allow us to avoid creating auxiliary
        // predicates / nodes when the CFG already has the desired structure.
        if cont.reentry_nodes.len() == 1 {
            // There are multiple branches that all converge to a single "tail"
            // node. This is _almost_ the structure that we want, but we need to
            // add empty placeholder blocks and a few other things.
            let tail = *cont.reentry_nodes.iter().next().unwrap();
            for (edge, target) in self
                .graph
                .edges_directed(start, Direction::Outgoing)
                .map(|x| (x.id(), x.target()))
                // We make recursive calls to this method in this loop, so need
                // to copy edge information out in this frame.
                .collect::<Vec<_>>()
            {
                // This edge goes directly to the tail. Create an empty basic
                // block (this will make our CFG look like a diamond rather than
                // a triangle, which in turn makes it easier for us to generate
                // a 'passthrough region' for this branch in the RVSDG).
                if target == tail {
                    self.split_arc(edge);
                    continue;
                }

                let reentry_edges = &cont.exit_arcs[&edge];
                if reentry_edges.len() == 1 {
                    // We don't go directly to the tail, but there is at least a
                    // single exit from the subgraph pointed to by `edge`.
                    //
                    // Restructure this subgraph (recursively).
                    let next_edge = *reentry_edges.iter().next().unwrap();
                    debug_assert_ne!(next_edge, edge);
                    let branch_end = self.graph.edge_endpoints(next_edge).unwrap().0;
                    self.restructure_branches_inner(target, branch_end, state);
                    continue;
                }
                // There are multiple exit edges to the tail. Have them all join
                // into an intermediate block.
                let inter = self.fresh_block();
                self.graph.add_edge(inter, tail, JMP);
                for e in reentry_edges {
                    self.retarget(*e, inter);
                }
                self.restructure_branches_inner(target, inter, state);
            }
            self.restructure_branches_inner(tail, exit, state);
        } else {
            // The general case:
            // We have multiple potential continuation points. Create a new node
            // and a variable to demux them.
            let demux = self.fresh_block();
            let (conds, pred) =
                self.make_demux_node(demux, cont.reentry_nodes.iter().copied(), state);
            for (edge, target) in self
                .graph
                .edges_directed(start, Direction::Outgoing)
                .map(|x| (x.id(), x.target()))
                .collect::<Vec<_>>()
            {
                // Fan all outgoing edges from this branch subgraph into an
                // intermediate node.
                //
                // Then ahead of this branch, assign `pred` to the correct
                // value.
                let reentry_edges = &cont.exit_arcs[&edge];
                let inter = self.fresh_block();
                self.graph.add_edge(inter, demux, JMP);
                for edge in reentry_edges {
                    let (src, target) = self.graph.edge_endpoints(*edge).unwrap();
                    self.graph[src].footer.push(Annotation::AssignCond {
                        dst: pred.clone(),
                        cond: conds[&target],
                    });
                    self.retarget(*edge, inter);
                }
                self.restructure_branches_inner(target, inter, state);
            }
            self.restructure_branches_inner(demux, exit, state);
        }
    }

    fn extract_edge(&mut self, edge: EdgeIndex) -> EdgeData {
        let (src, dst) = self.graph.edge_endpoints(edge).unwrap();
        let branch = self.graph.remove_edge(edge).unwrap();
        EdgeData { src, dst, branch }
    }

    fn insert_edge(&mut self, data: EdgeData) {
        self.graph.add_edge(data.src, data.dst, data.branch);
    }

    fn remove_cycles(&mut self) -> Vec<EdgeData> {
        let dom = dominators::simple_fast(&self.graph, self.entry);
        let dominates = |x: NodeIndex, y| {
            dom.dominators(y)
                .map(|mut ds| ds.any(|d| x == d))
                .unwrap_or(false)
        };

        let to_remove: Vec<_> = self
            .graph
            .edge_references()
            .filter_map(|edge| {
                if dominates(edge.target(), edge.source()) {
                    Some(edge.id())
                } else {
                    None
                }
            })
            .collect();
        to_remove
            .into_iter()
            .map(|x| self.extract_edge(x))
            .collect()
    }

    fn restructure_branches(&mut self, state: &mut RestructureState) {
        let tails = self.remove_cycles();
        self.restructure_branches_inner(self.entry, self.exit, state);
        tails.into_iter().for_each(|e| self.insert_edge(e))
    }
}

const JMP: Branch = Branch {
    op: BranchOp::Jmp,
    pos: None,
};

/// A "Continuation" (in the parlance of the "Perfect Reconstructability" paper)
/// is the set of points where a branch subgraph rejoins the main flow of control.
///
/// The game for branch restructuring is to take a potentially unstructured,
/// acylclic CFG and translate it into one that is structured. The algorithm
/// traverses from the entry node to a CFG and waits until it finds a branch:
/// ```ignore
///    [ b1 ] --> ...
///   /
///  * -- [ b2 ] --> ...
///   \
///    [ b3 ] --> ...
/// ```
/// To decompose this graph into something useful, we compute the (potentially
/// empty!) subgraphs dominated by each edge. We eventually need these subgraphs
/// to join back at some "tail" node:
///
/// ```ignore
///    [ b1 ] --- B1 ----*
///   /                    \?
///  * -- [ b2 ] --- B2 ---? T
///   \                    /?
///    [ b3 ] --- B3 ----*
/// ```
///
/// These `Bi` subgraphs need to meet back at `T`, but initially they may
/// have quite a lot of outgoing edges, rather than just one. Continuations
/// track these edges/nodes so that we can clean them up before recurring on
/// each `Bi` and `T`.
#[derive(Default)]
struct Continuation {
    /// Nodes in the "tail" (`T` above) that are targetted by an edge out of the
    /// given branch node.
    reentry_nodes: IndexSet<NodeIndex>,
    /// A mapping from branch edge, to edges back to nodes not dominated by that edge.
    exit_arcs: IndexMap<EdgeIndex, IndexSet<EdgeIndex>>,
}

struct EdgeData {
    src: NodeIndex,
    dst: NodeIndex,
    branch: Branch,
}
