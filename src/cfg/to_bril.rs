use bril_rs::{Code, EffectOps, Function, Instruction, Program};

use super::{BasicBlock, SimpleBranch, SimpleCfgFunction, SimpleCfgProgram};
use petgraph::visit::{DfsPostOrder, Walker};

impl SimpleCfgProgram {
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
    pub fn to_bril(&self) -> Function {
        let mut bril = Function {
            name: self.name.clone(),
            args: self.args.clone(),
            instrs: vec![],
            pos: None,
            return_type: self.return_ty.clone(),
        };

        // start with the entry block

        DfsPostOrder::new(&self.graph, self.entry)
            .iter(&self.graph)
            .for_each(|node| {
                // if this is not the start block, add a label for it
                if node != self.entry {
                    bril.instrs.push(Code::Label {
                        label: format!("{}", self.graph[node].name),
                        pos: None,
                    });
                }

                let block = &self.graph[node];
                bril.instrs.extend(block.to_bril());

                // now add the jump to another block
                match self.get_branch(node) {
                    SimpleBranch::NoBranch => {}
                    SimpleBranch::Jmp(to) => {
                        bril.instrs.push(Code::Instruction(Instruction::Effect {
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
                        bril.instrs.push(Code::Instruction(Instruction::Effect {
                            op: EffectOps::Branch,
                            args: vec![arg.to_string()],
                            labels: vec![format!("{}", then_branch), format!("{}", else_branch)],
                            funcs: vec![],
                            pos: None,
                        }));
                    }
                }
            });

        bril
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
        assert!(self.footer.is_empty());
        res
    }
}
