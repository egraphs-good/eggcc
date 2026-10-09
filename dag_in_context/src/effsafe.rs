//! eggcc's cost model for effect-safe extraction (egglog-experimental's
//! `effsafe-extract`): a per-operator cost table and the control-flow
//! heuristics for `If` and `DoWhile`.

use egglog::extract::{DagCostModel, DefaultCost, TreeCostModel};
use egglog::{ArcSort, EGraph, Enode, Function, Value};

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

/// How subregions are charged at region boundaries: both branches of an
/// `If`, the cheaper one at a quarter; a `DoWhile` body as if it ran many
/// times. The fold receives subregion costs in their argument positions and 0
/// elsewhere (the other children are charged by the enclosing region's DAG).
#[derive(Clone, Copy, Debug, Default)]
pub struct EggccRegionCosts;

/// Which boundary rule applies to an e-node, with the e-node's own cost.
#[derive(Clone, Copy, Debug)]
pub struct Boundary {
    kind: BoundaryKind,
    own: DefaultCost,
}

#[derive(Clone, Copy, Debug)]
enum BoundaryKind {
    If,
    DoWhile,
    Other,
}

impl TreeCostModel<DefaultCost> for EggccRegionCosts {
    type EnodeCost = Boundary;
    type ContainerCost = DefaultCost;

    fn base_value_cost(&self, _egraph: &EGraph, _sort: &ArcSort, _value: Value) -> DefaultCost {
        0
    }

    fn enode_cost(&self, _egraph: &EGraph, func: &Function, _enode: &Enode<'_>) -> Boundary {
        let kind = match func.name() {
            "If" => BoundaryKind::If,
            "DoWhile" => BoundaryKind::DoWhile,
            _ => BoundaryKind::Other,
        };
        Boundary {
            kind,
            own: op_cost(func.name()),
        }
    }

    fn container_cost(&self, _egraph: &EGraph, _sort: &ArcSort, _value: Value) -> DefaultCost {
        0
    }

    fn fold_enode_cost(&self, boundary: Boundary, child_costs: &[DefaultCost]) -> DefaultCost {
        let regions = match (boundary.kind, child_costs) {
            // (If pred inputs then else)
            (BoundaryKind::If, &[_, _, then_cost, else_cost]) => {
                then_cost.max(else_cost) + (then_cost.min(else_cost) >> 2)
            }
            // (DoWhile inputs body); could be improved by plumbing in the
            // loop iteration analysis.
            (BoundaryKind::DoWhile, &[_, body]) => body.saturating_mul(500),
            _ => child_costs
                .iter()
                .fold(0 as DefaultCost, |acc, &c| acc.saturating_add(c)),
        };
        boundary.own.saturating_add(regions)
    }

    fn fold_container_cost(&self, own: DefaultCost, element_costs: &[DefaultCost]) -> DefaultCost {
        element_costs
            .iter()
            .fold(own, |acc, &c| acc.saturating_add(c))
    }
}
