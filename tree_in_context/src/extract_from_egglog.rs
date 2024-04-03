use std::collections::HashMap;

use egglog::*;

pub fn serialized_egraph(egraph: egglog::EGraph, root: egglog::Value) -> egraph_serialize::EGraph {
    let mut config = SerializeConfig::default();
    config.root_eclasses = vec![root];
    let egraph = egraph.serialize(config);
    egraph
}

type Cost = f64;

pub struct CostSet {
    pub total: Cost,
    // TODO this would be more efficient as a
    // persistent data structure
    pub costs: HashMap<Value, Cost>,
    pub term: Term,
}

pub fn extract(egraph: egraph_serialize::EGraph) -> CostSet {
    todo!()
}

struct CostModel {
    Op: HashMap<&str, Cost>,
}

impl CostModel {
    fn simple_cost_model() -> CostModel {
        CostModel {
            Op: HashMap::from([
                // Bop
                // TODO: actually we also need type info
                // to figure out the cost
                ("Add", 1.),
                ("Sub", 1.),
                ("Mul", 1.),
                ("Div", 1.),
                ("Eq", 1.),
                ("LessThan", 1.),
                ("GreaterThan", 1.),
                ("LessEq", 1.),
                ("GreaterEq", 1.),
                ("And", 1.),
                ("Or", 1.),
                ("Write", 1.),
                ("PtrAdd", 1.),
                // Uop
                ("Not", 1.),
                ("Print", 1.),
                ("Load", 1.),
                ("Free", 1.),
            ]),
        }
    }
}
