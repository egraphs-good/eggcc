use bril_rs::{Code, EffectOps, Function, Instruction, Program};

use super::{Annotation, BasicBlock, SimpleBranch, SimpleCfgFunction, SimpleCfgProgram};
use petgraph::{
    stable_graph::NodeIndex,
    visit::{DfsPostOrder, Walker},
};

impl SimpleCfgProgram {
    /// Converts the Cfg into a bril program.
    pub fn to_bril(&self) -> Program {
        let mut bril = Program {
            functions: vec![],
            imports: vec![],
        };
        for func in &self.functions {
            bril.functions.push(func.to_bril());
        }
        bril
    }
}

impl SimpleCfgFunction {
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

        // start with the entry block
        self.node_to_bril(self.entry, &mut func);

        // The order of this traversal does not matter, just need to loop over the blocks
        DfsPostOrder::new(&self.graph, self.entry)
            .iter(&self.graph)
            .for_each(|node| {
                // don't do the exit or entry
                if node != self.entry && node != self.exit {
                    // Add a label for the block
                    func.instrs.push(Code::Label {
                        label: format!("{}", self.graph[node].name),
                        pos: None,
                    });
                    // rest of the block
                    self.node_to_bril(node, &mut func);
                }
            });

        // now do the exit at the end
        func.instrs.push(Code::Label {
            label: format!("{}", self.graph[self.exit].name),
            pos: None,
        });
        self.node_to_bril(self.exit, &mut func);

        func
    }

    // Converts a node to bril, including jumps at the end
    // Doesn't add the label for the node
    fn node_to_bril(&self, node: NodeIndex, func: &mut Function) {
        let block = &self.graph[node];
        func.instrs.extend(block.to_bril());

        // now add the jump to another block
        match self.get_branch(node) {
            SimpleBranch::NoBranch => {}
            SimpleBranch::Jmp(to) => {
                func.instrs.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Jump,
                    args: vec![],
                    labels: vec![format!("{}", to)],
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
                    labels: vec![format!("{}", then_branch), format!("{}", else_branch)],
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
        // Here we can assume they don't have any
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
