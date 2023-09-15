use bril_rs::{Code, Function, Program};

use super::{Operand, RvsdgFunction, RvsdgProgram};

impl RvsdgProgram {
    pub fn to_bril(&self) -> Program {
        // TODO right now we only support one function
        // which is named main
        assert!(self.functions.len() == 1);
        Program {
            functions: self.functions.iter().map(|f| f.to_bril()).collect(),
            imports: vec![],
        }
    }
}

impl RvsdgFunction {
    pub fn to_bril(&self) -> Function {
        let mut instrs = vec![];

        self.operand_to_bril(self.state, &mut instrs);
        if let Some(operand) = self.result {
            self.operand_to_bril(operand, &mut instrs);
        }

        // TODO hard-coded name
        Function {
            name: "main".into(),
            args: vec![],
            instrs,
            pos: None,
            return_type: None,
        }
    }

    pub fn operand_to_bril(&self, operand: Operand, instrs: &mut Vec<Code>) {}
}
