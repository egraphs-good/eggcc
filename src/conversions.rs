use crate::EggCCError;
use bril_rs::{Code, Instruction, Program, ValueOps};

/// The bril to_ssa script generates __undefined variables
/// whenever a variable is used before it is defined in a phi node.
/// We reject these programs because it means the variable was not defined
/// in all control flow paths to the phi node.
pub fn check_for_uninitialized_vars(prog: &Program) -> Result<(), EggCCError> {
    for func in &prog.functions {
        for instr in &func.instrs {
            if let Code::Instruction(Instruction::Value {
                dest: _,
                args,
                funcs: _funcs,
                op: ValueOps::Phi,
                labels: _labels,
                pos: _pos,
                op_type: _op_type,
            }) = instr
            {
                assert!(args.len() == 2);
                if args[0] == "__undefined" {
                    return Err(EggCCError::UninitializedVariable(
                        args[1].clone(),
                        func.name.clone(),
                    ));
                } else if args[1] == "__undefined" {
                    return Err(EggCCError::UninitializedVariable(
                        args[0].clone(),
                        func.name.clone(),
                    ));
                }
            }
        }
    }
    Ok(())
}
