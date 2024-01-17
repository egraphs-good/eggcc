use std::fmt::Display;

use super::BasicBlock;
use bril_rs::{Argument, Code, EffectOps, Function, Instruction, Program};

#[derive(Debug, PartialEq, Clone)]
pub enum StructuredBlock {
    // Variable, then and else blocks
    Ite(String, Box<StructuredBlock>, Box<StructuredBlock>),
    Loop(Box<StructuredBlock>),
    Block(Box<StructuredBlock>),
    Sequence(Vec<StructuredBlock>),
    // how many layers of blocks / loops to break out of
    Break(usize),
    Return(Option<String>),
    Basic(Box<BasicBlock>),
}

#[derive(Debug)]
pub struct StructuredProgram {
    pub functions: Vec<StructuredFunction>,
}

impl Display for StructuredProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut functions = self
            .functions
            .iter()
            .map(|f| format!("{}", f))
            .collect::<Vec<String>>();
        functions.sort();
        write!(f, "{}", functions.join("\n\n"))
    }
}

impl StructuredProgram {
    pub fn to_program(&self) -> Program {
        Program {
            functions: self
                .functions
                .clone()
                .into_iter()
                .map(|f| f.to_function())
                .collect(),
            imports: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructuredFunction {
    pub name: String,
    pub args: Vec<Argument>,
    pub block: StructuredBlock,
}

impl Display for StructuredFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{\n{}\n}}", self.name, self.block)
    }
}

impl Display for StructuredBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display(0))
    }
}

impl StructuredBlock {
    fn display(&self, indent: usize) -> String {
        let indent = indent;
        let whitespace = " ".repeat(indent);
        match self {
            StructuredBlock::Ite(var, then, els) => format!(
                "{whitespace}if {}:\n{}\n{whitespace}else:\n{}",
                var,
                then.display(indent + 1),
                els.display(indent + 1)
            ),
            StructuredBlock::Loop(body) => {
                format!("{whitespace}while true:\n{}", body.display(indent + 1))
            }
            StructuredBlock::Block(body) => {
                format!("{}block:\n{}", " ".repeat(indent), body.display(indent + 1))
            }
            StructuredBlock::Sequence(blocks) => blocks
                .iter()
                .map(|b| b.display(indent))
                .collect::<Vec<String>>()
                .join("\n"),
            StructuredBlock::Break(n) => format!("{}break {}", " ".repeat(indent), n),
            StructuredBlock::Basic(block) => block
                .instrs
                .iter()
                .map(|i| format!("{}{i}", " ".repeat(indent)))
                .collect::<Vec<String>>()
                .join("\n"),
            StructuredBlock::Return(val) => {
                if let Some(val) = val {
                    format!("{whitespace}return {}", val)
                } else {
                    format!("{whitespace}return")
                }
            }
        }
    }
}

impl StructuredFunction {
    pub fn to_function(&self) -> Function {
        let mut builder = StructuredCfgBuilder {
            resulting_code: vec![],
            scopes: vec![],
            fresh_block_name_count: 0,
        };
        self.block.to_code(&mut builder);

        Function {
            name: self.name.clone(),
            args: self.args.clone(),
            instrs: builder.resulting_code,
            pos: None,
            return_type: None,
        }
    }
}

pub struct StructuredCfgBuilder {
    resulting_code: Vec<Code>,
    scopes: Vec<String>,
    fresh_block_name_count: usize,
}

impl StructuredCfgBuilder {
    fn fresh_block_name(&mut self) -> String {
        self.fresh_block_name_count += 1;
        format!("sblock___{}", self.fresh_block_name_count - 1)
    }

    /// find the name of the scope that we break to
    /// when breaking out of `num` layers of blocks
    /// we must break out of at least one block
    fn scope_break_to(&self, num: usize) -> String {
        assert!(num > 0);
        self.scopes[self.scopes.len() - num].clone()
    }
}

impl StructuredBlock {
    pub(crate) fn to_code(&self, builder: &mut StructuredCfgBuilder) {
        match self {
            StructuredBlock::Basic(block) => builder.resulting_code.extend(block.to_code()),
            StructuredBlock::Block(block) => {
                // add a label at the end of the block so
                // we can break out of it
                let end_of_block_label = builder.fresh_block_name();
                builder.scopes.push(end_of_block_label.clone());
                block.to_code(builder);
                builder.resulting_code.push(Code::Label {
                    label: end_of_block_label.clone(),
                    pos: None,
                });
                // pop the scope after we are done processing this block
                assert!(builder.scopes.pop().unwrap() == end_of_block_label);
            }
            StructuredBlock::Break(num) => {
                if *num != 0 {
                    builder
                        .resulting_code
                        .push(Code::Instruction(Instruction::Effect {
                            op: EffectOps::Jump,
                            args: vec![],
                            funcs: vec![],
                            labels: vec![builder.scope_break_to(*num)],
                            pos: None,
                        }));
                }
            }
            StructuredBlock::Ite(cond, then_block, else_block) => {
                let then_name = builder.fresh_block_name();
                let else_name = builder.fresh_block_name();
                builder
                    .resulting_code
                    .push(Code::Instruction(Instruction::Effect {
                        op: EffectOps::Branch,
                        args: vec![cond.to_string()],
                        funcs: vec![],
                        labels: vec![then_name.clone(), else_name.clone()],
                        pos: None,
                    }));
                builder.resulting_code.push(Code::Label {
                    label: then_name,
                    pos: None,
                });
                then_block.to_code(builder);

                builder.resulting_code.push(Code::Label {
                    label: else_name,
                    pos: None,
                });

                else_block.to_code(builder);
            }
            StructuredBlock::Loop(block) => {
                // we need to be able to loop back to the start
                let loop_start_name = builder.fresh_block_name();
                builder.resulting_code.push(Code::Label {
                    label: loop_start_name.clone(),
                    pos: None,
                });

                // loops can be broken out of
                let loop_end_name = builder.fresh_block_name();
                builder.scopes.push(loop_end_name.clone());
                block.to_code(builder);

                // jump back to the start of the loop if you get to the end
                builder
                    .resulting_code
                    .push(Code::Instruction(Instruction::Effect {
                        op: EffectOps::Jump,
                        args: vec![],
                        funcs: vec![],
                        labels: vec![loop_start_name],
                        pos: None,
                    }));
                builder.resulting_code.push(Code::Label {
                    label: loop_end_name.clone(),
                    pos: None,
                });
                assert!(builder.scopes.pop().unwrap() == loop_end_name);
            }
            StructuredBlock::Sequence(blocks) => {
                for block in blocks {
                    block.to_code(builder);
                }
            }
            StructuredBlock::Return(val) => {
                let args = match val {
                    Some(v) => vec![v.clone()],
                    None => vec![],
                };
                builder
                    .resulting_code
                    .push(Code::Instruction(Instruction::Effect {
                        op: EffectOps::Return,
                        args,
                        funcs: vec![],
                        labels: vec![],
                        pos: None,
                    }));
            }
        }
    }
}
