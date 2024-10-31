use std::iter;

use bril_rs::{Code, EffectOps, Function, Instruction, Program};
use indexmap::IndexSet;

use super::{Annotation, BasicBlock, BlockName, SimpleBranch, SimpleCfgFunction, SimpleCfgProgram};
use petgraph::{
    stable_graph::NodeIndex,
    visit::{Dfs, Walker},
};

impl SimpleCfgProgram {
    /// Converts the Cfg into a bril program.
    pub fn to_bril(&self) -> Program {
        let bril = Program {
            functions: self.functions.iter().map(|func| func.to_bril()).collect(),
            imports: vec![],
        };
        bril
    }
}

impl SimpleCfgFunction {
    pub(crate) fn label_name(block_name: &BlockName) -> String {
        format!("{}", block_name)
    }

    /// Converts the cfg function into a bril program.
    pub fn to_bril(&self) -> Function {
        // Make an empty function
        let mut func = Function {
            name: self.name.clone(),
            args: self.args.clone(),
            instrs: vec![],
            pos: None,
            return_type: self.return_ty.clone(),
        };

        // Pre-traverse the graph to get the order in which nodes will appear.
        // We'll use this to omit direct jumps to the block immediately
        // following the current one.
        //
        // Use DFS (preorder) to move adjacent blocks next to each other in the
        // bril output.
        let mut node_order = IndexSet::<&BlockName>::with_capacity(self.graph.node_count());
        for node in Dfs::new(&self.graph, self.entry)
            .iter(&self.graph)
            // don't do the exit or entry
            .filter(|node| node != &self.exit)
            .map(|node| &self.graph[node].name)
            .chain(iter::once(&self.graph[self.exit].name))
        {
            assert!(
                node_order.insert(node),
                "logic bug: DFS of graph visited node {node:?} twice"
            );
        }
        self.push_label(&mut func, self.entry);
        self.node_to_bril(self.entry, &mut func, &node_order);
        Dfs::new(&self.graph, self.entry)
            .iter(&self.graph)
            .filter(|node| node != &self.entry && node != &self.exit)
            .for_each(|node| {
                // Add a label for the block
                self.push_label(&mut func, node);
                // rest of the block
                self.node_to_bril(node, &mut func, &node_order);
            });

        // now do the exit at the end
        if self.exit != self.entry {
            self.push_label(&mut func, self.exit);

            self.node_to_bril(self.exit, &mut func, &node_order);
        }

        if func.instrs.is_empty() {
            // RVSDG conversions for empty functions do not add returns on their
            // own. The bril interpreter rejects functions without returns.
            func.instrs.push(Code::Instruction(Instruction::Effect {
                op: EffectOps::Return,
                args: vec![],
                funcs: vec![],
                labels: vec![],
                pos: None,
            }));
        }

        func
    }

    fn push_label(&self, func: &mut Function, node: NodeIndex) {
        func.instrs.push(Code::Label {
            label: Self::label_name(&self.graph[node].name),
            pos: None,
        });
    }

    // Converts a node to bril, including jumps at the end
    // Doesn't add the label for the node
    fn node_to_bril(
        &self,
        node: NodeIndex,
        func: &mut Function,
        node_order: &IndexSet<&BlockName>,
    ) {
        let block = &self.graph[node];
        let cur_index = node_order.get_index_of(&block.name).unwrap();
        let next_node = node_order.get_index(cur_index + 1).copied();
        func.instrs.extend(block.to_bril());

        // now add the jump to another block
        match self.get_branch(node) {
            SimpleBranch::NoBranch => {}
            SimpleBranch::Jmp(to) => {
                if Some(&to) == next_node {
                    // We will implicitly jump to the next block
                    return;
                }

                func.instrs.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Jump,
                    args: vec![],
                    labels: vec![Self::label_name(&to)],
                    funcs: vec![],
                    pos: None,
                }));
            }
            SimpleBranch::If {
                arg,
                then_branch,
                else_branch,
            } => {
                func.instrs.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Branch,
                    args: vec![arg.to_string()],
                    labels: vec![
                        Self::label_name(&then_branch),
                        Self::label_name(&else_branch),
                    ],
                    funcs: vec![],
                    pos: None,
                }));
            }
        }
    }
}

impl BasicBlock {
    // Converts this block to bril, not including jumps to other block
    // or edges.
    pub fn to_bril(&self) -> Vec<Code> {
        let mut res = vec![];
        for instr in &self.instrs {
            res.push(Code::Instruction(instr.clone()));
        }

        // CFGs only have annotations when they are restructured by RVSDG conversion
        // Here we can in_context they don't have any
        for annotation in &self.footer {
            match annotation {
                Annotation::AssignCond { .. } => {
                    panic!("No AssignCond annotations should be present for a Simple CFG")
                }
                Annotation::AssignRet { src } => {
                    res.push(Code::Instruction(Instruction::Effect {
                        op: EffectOps::Return,
                        args: vec![src.to_string()],
                        funcs: vec![],
                        labels: vec![],
                        pos: None,
                    }));
                }
            }
        }

        res
    }
}
