//! eggcc's cost model for effect-safe extraction (egglog-experimental's
//! `effsafe-extract`): a per-operator cost table and the control-flow
//! heuristics for `If` and `DoWhile`.

use egglog::extract::{DagCostModel, DefaultCost};
use egglog::{ArcSort, EGraph, Enode, Function, RegionCostModel, Value};

/// Marginal e-node costs by constructor. Primitives, types and lists are free.
#[derive(Clone, Copy, Debug, Default)]
pub struct EggccCostModel;

pub fn op_cost(op: &str) -> DefaultCost {
    match op {
        "Const" => 10,
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
        "If" => 250,
        // DoWhile's cost is its body, see `EggccRegionCosts`.
        _ => 0,
    }
}

impl DagCostModel<DefaultCost> for EggccCostModel {
    fn base_value_cost(&self, _egraph: &EGraph, _sort: &ArcSort, _value: Value) -> DefaultCost {
        0
    }

    fn enode_cost(&self, _egraph: &EGraph, func: &Function, _enode: &Enode<'_>) -> DefaultCost {
        op_cost(func.name())
    }
}

/// How subregions are charged: both branches of an `If`, the cheaper one at a
/// quarter; a `DoWhile` body as if it ran many times.
#[derive(Clone, Copy, Debug, Default)]
pub struct EggccRegionCosts;

impl RegionCostModel for EggccRegionCosts {
    fn fold_regions(
        &self,
        _egraph: &EGraph,
        constructor: &str,
        costs: &[DefaultCost],
    ) -> DefaultCost {
        match (constructor, costs) {
            ("If", &[then_cost, else_cost]) => {
                then_cost.max(else_cost) + (then_cost.min(else_cost) >> 2)
            }
            // Could be improved by plumbing in the loop iteration analysis.
            ("DoWhile", &[body]) => body.saturating_mul(500),
            _ => costs.iter().fold(0, |acc, &c| acc.saturating_add(c)),
        }
    }
}
