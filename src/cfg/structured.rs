use std::fmt::Display;

use super::BasicBlock;
use bril_rs::Argument;

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
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
