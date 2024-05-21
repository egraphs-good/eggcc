//! This module takes as input a [`SimpleCfgProgram`] and gets rid of any
//! direct jumps between blocks.
//! The block that is jumped to must have only one predecessor.
//! This is used by `to_cfg` to clean up
//! the output.

use hashbrown::HashMap;
use petgraph::{
    graph::EdgeIndex,
    stable_graph::{NodeIndex, StableDiGraph, StableGraph},
    visit::{Bfs, EdgeRef},
    Direction,
};

use crate::cfg::{BasicBlock, Branch, Simple, SimpleCfgFunction, SimpleCfgProgram};

#[cfg(test)]
use crate::Optimizer;

impl SimpleCfgFunction {
    pub fn optimize_jumps(&self) -> Self {
        self.single_in_single_out().collapse_empty_blocks()
    }

    fn single_in_single_out(&self) -> SimpleCfgFunction {
        let mut resulting_graph: StableGraph<BasicBlock, Branch> = StableDiGraph::new();

        // a map from nodes in the old graph to nodes in the
        // new graph
        // if a node was fused into another node,
        // it points to the new, fused node
        let mut node_mapping: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        // we use a bfs so that previous nodes are mapped to new nodes
        // before their children.
        // This ensures that `node_mapping[&previous]` succeeds.
        let mut bfs = Bfs::new(&self.graph, self.entry);

        let mut edges_to_add = vec![];

        // copy the graph without the edges
        // also choose which nodes get fused to which
        // by re-assigning in the node map
        while let Some(node) = bfs.next(&self.graph) {
            let mut collapse_node = false;
            let edges = self
                .graph
                .edges_directed(node, Direction::Incoming)
                .collect::<Vec<_>>();
            // single incoming edge to node
            if let &[single_edge] = edges.as_slice() {
                let previous = single_edge.source();
                let previous_outgoing = self
                    .graph
                    .edges_directed(previous, Direction::Outgoing)
                    .collect::<Vec<_>>();
                // single outgoing edge from previous
                // and two distinct nodes
                if previous_outgoing.len() == 1 && previous != node {
                    let previous_new = node_mapping[&previous];

                    // this node will be mapped to the previous
                    node_mapping.insert(node, previous_new);

                    // add instructions to the end of the previous node
                    resulting_graph[previous_new]
                        .instrs
                        .extend(self.graph[node].instrs.to_vec());
                    resulting_graph[previous_new]
                        .footer
                        .extend(self.graph[node].footer.to_vec());

                    collapse_node = true;
                }
            }

            if !collapse_node {
                // add the node
                let new_node = resulting_graph.add_node(self.graph[node].clone());
                node_mapping.insert(node, new_node);

                edges_to_add.extend(self.graph.edges_directed(node, Direction::Incoming));
            }
        }

        for edge in edges_to_add {
            let source = node_mapping[&edge.source()];
            let target = node_mapping[&edge.target()];
            resulting_graph.add_edge(source, target, edge.weight().clone());
        }

        let mut label_mapping = HashMap::<String, String>::new();
        for (old, new) in node_mapping.iter() {
            let old_entry = label_mapping.insert(
                self.graph[*old].name.to_string(),
                resulting_graph[*new].name.to_string(),
            );
            assert!(old_entry.is_none(), "Duplicate labels in graph");
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

    // this function looks for all CFG fragments of the form
    // source node --(source edge)-> empty node --(target edge)-> target node
    // (where there is only one target edge and the empty node has no instructions)
    // and changes the source edge to point to the target node
    fn collapse_empty_blocks(mut self) -> SimpleCfgFunction {
        // loop over every non-empty block, and look at its parents
        for block in self.graph.node_indices().collect::<Vec<_>>() {
            // find parents
            let parents: Vec<NodeIndex> = self
                .graph
                .edges_directed(block, Direction::Incoming)
                .map(|edge| edge.source())
                .collect();

            for parent in &parents {
                self.collapse_empty_block(*parent);
            }
        }

        self
    }

    /// Detect the case where source -> empty -> target
    /// The empty block should have a single incoming and single outgoing edge
    fn get_single_in_single_out(&self, parent_block: NodeIndex) -> Option<(EdgeIndex, EdgeIndex)> {
        if !self.graph[parent_block].instrs.is_empty()
            || !self.graph[parent_block].footer.is_empty()
        {
            return None;
        }

        let outgoing = self
            .graph
            .edges_directed(parent_block, Direction::Outgoing)
            .collect::<Vec<_>>();
        let incoming = self
            .graph
            .edges_directed(parent_block, Direction::Incoming)
            .collect::<Vec<_>>();
        if let ([source_edge], [outgoing]) = (incoming.as_slice(), outgoing.as_slice()) {
            Some((source_edge.id(), outgoing.id()))
        } else {
            None
        }
    }

    /// Detect the case when source -> empty -> target
    /// and collapse it to source -> target
    fn collapse_empty_block(&mut self, parent_block: NodeIndex) {
        if let Some((source_edge, outgoing)) = self.get_single_in_single_out(parent_block) {
            let weight = self.graph.edge_weight(source_edge).unwrap().clone();
            let (source, empty) = self.graph.edge_endpoints(source_edge).unwrap();
            let (empty_, target) = self.graph.edge_endpoints(outgoing).unwrap();
            assert_eq!(empty, empty_);

            self.graph.remove_edge(source_edge);
            self.graph.add_edge(source, target, weight);
        }
    }
}

impl SimpleCfgProgram {
    pub fn optimize_jumps(&self) -> Self {
        SimpleCfgProgram {
            functions: self.functions.iter().map(|f| f.optimize_jumps()).collect(),
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
fn add_block_ind_test() {
    let prog = include_str!("../../tests/passing/small/add_block_indirection.bril");
    let cfg = Optimizer::program_to_cfg(&Optimizer::parse_bril(prog).unwrap());
    insta::assert_snapshot!(cfg.optimize_jumps().to_bril().to_string());
}
