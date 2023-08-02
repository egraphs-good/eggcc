use std::fmt::Display;

use super::BasicBlock;
use bril_rs::Argument;

#[derive(Debug, PartialEq)]
pub(crate) enum StructuredBlock {
    // Variable, then and else blocks
    Ite(String, Box<StructuredBlock>, Box<StructuredBlock>),
    Loop(Box<StructuredBlock>),
    Block(Box<StructuredBlock>),
    Sequence(Vec<StructuredBlock>),
    // how many layers of blocks / loops to break out of
    Break(usize),
    Basic(Box<BasicBlock>),
}

#[derive(Debug)]
pub(crate) struct StructuredFunction {
    pub(crate) args: Vec<Argument>,
    pub(crate) block: StructuredBlock,
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
            StructuredBlock::Loop(body) => format!(
                "{}while true:\n{}",
                " ".repeat(indent),
                body.display(indent + 1)
            ),
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
        }
    }
}
