use bril_rs::Type;

use self::constant_fold::constant_fold_egglog;

pub(crate) mod constant_fold;

pub fn rvsdg_egglog_code() -> String {
    let code = vec![
        include_str!("schema.egg").to_string(),
        include_str!("subst.egg").to_string(),
        include_str!("shift.egg").to_string(),
        constant_fold_egglog(),
        include_str!("gamma_rewrite.egg").to_string(),
    ];
    code.join("\n")
}

pub fn rvsdg_egglog_schedule() -> String {
    "(run 3)".to_string()
}

#[derive(Debug, PartialEq, Clone)]
struct BrilOp {
    op: &'static str,
    egglog_op: &'static str,
    // so far we don't need more than 2 types
    // if the field is None, it is beyond the number of inputs
    input_types: [Option<Type>; 2],
    output_type: Type,
}

impl BrilOp {
    pub fn num_inputs(&self) -> usize {
        self.input_types.iter().filter(|t| t.is_some()).count()
    }
}

// an in-progress list of bril operators and their implementation in egglog
// TODO do I really need to put the constant here for the size of the array?
const BRIL_OPS: [BrilOp; 4] = [
    BrilOp {
        op: "add",
        egglog_op: "+",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
    BrilOp {
        op: "sub",
        egglog_op: "-",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
    BrilOp {
        op: "mul",
        egglog_op: "*",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
    BrilOp {
        op: "div",
        egglog_op: "/",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
];
