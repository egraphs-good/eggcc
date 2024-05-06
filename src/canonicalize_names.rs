//! This module gives unique names to all the variables in the program.
//! TODO: This is probably just covering up non-determinism issues, so we should
//! track those down instead.

use bril_rs::{Code, Function, Instruction, Program};
use hashbrown::HashMap;

struct Renamer {
    name_map: HashMap<String, String>,
}

pub(crate) fn canonicalize_bril(prog: &Program) -> Program {
    Program {
        functions: prog.functions.iter().map(canonicalize_func_names).collect(),
        imports: prog.imports.clone(),
    }
}

fn canonicalize_func_names(func: &Function) -> Function {
    let mut renamer = Renamer {
        name_map: HashMap::new(),
    };
    for arg in &func.args {
        // don't touch argument names
        renamer.name_map.insert(arg.name.clone(), arg.name.clone());
    }

    Function {
        args: func.args.clone(),
        instrs: renamer.canonicalize_codes_names(&func.instrs),
        name: func.name.clone(),
        pos: func.pos.clone(),
        return_type: func.return_type.clone(),
    }
}

impl Renamer {
    fn get_name(&mut self, name: &str) -> String {
        if let Some(new_name) = self.name_map.get(name) {
            new_name.clone()
        } else {
            let new_name = format!("v{}_", self.name_map.len());
            self.name_map.insert(name.to_string(), new_name.clone());
            new_name
        }
    }

    fn canonicalize_codes_names(&mut self, instrs: &[Code]) -> Vec<Code> {
        instrs
            .iter()
            .map(|instr| self.canonicalize_code_names(instr))
            .collect()
    }

    fn canonicalize_code_names(&mut self, instr: &Code) -> Code {
        match instr {
            Code::Instruction(instr) => {
                Code::Instruction(self.canonicalize_instruction_names(instr))
            }
            Code::Label { label, pos } => Code::Label {
                label: self.get_name(label),
                pos: pos.clone(),
            },
        }
    }

    fn canonicalize_instruction_names(&mut self, instr: &Instruction) -> Instruction {
        match instr {
            Instruction::Constant {
                dest,
                op,
                pos,
                const_type,
                value,
            } => Instruction::Constant {
                dest: self.get_name(dest),
                op: *op,
                pos: pos.clone(),
                const_type: const_type.clone(),
                value: value.clone(),
            },
            Instruction::Value {
                args,
                dest,
                funcs,
                labels,
                op,
                pos,
                op_type,
            } => Instruction::Value {
                args: {
                    let mut args: Vec<_> = args.iter().map(|arg| self.get_name(arg)).collect();
                    use bril_rs::ValueOps::*;
                    if matches!(op, Add | Mul | Eq | And | Or) {
                        args.sort();
                    }
                    args
                },
                dest: self.get_name(dest),
                funcs: funcs.clone(),
                labels: labels.iter().map(|label| self.get_name(label)).collect(),
                op: *op,
                pos: pos.clone(),
                op_type: op_type.clone(),
            },
            Instruction::Effect {
                args,
                funcs,
                labels,
                op,
                pos,
            } => Instruction::Effect {
                args: args.iter().map(|arg| self.get_name(arg)).collect(),
                funcs: funcs.clone(),
                labels: labels.iter().map(|label| self.get_name(label)).collect(),
                op: *op,
                pos: pos.clone(),
            },
        }
    }
}
