//! The cost model: a fixed per-operator cost table for eggcc's IR.

use crate::egraph::ENode;

pub type Cost = u64;

pub const INFINITE: Cost = u64::MAX;

/// Type constructors, which cost nothing to extract.
pub fn is_type_op(op: &str) -> bool {
    matches!(
        op,
        "IntT" | "BoolT" | "FloatT" | "PointerT" | "StateT" | "Base" | "TupleT" | "TNil" | "TCons"
    )
}

/// The cost of an e-node on its own, not counting its children. `If` and
/// `DoWhile` get extra treatment in the greedy and statewalk cost functions.
pub fn enode_cost(enode: &ENode) -> Cost {
    if enode.is_primitive() {
        return 0;
    }
    let op = enode.op.as_str();
    if is_type_op(op) {
        return 0;
    }
    match op {
        "Const" => 10,
        "Arg" | "Int" | "Bool" | "Float" => 0,
        "Empty" | "Single" | "Concat" | "Nil" | "Cons" => 0,
        "Get" => 1,
        "Abs" | "Bitand" | "Neg" | "Add" | "PtrAdd" | "Sub" | "And" | "Or" | "Not" | "Shl"
        | "Shr" => 100,
        "FAdd" | "FSub" | "Fmax" | "Fmin" => 500,
        "Mul" => 300,
        "FMul" => 1500,
        "Div" => 500,
        "FDiv" => 2500,
        "Eq" | "LessThan" | "GreaterThan" | "LessEq" | "GreaterEq" => 100,
        "Select" => 1500,
        "Smax" | "Smin" | "FEq" => 100,
        "FLessThan" | "FGreaterThan" | "FLessEq" | "FGreaterEq" => 1000,
        "Print" | "Write" | "Load" => 500,
        "Alloc" | "Free" => 1000,
        "Call" => 500_000,
        "Program" | "Function" => 0,
        "DoWhile" => 1,
        "If" | "Switch" => 250,
        "Uop" | "Bop" | "Top" => 0,
        _ => {
            debug_assert!(false, "op of unknown cost: {op}");
            0
        }
    }
}

/// Heuristic cost of an `If`: both branches are charged, the cheaper one at a quarter.
pub fn if_branches_cost(then_cost: Cost, else_cost: Cost) -> Cost {
    then_cost.max(else_cost) + (then_cost.min(else_cost) >> 2)
}

/// Heuristic cost of a `DoWhile` body, assuming it runs many times.
pub fn loop_body_cost(body_cost: Cost) -> Cost {
    // Could be improved by plumbing in the loop iteration analysis. This can
    // overflow for deeply nested loops.
    body_cost * 500
}
