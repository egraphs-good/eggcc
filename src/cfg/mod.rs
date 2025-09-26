//! Parse a bril program into a CFG.
//!
//! The methods here largely ignore the instructions in the program: all that we
//! look for here are instructions that may break up basic blocks (`jmp`, `br`,
//! `ret`), and labels. All other instructions are copied into the CFG.
use core::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use std::{fmt, mem};

use bril_rs::{Argument, Code, EffectOps, Function, Instruction, Position, Program, Type};
use indexmap::IndexMap;
use indexmap::IndexSet;
use petgraph::dot::Dot;

use petgraph::stable_graph::{EdgeReference, StableDiGraph};
use petgraph::visit::{EdgeRef, Visitable};
use petgraph::{graph::NodeIndex, visit::DfsPostOrder};

use crate::rvsdg::from_cfg::FunctionTypes;
use crate::util::{run_cmd_line, ListDisplay, Visualization};

/// A subset of nodes for a particular CFG.
pub(crate) type NodeSet = <StableDiGraph<BasicBlock, Branch> as Visitable>::Map;

#[cfg(test)]
pub(crate) mod tests;

pub(crate) mod to_bril;

/// Convert a program to a cfg.
/// Loops over all the functions, translating individually.
pub(crate) fn program_to_cfg(program: &Program) -> SimpleCfgProgram {
    eprintln!("Converting program to CFG");
    let mut functions = Vec::new();
    for func in &program.functions {
        let cfg = function_to_cfg(func);
        functions.push(cfg);
    }

    // If one of the functions is called "main", put it last
    functions.sort_by_key(|f| f.name == "main");
    CfgProgram { functions }
}

#[derive(Clone, Debug)]
pub struct CfgProgram<CfgType> {
    /// A list of functions in the program.
    /// The last entry of this list is the entry point.
    pub functions: Vec<CfgFunction<CfgType>>,
}

/// Simple programs only branch on booleans defined in bril
pub type SimpleCfgProgram = CfgProgram<Simple>;
/// Switch programs can also branch on values defined in annotations
pub type SwitchCfgProgram = CfgProgram<Switch>;

impl SimpleCfgProgram {
    /// Convert a simple program to a switch program
    /// trivial, since simple programs are a subset of switch programs
    pub fn into_switch(self) -> SwitchCfgProgram {
        SwitchCfgProgram {
            functions: self
                .functions
                .into_iter()
                .map(|f| f.into_switch())
                .collect(),
        }
    }
}

impl SimpleCfgFunction {
    pub fn into_switch(self) -> SwitchCfgFunction {
        SwitchCfgFunction {
            args: self.args,
            graph: self.graph,
            entry: self.entry,
            exit: self.exit,
            name: self.name,
            return_ty: self.return_ty,
            _phantom: Switch,
        }
    }
}

impl<CfgType> CfgProgram<CfgType> {
    pub(crate) fn function_types(&self) -> FunctionTypes {
        let mut types = FunctionTypes::default();
        for func in &self.functions {
            let output_type = func.return_ty.clone();
            types.insert(func.name.clone(), output_type);
        }
        types
    }

    pub(crate) fn visualizations(&self) -> Vec<Visualization> {
        let mut visualizations = vec![];
        for function in &self.functions {
            let svg = function.to_svg();
            visualizations.push(Visualization {
                result: svg,
                file_extension: ".svg".to_string(),
                name: function.name.clone(),
            });
        }
        visualizations
    }
}

/// The name (or label) associated with a basic block.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockName {
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
    Identifier::Num(usize::MAX - 1)
}

/// The distinguished identifier associated with "state".
///
/// To recover implicit ordering dependencies between impure operations in bril,
/// we treat effectful operations as taking an "extra argument" and then
/// assigning to that a state variable. This allows the rest of the
/// transformation to preserve the ordering information. So code like:
///
/// > x: int = const 1
/// > print x
/// > y: int = const 2
/// > print y
///
/// Is effectively translated to:
///
/// > x: int = const 1
/// > <state> = print x <state>
/// > y: int = const 2
/// > <state> = print y <state>
///
/// `state_id` is the identifier corresponding to this <state> variable.
pub(crate) fn state_id() -> Identifier {
    Identifier::Num(usize::MAX)
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
    // not present in Simple CFGs
    AssignCond { dst: Identifier, cond: u32 },
    // can be present in Simple CFGs
    AssignRet { src: Identifier },
}

/// A branch-free sequence of instructions.
#[derive(Clone, PartialEq)]
pub struct BasicBlock {
    /// The primary instructions for a block.
    pub(crate) instrs: Vec<Instruction>,
    /// Any annotations added to the end of the block during restructuring.
    pub(crate) footer: Vec<Annotation>,
    /// The name for the block.
    pub(crate) name: BlockName,
    pub(crate) pos: Option<Position>,
}

impl Debug for BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut code_block = self.to_code();
        code_block.extend(self.debug_code_for_footer());
        write!(f, "{}", ListDisplay(code_block, "\n"))
    }
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

    /// Annotations should not be directly translated back into bril, as they
    /// include values (like "conditions" branching >2 ways) that do not have a
    /// clear bril type.
    ///
    /// This method is used for debug printing purposes only, and for that
    /// reason it "reflects" conditions back into bril to the best of its
    /// ability.
    fn debug_code_for_footer(&self) -> impl Iterator<Item = Code> + '_ {
        self.footer.iter().map(|ann| match ann {
            // Conditions are untyped and they do not generally make sense to directly "reflect" back into bril. For the purposes of printing.
            Annotation::AssignCond { dst, cond } => Code::Instruction(Instruction::Constant {
                dest: format!("{dst}"),
                op: bril_rs::ConstOps::Const,
                pos: None,
                const_type: if *cond < 2 { Type::Bool } else { Type::Int },
                value: match cond {
                    0 => bril_rs::Literal::Bool(false),
                    1 => bril_rs::Literal::Bool(true),
                    n => bril_rs::Literal::Int(*n as i64),
                },
            }),
            Annotation::AssignRet { src } => Code::Instruction(Instruction::Effect {
                op: EffectOps::Return,
                args: vec![format!("{src}")],
                funcs: vec![],
                labels: vec![],
                pos: None,
            }),
        })
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
    Cond {
        arg: Identifier,
        /// If the condition matches the
        /// CondVal, take this branch.
        val: CondVal,
        /// A condition can branch on either
        /// a boolean or an integer.
        bril_type: Type,
    },
}

#[derive(Debug, Clone)]
pub struct Switch;
#[derive(Debug, Clone)]
pub struct Simple;

/// The control-flow graph for a single function.
#[derive(Debug, Clone)]
pub struct CfgFunction<CfgType> {
    /// The arguments to the function.
    pub(crate) args: Vec<Argument>,
    /// The graph itself.
    /// Invariant: contains only reachable nodes.
    pub(crate) graph: StableDiGraph<BasicBlock, Branch>,
    /// The entry node for the CFG.
    pub(crate) entry: NodeIndex,
    /// The (single) exit node for the CFG.
    pub(crate) exit: NodeIndex,
    /// The name of the function.
    pub(crate) name: String,
    pub(crate) _phantom: CfgType,
    pub(crate) return_ty: Option<Type>,
}

/// A simple CFG branches only on booleans, instead of allowing annotations
/// and branching on these annotations.
pub type SimpleCfgFunction = CfgFunction<Simple>;
pub type SwitchCfgFunction = CfgFunction<Switch>;

impl<CfgType> CfgFunction<CfgType> {
    pub(crate) fn remove_unreachable(&mut self) {
        let mut reachable = IndexSet::new();
        let mut dfs = DfsPostOrder::new(&self.graph, self.entry);
        while let Some(node) = dfs.next(&self.graph) {
            reachable.insert(node);
        }
        let mut to_remove = Vec::new();
        for node in self.graph.node_indices() {
            if !reachable.contains(&node) {
                to_remove.push(node);
            }
        }
        for node in to_remove {
            self.graph.remove_node(node);
        }
    }
    pub(crate) fn has_return_value(&self) -> bool {
        self.return_ty.is_some()
    }

    pub fn to_dot(&self) -> String {
        format!("{:?}", Dot::new(&self.graph))
    }

    pub fn to_svg(&self) -> String {
        let dot_code = self.to_dot();
        run_cmd_line("dot", ["-Tsvg"], &dot_code).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum SimpleBranch {
    NoBranch, // must be the end of the function
    Jmp(BlockName),
    If {
        // arg is an integer specifying which branch to jump to
        arg: Identifier,
        then_branch: BlockName,
        else_branch: BlockName,
    },
}

impl SimpleCfgFunction {
    pub fn get_branch(&self, node: NodeIndex) -> SimpleBranch {
        let outgoing = self.graph.edges(node);
        match &outgoing.collect::<Vec<EdgeReference<Branch>>>().as_slice() {
            [] => {
                assert!(self.exit == node);
                SimpleBranch::NoBranch
            }
            [edge] => {
                let target: NodeIndex = edge.target();
                let branch = edge.weight();
                let BranchOp::Jmp = branch.op else {
                    panic!("Unexpected branch type");
                };
                SimpleBranch::Jmp(self.graph[target].name.clone())
            }
            [edge1, edge2] => match (&edge1.weight().op, &edge2.weight().op) {
                (
                    BranchOp::Cond {
                        arg: cond1,
                        val: CondVal { val: val1, of: 2 },
                        bril_type: Type::Bool,
                    },
                    BranchOp::Cond {
                        arg: cond2,
                        val: CondVal { val: val2, of: 2 },
                        bril_type: Type::Bool,
                    },
                ) => {
                    assert_eq!(cond1, cond2);

                    if *val1 == 0 && *val2 == 1 {
                        // swap then and else branches
                        SimpleBranch::If {
                            arg: cond1.clone(),
                            then_branch: self.graph[edge2.target()].name.clone(),
                            else_branch: self.graph[edge1.target()].name.clone(),
                        }
                    } else if *val1 == 1 && *val2 == 0 {
                        SimpleBranch::If {
                            arg: cond1.clone(),
                            then_branch: self.graph[edge1.target()].name.clone(),
                            else_branch: self.graph[edge2.target()].name.clone(),
                        }
                    } else {
                        panic!("Unexpected branch values");
                    }
                }
                _ => panic!(
                    "Invalid branch types {:?} and {:?}",
                    edge1.weight().op,
                    edge2.weight().op
                ),
            },
            _ => panic!("Too many outgoing edges"),
        }
    }
}

/// Get the underyling CFG corresponding to the function `func`.
///
/// The structure is reproduced exactly, aside from the addition of a single
/// exit node branched to from all return statements.
/// Generates a Cfg<Switch> because it returns a value in the annotation
pub(crate) fn function_to_cfg(func: &Function) -> SimpleCfgFunction {
    eprintln!("Converting function {} to CFG", func.name);
    let mut builder = CfgBuilder::new(func);
    let mut block = Vec::new();
    let mut anns = Vec::new();
    let mut current = builder.cfg.entry;
    let mut had_branch = false;
    for inst in &func.instrs {
        match inst {
            Code::Label { label, pos } => {
                assert!(
                    label != &BlockName::Entry.to_string(),
                    "not allowed to use entry___ as a label"
                );
                assert!(
                    label != &BlockName::Exit.to_string(),
                    "not allowed to use exit___ as a label"
                );
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
            }) if !had_branch => {
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
                            bril_type: Type::Bool,
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
                            bril_type: Type::Bool,
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
            }) if !had_branch => {
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
            }) if !had_branch => {
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
            Code::Instruction(i) if !had_branch => block.push(i.clone()),
            // If we have already hit a branch in this block,
            // avoid emitting any further instructions.
            _ => {}
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

    // also, remove any unreachable blocks
    builder.cfg.remove_unreachable();

    builder.cfg
}

struct CfgBuilder {
    cfg: SimpleCfgFunction,
    label_to_block: IndexMap<String, NodeIndex>,
}

impl CfgBuilder {
    fn new(func: &Function) -> CfgBuilder {
        let mut graph = StableDiGraph::default();
        let entry = graph.add_node(BasicBlock::empty(BlockName::Entry));
        let exit = graph.add_node(BasicBlock::empty(BlockName::Exit));
        CfgBuilder {
            cfg: SimpleCfgFunction {
                args: func.args.clone(),
                graph,
                entry,
                exit,
                name: func.name.clone(),
                return_ty: func.return_type.clone(),
                _phantom: Simple,
            },
            label_to_block: IndexMap::new(),
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
