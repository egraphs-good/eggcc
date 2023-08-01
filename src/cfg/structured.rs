use super::BasicBlock;
use bril_rs::Argument;

type Label = String;

#[derive(Debug, PartialEq)]
pub(crate) enum StructuredBlock {
    // Variable, then and else blocks
    Ite(String, Box<StructuredBlock>, Box<StructuredBlock>),
    Block(Label, Box<StructuredBlock>),
    Sequence(Vec<StructuredBlock>),
    Break(Label),
    Basic(Box<BasicBlock>),
}

#[derive(Debug)]
pub(crate) struct StructuredFunction {
    pub(crate) args: Vec<Argument>,
    pub(crate) block: StructuredBlock,
}
