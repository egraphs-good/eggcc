use bril_rs::Type;

use self::{
    constant_fold::constant_fold_egglog, extraction_rules::extraction_rules,
    loop_invariant::loop_invariant_detection, passthrough_optimize::passthrough_optimize_rules,
    reassoc::reassoc_rules,
};

pub(crate) mod constant_fold;
pub(crate) mod extraction_rules;
pub(crate) mod fast_analyses;
pub(crate) mod loop_invariant;
pub(crate) mod passthrough_optimize;
pub(crate) mod reassoc;
pub(crate) mod subst;

pub fn rvsdg_egglog_code() -> String {
    let code = vec![
        include_str!("schema.egg").to_string(),
        fast_analyses::all_rules(),
        subst::all_rules(),
        include_str!("util.egg").to_string(),
        constant_fold_egglog(),
        extraction_rules(),
        passthrough_optimize_rules(),
        include_str!("gamma_rewrites.egg").to_string(),
        passthrough_optimize_rules(),
        include_str!("interval-analysis.egg").to_string(),
        include_str!("rvsdg-logic.egg").to_string(),
        include_str!("loop-optimizations.egg").to_string(),
        include_str!("function_inline.egg").to_string(),
        include_str!("conditional_invariant_code_motion.egg").to_string(),
        reassoc_rules(),
        loop_invariant_detection(),
        include_str!("loop_strength_red.egg").to_string(),
    ];
    code.join("\n")
}

pub fn rvsdg_egglog_schedule() -> String {
    "(run-schedule
        ; It is sound to not saturate fast-analyses/subst, but we do because
        ; they won't blow up and will help other rules go through.
        (repeat 5
            (saturate fast-analyses)
            ;; extraction rules- vector extraction is expensive, interleave with other extraction rules
            (seq (saturate extraction) (saturate extraction-vec))
            (run)
            (saturate subst))
        ; Right now, subst-beneath is inefficent (it extracts every possible
        ; spine - we are working on this!), so we only run it a few times at the
        ; end to apply substitutions that the main optimizations find. It's
        ; interleaved with fast-analyses because it relies on reified vecs.
        (seq (saturate fast-analyses) (saturate boundary-analyses) (saturate loop-inv-motion))
        (repeat 1000 subst)
        (repeat 6 subst-beneath (saturate fast-analyses))
    )"
    .to_string()
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
const BRIL_OPS: [BrilOp; 11] = [
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
    // add after a bool eq function is added to egglog
    // BrilOp {
    //     op: "beq",
    //     egglog_op: "bool-=",
    //     input_types: [Some(Type::Int), Some(Type::Int)],
    //     output_type: Type::Bool,
    // },
    BrilOp {
        op: "blt",
        egglog_op: "bool-<",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Bool,
    },
    BrilOp {
        op: "bgt",
        egglog_op: "bool->",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Bool,
    },
    BrilOp {
        op: "ble",
        egglog_op: "bool-<=",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Bool,
    },
    BrilOp {
        op: "bge",
        egglog_op: "bool->=",
        input_types: [Some(Type::Int), Some(Type::Int)],
        output_type: Type::Bool,
    },
    BrilOp {
        op: "bnot",
        egglog_op: "not",
        input_types: [Some(Type::Bool), None],
        output_type: Type::Bool,
    },
    BrilOp {
        op: "band",
        egglog_op: "and",
        input_types: [Some(Type::Bool), Some(Type::Bool)],
        output_type: Type::Bool,
    },
    BrilOp {
        op: "bor",
        egglog_op: "or",
        input_types: [Some(Type::Bool), Some(Type::Bool)],
        output_type: Type::Bool,
    },
];
