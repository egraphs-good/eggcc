//! Parse a bril program into a CFG.
//!
//! The methods here largely ignore the instructions in the program: all that we
//! look for here are instructions that may break up basic blocks (`jmp`, `br`,
//! `ret`), and labels. All other instructions are copied into the CFG.
use std::str::FromStr;
use std::{collections::HashMap, fmt::Display};
use std::{fmt, mem};

use bril_rs::{Argument, Code, EffectOps, Function, Instruction, Position, Program, Type};
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::Visitable;
use petgraph::{
    graph::NodeIndex,
    visit::{DfsPostOrder, Walker},
};

/// A subset of nodes for a particular CFG.
pub(crate) type NodeSet = <StableDiGraph<BasicBlock, Branch> as Visitable>::Map;

#[cfg(test)]
mod tests;

pub(crate) mod structured;
pub(crate) mod to_structured;

/// Convert a program to a cfg.
/// Loops over all the functions, translating individually.
pub(crate) fn program_to_cfg(program: &Program) -> CfgProgram {
    let mut functions = Vec::new();
    for func in &program.functions {
        let cfg = to_cfg(func);
        functions.push(cfg);
    }
    CfgProgram { functions }
}

#[derive(Clone)]
pub struct CfgProgram {
    pub functions: Vec<Cfg>,
}

/// The name (or label) associated with a basic block.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BlockName {
    /// The unique entrypoint for a function.
    Entry,
    /// The unique exit point for a function (assuming the function is not an infinite loop).
    Exit,
    /// An unnamed block generated as part of the restructuring process.
    Placeholder(usize),
    /// A named block from the original Bril program.
    Named(String),
}

/// The distinguished identifier associated with the return value of a function,
/// if it has one.
pub(crate) fn ret_id() -> Identifier {
    Identifier::Num(!0)
}

impl Display for BlockName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockName::Entry => write!(f, "entry___"),
            BlockName::Exit => write!(f, "exit___"),
            BlockName::Placeholder(n) => write!(f, "__{n}__"),
            BlockName::Named(s) => write!(f, "{}", s),
        }
    }
}

/// An number (`val`) between 0 and `of` (exclusive).
///
/// These are used to represent "switch" blocks in the CFG.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CondVal {
    pub(crate) val: u32,
    pub(crate) of: u32,
}

impl From<bool> for CondVal {
    fn from(value: bool) -> Self {
        if value {
            CondVal { val: 1, of: 2 }
        } else {
            CondVal { val: 0, of: 2 }
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

/// Identifiers either come from the source Bril program or are synthesized as
/// part of the RVSDG conversion process. The `Identifier` type stores both
/// kinds of name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Identifier {
    Name(String),
    Num(usize),
}

impl<T: AsRef<str>> From<T> for Identifier {
    fn from(value: T) -> Identifier {
        Identifier::Name(value.as_ref().into())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Name(n) => {
                write!(f, "{n}")
            }
            Identifier::Num(n) => {
                write!(f, "@{n}")
            }
        }
    }
}

/// An annotation appended to the end of a basic block.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Annotation {
    AssignCond { dst: Identifier, cond: u32 },
    AssignRet { src: Identifier },
}

/// A branch-free sequence of instructions.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    /// The primary instructions for a block.
    pub(crate) instrs: Vec<Instruction>,
    /// Any annotations added to the end of the block during restructuring.
    pub(crate) footer: Vec<Annotation>,
    /// The name for the block.
    pub(crate) name: BlockName,
    pub(crate) pos: Option<Position>,
}

impl BasicBlock {
    pub(crate) fn empty(name: BlockName) -> BasicBlock {
        BasicBlock {
            instrs: Default::default(),
            footer: Default::default(),
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
    #[allow(unused)]
    pub(crate) pos: Option<Position>,
}

/// The types of branch.
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum BranchOp {
    /// An unconditional branch to a block.
    Jmp,
    /// A conditional branch to a block.
    Cond { arg: Identifier, val: CondVal },
}

/// The control-flow graph for a single function.
#[derive(Debug, Clone)]
pub struct Cfg {
    /// The arguments to the function.
    pub(crate) args: Vec<Argument>,
    /// The graph itself.
    pub(crate) graph: StableDiGraph<BasicBlock, Branch>,
    /// The entry node for the CFG.
    pub(crate) entry: NodeIndex,
    /// The (single) exit node for the CFG.
    pub(crate) exit: NodeIndex,
    /// The name of the function.
    pub(crate) name: String,
    return_ty: Option<Type>,
}

impl Cfg {
    pub(crate) fn has_return_value(&self) -> bool {
        self.return_ty.is_some()
    }
}

impl Cfg {
    fn reverse_postorder(self: &Cfg) -> HashMap<BlockName, usize> {
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

/// Get the underyling CFG corresponding to the function `func`.
///
/// The structure is reproduced exactly, aside from the addition of a single
/// exit node branched to from all return statements.
pub(crate) fn to_cfg(func: &Function) -> Cfg {
    let mut builder = CfgBuilder::new(func);
    let mut block = Vec::new();
    let mut anns = Vec::new();
    let mut current = builder.cfg.entry;
    let mut had_branch = false;
    for inst in &func.instrs {
        match inst {
            Code::Label { label, pos } => {
                let next_block = builder.get_index(label);
                builder.finish_block(current, mem::take(&mut block), mem::take(&mut anns));
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
                            arg: arg.into(),
                            val: true.into(),
                        },
                        pos: pos.clone(),
                    },
                );
                builder.add_edge(
                    current,
                    false_block,
                    Branch {
                        op: BranchOp::Cond {
                            arg: arg.into(),
                            val: false.into(),
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
                        anns.push(Annotation::AssignRet { src: arg.into() });
                        builder.add_edge(
                            current,
                            builder.cfg.exit,
                            Branch {
                                op: BranchOp::Jmp,
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
    builder.finish_block(current, block, anns);
    if !had_branch {
        builder.add_edge(
            current,
            builder.cfg.exit,
            Branch {
                op: BranchOp::Jmp,
                pos: None,
            },
        )
    }

    builder.cfg
}

struct CfgBuilder {
    cfg: Cfg,
    label_to_block: HashMap<String, NodeIndex>,
}

impl CfgBuilder {
    fn new(func: &Function) -> CfgBuilder {
        let mut graph = StableDiGraph::default();
        let entry = graph.add_node(BasicBlock::empty(BlockName::Entry));
        let exit = graph.add_node(BasicBlock::empty(BlockName::Exit));
        CfgBuilder {
            cfg: Cfg {
                args: func.args.clone(),
                graph,
                entry,
                exit,
                name: func.name.clone(),
                return_ty: func.return_type.clone(),
            },
            label_to_block: HashMap::new(),
        }
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

    fn finish_block(&mut self, index: NodeIndex, block: Vec<Instruction>, anns: Vec<Annotation>) {
        let BasicBlock { instrs, footer, .. } = self.cfg.graph.node_weight_mut(index).unwrap();
        debug_assert!(instrs.is_empty());
        debug_assert!(footer.is_empty());
        *instrs = block;
        *footer = anns;
    }

    fn set_pos(&mut self, index: NodeIndex, pos: Option<Position>) {
        self.cfg.graph.node_weight_mut(index).unwrap().pos = pos;
    }

    fn add_edge(&mut self, src: NodeIndex, dst: NodeIndex, branch: Branch) {
        self.cfg.graph.add_edge(src, dst, branch);
    }
}
