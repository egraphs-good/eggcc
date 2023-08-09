#![allow(dead_code)] // TODO: remove this once wired in
//! Parse a bril program into a CFG.
//!
//! The methods here largely ignore the instructions in the program: all that we
//! look for here are instructions that may break up basic blocks (`jmp`, `br`,
//! `ret`), and labels. All other instructions are copied into the CFG.
use std::mem;
use std::str::FromStr;
use std::{collections::HashMap, fmt::Display};

use bril_rs::{Argument, Code, EffectOps, Function, Instruction, Position, Program};
use petgraph::{
    graph::NodeIndex,
    visit::{DfsPostOrder, Walker},
    Graph,
};

use self::structured::StructuredProgram;

#[cfg(test)]
mod tests;

pub(crate) mod structured;
pub(crate) mod to_structured;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BlockName {
    Entry,
    Exit,
    Named(String),
}

impl Display for BlockName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockName::Entry => write!(f, "entry___"),
            BlockName::Exit => write!(f, "exit___"),
            BlockName::Named(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for BlockName {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "entry___" => Ok(BlockName::Entry),
            "exit___" => Ok(BlockName::Exit),
            s => Ok(BlockName::Named(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub(crate) instrs: Vec<Instruction>,
    pub(crate) name: BlockName,
    pub(crate) pos: Option<Position>,
}

impl BasicBlock {
    fn empty(name: BlockName) -> BasicBlock {
        BasicBlock {
            instrs: Default::default(),
            name,
            pos: None,
        }
    }

    fn to_code(&self) -> Vec<Code> {
        let mut instrs = Vec::new();
        instrs.push(Code::Label {
            label: self.name.to_string(),
            pos: self.pos.clone(),
        });
        instrs.extend(self.instrs.iter().map(|i| Code::Instruction(i.clone())));
        instrs
    }
}

/// A branch in the CFG.
#[derive(Debug, Clone)]
pub(crate) struct Branch {
    /// The type of branch.
    pub(crate) op: BranchOp,
    /// The position of the branch in the original program.
    pub(crate) pos: Option<Position>,
}

/// The types of branch.
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum BranchOp {
    /// An unconditional branch to a block.
    Jmp,
    /// A conditional branch to a block.
    Cond { arg: String, val: bool },
    /// A return statement carrying a value.
    RetVal { arg: String },
}

/// The control-flow graph for a single function.
#[derive(Debug)]
pub(crate) struct Cfg {
    /// The arguments to the function.
    pub(crate) args: Vec<Argument>,
    /// The graph itself.
    pub(crate) graph: Graph<BasicBlock, Branch>,
    /// The entry node for the CFG.
    pub(crate) entry: NodeIndex,
    /// The (single) exit node for the CFG.
    pub(crate) exit: NodeIndex,
    pub(crate) name: String,
}

impl Cfg {
    fn reverse_posorder(self: &Cfg) -> HashMap<BlockName, usize> {
        let mut reverse_postorder = HashMap::<BlockName, usize>::new();
        let mut post_counter = 0;
        DfsPostOrder::new(&self.graph, self.entry)
            .iter(&self.graph)
            .for_each(|node| {
                reverse_postorder.insert(self.graph[node].name.clone(), post_counter);
                post_counter += 1;
            });

        reverse_postorder
    }
}

pub(crate) fn program_to_structured(program: &Program) -> StructuredProgram {
    let mut functions = Vec::new();
    for func in &program.functions {
        let cfg = to_cfg(func);
        let structured = to_structured::to_structured(&cfg).unwrap();
        functions.push(structured);
    }
    StructuredProgram { functions }
}

/// Get the underyling CFG corresponding to the function `func`.
///
/// The structure is reproduced exactly, aside from the addition of a single
/// exit node branched to from all return statements.
pub(crate) fn to_cfg(func: &Function) -> Cfg {
    let mut builder = CfgBuilder::new(func);
    let mut block = Vec::new();
    let mut current = builder.cfg.entry;
    let mut had_branch = false;
    for inst in &func.instrs {
        match inst {
            Code::Label { label, pos } => {
                let next_block = builder.get_index(label);
                builder.finish_block(current, mem::take(&mut block));
                builder.set_pos(next_block, pos.clone());
                if !had_branch {
                    builder.add_edge(
                        current,
                        next_block,
                        Branch {
                            op: BranchOp::Jmp,
                            pos: pos.clone(),
                        },
                    );
                }
                current = next_block;
                had_branch = false;
            }
            Code::Instruction(Instruction::Effect {
                args,
                funcs: _,
                labels,
                op: EffectOps::Branch,
                pos,
            }) => {
                had_branch = true;
                assert_eq!(labels.len(), 2, "unexpected format to branch instruction");
                assert_eq!(args.len(), 1, "unexpected format to branch instruction");
                let true_block = builder.get_index(&labels[0]);
                let false_block = builder.get_index(&labels[1]);
                let arg = &args[0];
                builder.add_edge(
                    current,
                    true_block,
                    Branch {
                        op: BranchOp::Cond {
                            arg: arg.clone(),
                            val: true,
                        },
                        pos: pos.clone(),
                    },
                );
                builder.add_edge(
                    current,
                    false_block,
                    Branch {
                        op: BranchOp::Cond {
                            arg: arg.clone(),
                            val: false,
                        },
                        pos: pos.clone(),
                    },
                );
            }
            Code::Instruction(Instruction::Effect {
                args: _,
                funcs: _,
                labels,
                op: EffectOps::Jump,
                pos,
            }) => {
                had_branch = true;
                assert_eq!(labels.len(), 1, "unexpected format to jump instruction");
                let dest_block = builder.get_index(&labels[0]);
                builder.add_edge(
                    current,
                    dest_block,
                    Branch {
                        op: BranchOp::Jmp,
                        pos: pos.clone(),
                    },
                );
            }
            Code::Instruction(Instruction::Effect {
                args,
                funcs: _,
                labels: _,
                op: EffectOps::Return,
                pos,
            }) => {
                had_branch = true;
                match args.as_slice() {
                    [] => {
                        builder.add_edge(
                            current,
                            builder.cfg.exit,
                            Branch {
                                op: BranchOp::Jmp,
                                pos: pos.clone(),
                            },
                        );
                    }
                    [arg] => {
                        builder.add_edge(
                            current,
                            builder.cfg.exit,
                            Branch {
                                op: BranchOp::RetVal { arg: arg.clone() },
                                pos: pos.clone(),
                            },
                        );
                    }
                    _ => panic!("unexpected format to return instruction"),
                }
            }
            Code::Instruction(i) => block.push(i.clone()),
        }
    }
    builder.finish_block(current, mem::take(&mut block));
    builder.build()
}

struct CfgBuilder {
    cfg: Cfg,
    label_to_block: HashMap<String, NodeIndex>,
}

impl CfgBuilder {
    fn new(func: &Function) -> CfgBuilder {
        let mut graph = Graph::default();
        let entry = graph.add_node(BasicBlock::empty(BlockName::Entry));
        let exit = graph.add_node(BasicBlock::empty(BlockName::Exit));
        CfgBuilder {
            cfg: Cfg {
                args: func.args.clone(),
                graph,
                entry,
                exit,
                name: func.name.clone(),
            },
            label_to_block: HashMap::new(),
        }
    }
    fn build(mut self) -> Cfg {
        // If there are no outgoing edges from the entry block, add a basic one returning to the exit.
        if self
            .cfg
            .graph
            .neighbors_directed(self.cfg.entry, petgraph::Outgoing)
            .next()
            .is_none()
        {
            self.cfg.graph.add_edge(
                self.cfg.entry,
                self.cfg.exit,
                Branch {
                    op: BranchOp::Jmp,
                    pos: None,
                },
            );
        }
        self.cfg
    }
    fn get_index(&mut self, label: &str) -> NodeIndex {
        *self
            .label_to_block
            .entry(label.to_string())
            .or_insert_with(|| {
                self.cfg
                    .graph
                    .add_node(BasicBlock::empty(BlockName::Named(label.into())))
            })
    }
    fn finish_block(&mut self, index: NodeIndex, block: Vec<Instruction>) {
        let BasicBlock { instrs, .. } = self.cfg.graph.node_weight_mut(index).unwrap();
        debug_assert!(instrs.is_empty());
        *instrs = block;
    }

    fn set_pos(&mut self, index: NodeIndex, pos: Option<Position>) {
        self.cfg.graph.node_weight_mut(index).unwrap().pos = pos;
    }

    fn add_edge(&mut self, src: NodeIndex, dst: NodeIndex, branch: Branch) {
        self.cfg.graph.add_edge(src, dst, branch);
    }
}
