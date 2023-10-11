use bril_rs::Type;

use self::{constant_fold::constant_fold_egglog, subst::subst_rules};

pub(crate) mod constant_fold;
pub(crate) mod subst;

pub fn rvsdg_egglog_code() -> String {
    let code = vec![
        include_str!("schema.egg").to_string(),
        subst_rules(),
        include_str!("shift.egg").to_string(),
        constant_fold_egglog(),
    ];
    code.join("\n")
}

pub fn rvsdg_egglog_schedule() -> String {
    "(run-schedule (repeat 3 (run) (saturate subst)))".to_string()
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

// an in-progress list of bril operators and their implementation in egglog
// TODO do I really need to put the constant here for the size of the array?
const BRIL_OPS: [BrilOp; 5] = [
    BrilOp {
        op: "badd",
        egglog_op: "+",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
    BrilOp {
        op: "bsub",
        egglog_op: "-",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
    BrilOp {
        op: "bmul",
        egglog_op: "*",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
    BrilOp {
        op: "bdiv",
        egglog_op: "/",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Int,
    },
    BrilOp {
        op: "blt",
        egglog_op: "bool-<",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Bool,
    },
];
