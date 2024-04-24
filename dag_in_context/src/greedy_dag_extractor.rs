use egglog::{util::IndexMap, *};
use egraph_serialize::{ClassId, EGraph, NodeId};
use ordered_float::NotNan;
use rustc_hash::FxHashMap;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    from_egglog::FromEgglog,
    schema::{RcExpr, TreeProgram, Type},
    typechecker::TypeChecker,
};

type RootId = ClassId;

pub(crate) struct EgraphInfo<'a> {
    pub(crate) egraph: &'a EGraph,
    // For every (root, eclass) pair, store the parent
    // (root, enode) pairs that may depend on it.
    pub(crate) parents: HashMap<(RootId, ClassId), Vec<(RootId, NodeId)>>,
    pub(crate) roots: Vec<(RootId, NodeId)>,
    pub(crate) cm: &'a dyn CostModel,
    /// A set of names of functions that are unextractable
    unextractables: HashSet<String>,
}

pub(crate) struct Extractor<'a> {
    pub(crate) termdag: &'a mut TermDag,
    costs: FxHashMap<ClassId, FxHashMap<ClassId, CostSet>>,

    // use to get the type of an expression
    pub(crate) typechecker: TypeChecker<'a>,

    // Each term must correspond to a node in the egraph. We store that here
    // Use an indexmap for deterministic order of iteration
    pub(crate) correspondence: IndexMap<Term, NodeId>,
    // Get the expression corresponding to a term.
    // This is computed after the extraction is done.
    pub(crate) term_to_expr: Option<HashMap<Term, RcExpr>>,
    pub(crate) node_to_type: Option<HashMap<NodeId, Type>>,
}

impl<'a> EgraphInfo<'a> {
    fn is_region_node(&self, node_id: NodeId) -> bool {
        !enode_regions(self.egraph, &self.egraph[&node_id]).is_empty()
    }

    pub(crate) fn new(
        cm: &'a dyn CostModel,
        egraph: &'a EGraph,
        unextractables: HashSet<String>,
    ) -> Self {
        // get all the roots needed
        let mut region_roots = HashSet::new();
        for node in egraph.classes().values().flat_map(|c| &c.nodes) {
            for root in enode_regions(egraph, &egraph[node]) {
                region_roots.insert(root);
            }
        }
        // also add the root of the egraph to region_roots
        region_roots.insert(egraph.nid_to_cid(&get_root(egraph)).clone());

        // find all the (root, child) pairs that are important
        let mut relavent_nodes: Vec<(ClassId, ClassId)> = vec![];
        for root in &region_roots {
            let reachable = region_reachable_classes(egraph, root.clone());
            for eclass in reachable {
                relavent_nodes.push((root.clone(), eclass));
            }
        }

        let mut roots = vec![];
        // find all the (root, enode) pairs that are root nodes (no children)
        for (root, eclass) in &relavent_nodes {
            for enode in egraph.classes()[eclass].nodes.iter() {
                if enode_children(egraph, &egraph[enode]).is_empty() {
                    roots.push((root.clone(), enode.clone()));
                }
            }
        }

        // sort roots for determinism
        roots.sort();

        let mut parents: HashMap<(RootId, ClassId), HashSet<(RootId, NodeId)>> = HashMap::new();
        for (root, eclass) in relavent_nodes {
            // iterate over every root, enode pair
            for enode in egraph.classes()[&eclass].nodes.iter() {
                // add to the parents table
                for (child, isregion) in enode_children(egraph, &egraph[enode]) {
                    let child_region = if isregion {
                        child.clone()
                    } else {
                        root.clone()
                    };
                    parents
                        .entry((child_region, child.clone()))
                        .or_default()
                        .insert((root.clone(), enode.clone()));
                }
            }
        }

        let mut parents_sorted = HashMap::new();
        for (key, parents) in parents {
            let mut parents_vec = parents.into_iter().collect::<Vec<_>>();
            parents_vec.sort();
            parents_sorted.insert(key, parents_vec);
        }

        EgraphInfo {
            cm,
            egraph,
            unextractables,
            parents: parents_sorted,
            roots,
        }
    }
}

impl<'a> Extractor<'a> {
    pub(crate) fn term_to_expr(&mut self, term: &Term) -> RcExpr {
        self.term_to_expr
            .as_ref()
            .unwrap()
            .get(term)
            .unwrap_or_else(|| panic!("Failed to find correspondence for term {:?}", term))
            .clone()
    }

    #[allow(dead_code)]
    pub(crate) fn term_to_type(&mut self, term: &Term) -> Type {
        let expr = self.term_to_expr(term);
        self.typechecker.add_arg_types_to_expr(expr, &None).0
    }

    pub(crate) fn expr_to_type(&mut self, expr: &RcExpr) -> Type {
        self.typechecker
            .add_arg_types_to_expr(expr.clone(), &None)
            .0
    }

    /// Checks if an expressions is effectful by checking if it returns something of type state.
    pub(crate) fn is_effectful(&mut self, expr: &RcExpr) -> bool {
        let ty = self.expr_to_type(expr);
        ty.contains_state()
    }

    /// Checks if an already extracted node is effectful.
    pub(crate) fn is_node_effectful(&mut self, node_id: NodeId) -> bool {
        let node_type = self.node_to_type.as_ref().unwrap().get(&node_id).unwrap();
        node_type.contains_state()
    }

    /// Convert the extracted terms to expressions, and also
    /// store their types.
    fn terms_to_expressions(&mut self, info: &EgraphInfo, prog: Term) -> TreeProgram {
        let mut converter = FromEgglog {
            termdag: self.termdag,
            conversion_cache: Default::default(),
        };
        let mut node_to_type: HashMap<NodeId, Type> = Default::default();

        for (term, node_id) in &self.correspondence {
            let node = info.egraph.nodes.get(node_id).unwrap();
            let eclass = info.egraph.nid_to_cid(node_id);
            let type_of_eclass = info
                .egraph
                .class_data
                .get(eclass)
                .unwrap()
                .typ
                .as_ref()
                .unwrap();
            // only convert expressions (that are not functions)
            if type_of_eclass == "Expr" && node.op != "Function" {
                let expr = converter.expr_from_egglog(term.clone());
                let ty = self
                    .typechecker
                    .add_arg_types_to_expr(expr.clone(), &None)
                    .0;
                node_to_type.insert(node_id.clone(), ty);
            }
        }

        let converted_prog = converter.program_from_egglog(prog);

        self.node_to_type = Some(node_to_type);
        self.term_to_expr = Some(converter.conversion_cache);

        // return the converted program
        converted_prog
    }

    pub(crate) fn node_of(&self, term: &Term) -> NodeId {
        self.correspondence
            .get(term)
            .unwrap_or_else(|| panic!("Failed to find correspondence for term {:?}", term))
            .clone()
    }

    pub(crate) fn new(original_prog: &'a TreeProgram, termdag: &'a mut TermDag) -> Self {
        Extractor {
            termdag,
            correspondence: Default::default(),
            costs: Default::default(),
            term_to_expr: Default::default(),
            typechecker: TypeChecker::new(original_prog, true),
            node_to_type: Default::default(),
        }
    }
}

fn get_root(egraph: &egraph_serialize::EGraph) -> NodeId {
    let mut root_nodes = egraph
        .nodes
        .iter()
        .filter(|(_nid, node)| node.op == "Program");
    let res = root_nodes.next().unwrap();
    assert!(root_nodes.next().is_none());
    res.0.clone()
}

pub fn get_unextractables(egraph: &egglog::EGraph) -> HashSet<String> {
    let unextractables = egraph
        .functions
        .iter()
        .filter_map(|(name, func)| {
            if func.is_extractable() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect();
    unextractables
}

pub fn serialized_egraph(
    egglog_egraph: egglog::EGraph,
) -> (egraph_serialize::EGraph, HashSet<String>) {
    let config = SerializeConfig::default();
    let egraph = egglog_egraph.serialize(config);

    (egraph, get_unextractables(&egglog_egraph))
}

type Cost = NotNan<f64>;

#[derive(Clone, Debug)]
pub struct CostSet {
    pub total: Cost,
    // TODO perhaps more efficient as
    // persistent data structure?
    pub costs: HashMap<ClassId, Cost>,
    pub term: Term,
}

impl<'a> Extractor<'a> {
    /// Construct a term for this operator with subterms from the cost sets
    /// We also need to add this term to the correspondence map so we can
    /// find its enode id later.
    fn get_term(&mut self, info: &EgraphInfo, node_id: NodeId, children: Vec<Term>) -> Term {
        let node = &info.egraph[&node_id];
        let op = &node.op;
        let term = if children.is_empty() {
            if op.starts_with('\"') {
                self.termdag
                    .lit(ast::Literal::String(op[1..op.len() - 1].into()))
            } else if let Ok(n) = op.parse::<i64>() {
                self.termdag.lit(ast::Literal::Int(n))
            } else if op == "true" {
                self.termdag.lit(ast::Literal::Bool(true))
            } else if op == "false" {
                self.termdag.lit(ast::Literal::Bool(false))
            } else {
                self.termdag.app(op.into(), children)
            }
        } else {
            self.termdag.app(op.into(), children)
        };

        self.correspondence.insert(term.clone(), node_id);

        term
    }
}

/// Calculates the cost set of a node based on cost sets of children.
/// Handles cycles by returning a cost set with infinite cost.
/// Returns None when costs for children are not yet available.
fn calculate_cost_set(
    rootid: ClassId,
    node_id: NodeId,
    extractor: &mut Extractor,
    info: &EgraphInfo,
) -> Option<CostSet> {
    let node = &info.egraph[&node_id];
    let cid = info.egraph.nid_to_cid(&node_id);

    let region_costs = extractor.costs.get(&rootid).unwrap();
    // get the cost sets for the children
    let child_cost_sets = enode_children(info.egraph, node)
        .iter()
        .filter_map(|(cid, is_region_root)| {
            if *is_region_root {
                Some((extractor.costs.get(cid)?.get(cid)?, *is_region_root))
            } else {
                region_costs
                    .get(cid)
                    .map(|cost_set| (cost_set, *is_region_root))
            }
        })
        .collect::<Vec<_>>();
    // if any are unavailable, we return none from this whole function
    if child_cost_sets.len() < node.children.len() {
        return None;
    }

    // cycle detection
    if child_cost_sets
        .iter()
        .any(|(cs, _is_region_root)| cs.costs.contains_key(cid))
    {
        return Some(CostSet {
            costs: Default::default(),
            total: std::f64::INFINITY.try_into().unwrap(),
            // returns junk children since this cost set is guaranteed to not be selected.
            term: extractor.termdag.app(node.op.as_str().into(), vec![]),
        });
    }

    let mut shared_total = NotNan::new(0.).unwrap();
    let mut unshared_total = info.cm.get_op_cost(&node.op);
    let mut costs: HashMap<ClassId, NotNan<f64>> = HashMap::default();

    if !info.cm.ignore_children(&node.op) {
        for (child_set, is_region_root) in child_cost_sets.iter() {
            if *is_region_root {
                unshared_total += child_set.total;
            } else {
                for (child_cid, child_cost) in &child_set.costs {
                    // it was already present in the set
                    if let Some(existing) = costs.get(child_cid) {
                        if existing > child_cost {
                            // if we found a lower-cost alternative for this child, use that and decrease cost
                            shared_total -= existing - *child_cost;
                        }
                        costs.insert(child_cid.clone(), *existing.min(child_cost));
                    } else {
                        costs.insert(child_cid.clone(), *child_cost);
                        shared_total += child_cost;
                    }
                }
            }
        }
    }
    costs.insert(cid.clone(), unshared_total);
    let total = unshared_total + shared_total;

    let sub_terms: Vec<Term> = child_cost_sets
        .iter()
        .map(|(cs, _is_region_root)| cs.term.clone())
        .collect();

    if node.op == "Bop" && sub_terms.len() < 3 {
        eprintln!("Node: {:?}", node);
        panic!("Expected 3 children for BOp, found {:?}", sub_terms.len());
    }

    let term = extractor.get_term(info, node_id, sub_terms);

    Some(CostSet { total, costs, term })
}

pub fn extract(
    original_prog: &TreeProgram,
    egraph: &egraph_serialize::EGraph,
    unextractables: HashSet<String>,
    termdag: &mut TermDag,
    cost_model: impl CostModel,
) -> (CostSet, TreeProgram) {
    eprintln!("Extracting");
    let egraph_info = EgraphInfo::new(&cost_model, egraph, unextractables);
    let extractor_not_linear = &mut Extractor::new(original_prog, termdag);

    let (cost_res, res) = extract_without_linearity(extractor_not_linear, &egraph_info);
    // TODO implement linearity
    let effectful_nodes_along_path = extractor_not_linear.find_effectful_nodes_in_program(&res);
    let _effectful_regions_along_path = effectful_nodes_along_path
        .into_iter()
        .filter(|nid| egraph_info.is_region_node(nid.clone()))
        .collect::<HashSet<NodeId>>();

    // TODO loop over effectful regions
    // 1) Find reachable nodes in this region
    // 2) Extract sub-regions
    // 3) Extract this region, banning all nodes in effectful regions not on the state edge path
    // 4) extract current region from scratch, sub-regions get cost from previous extraction
    //    a) mark effectful nodes along the path as extractable (just for this region)
    //    b) extract the region

    // To get the type of an e-node, we use the old extractor and query its type

    /*let mut linear_egraph = egraph.clone();
    remove_invalid_effectful_nodes(&mut linear_egraph, &effectful_regions, todo!());

    let extract = &mut Extractor::new(
        &cost_model,
        termdag,
        Default::default(),
        &linear_egraph,
        unextractables,
    );
    let res = extract_without_linearity(extractor_not_linear);*/

    (cost_res, res)
}

/// Perform a greedy extraction of the DAG, without considering linearity.
/// This uses the "fast_greedy_dag" algorithm from the extraction gym.
// pub fn extract_without_linearity(extractor: &mut Extractor) -> CostSet {
//     let n2c = |nid: &NodeId| extractor.egraph.nid_to_cid(nid);
//     let parents = build_parent_index(extractor.egraph);
//     let mut worklist = initialize_worklist(extractor.egraph);

//     while let Some(node_id) = worklist.pop() {
//         let class_id = n2c(&node_id);
//         let node = &extractor.egraph[&node_id];
//         if extractor.unextractables.contains(&node.op) {
//             continue;
//         }
//         if node
//             .children
//             .iter()
//             .all(|c| extractor.costs.contains_key(n2c(c)))
//         {
//             let lookup = extractor.costs.get(class_id);
//             let mut prev_cost: Cost = std::f64::INFINITY.try_into().unwrap();
//             if lookup.is_some() {
//                 prev_cost = lookup.unwrap().total;
//             }

//             let cost_set = calculate_cost_set(extractor.egraph, node_id.clone(), extractor);
//             if cost_set.total < prev_cost {
//                 extractor.costs.insert(class_id.clone(), cost_set);
//                 worklist.extend(parents[class_id].iter().cloned());
//             }
//         }
//     }

//     let mut root_eclasses = extractor.egraph.root_eclasses.clone();
//     root_eclasses.sort();
//     root_eclasses.dedup();

//     let root = get_root(extractor.egraph);
//     extractor.costs.get(n2c(&root)).unwrap().clone()
// }

pub fn extract_without_linearity(
    extractor: &mut Extractor,
    info: &EgraphInfo,
) -> (CostSet, TreeProgram) {
    let n2c = |nid: &NodeId| info.egraph.nid_to_cid(nid);
    let mut worklist = UniqueQueue::default();

    // first, add all the roots to the worklist
    for (root, nodeid) in &info.roots {
        worklist.insert((root.clone(), nodeid.clone()));
    }

    while let Some((rootid, nodeid)) = worklist.pop() {
        let classid = n2c(&nodeid);
        let node = info.egraph.nodes.get(&nodeid).unwrap();
        if info.unextractables.contains(&node.op) {
            continue;
        }

        // create a new region_costs map if it doesn't exist
        let region_costs = extractor.costs.entry(rootid.clone()).or_default();
        let lookup = region_costs.get(classid);
        let mut prev_cost: Cost = std::f64::INFINITY.try_into().unwrap();
        if lookup.is_some() {
            prev_cost = lookup.unwrap().total;
        }

        if let Some(cost_set) = calculate_cost_set(rootid.clone(), nodeid.clone(), extractor, info)
        {
            let region_costs = extractor.costs.get_mut(&rootid).unwrap();
            if cost_set.total < prev_cost {
                region_costs.insert(classid.clone(), cost_set);

                // we updated this eclass's cost, so we need to update its parents
                if let Some(parents) = info.parents.get(&(rootid.clone(), classid.clone())) {
                    for parent in parents {
                        worklist.insert(parent.clone());
                    }
                }
            }
        }
    }

    let root_eclass = n2c(&get_root(info.egraph));
    let root_costset = extractor
        .costs
        .get(root_eclass)
        .expect("Failed to extract program! Also failed to extract any functions in program.")
        .get(root_eclass)
        .expect("Failed to extract program!")
        .clone();

    // now run translation to expressions
    let resulting_prog = extractor.terms_to_expressions(info, root_costset.term.clone());

    (root_costset, resulting_prog)
}

pub trait CostModel {
    /// TODO: we could do better with type info
    fn get_op_cost(&self, op: &str) -> Cost;

    /// if true, the op's children are ignored in calculating the cost
    fn ignore_children(&self, op: &str) -> bool;
}

pub struct DefaultCostModel;

impl CostModel for DefaultCostModel {
    fn get_op_cost(&self, op: &str) -> Cost {
        match op {
            // Leaves
            "Const" => 1.,
            "Arg" => 0.,
            _ if op.parse::<i64>().is_ok() || op.starts_with('"') => 0.,
            "true" | "false" | "()" => 0.,
            // Lists
            "Empty" | "Single" | "Concat" | "Get" | "Nil" | "Cons" => 0.,
            // Types
            "IntT" | "BoolT" | "PointerT" | "StateT" => 0.,
            "Base" | "TupleT" | "TNil" | "TCons" => 0.,
            "Int" | "Bool" => 0.,
            // Algebra
            "Add" | "PtrAdd" | "Sub" | "And" | "Or" | "Not" => 10.,
            "Mul" => 30.,
            "Div" => 50.,
            // Comparisons
            "Eq" | "LessThan" | "GreaterThan" | "LessEq" | "GreaterEq" => 10.,
            // Effects
            "Print" | "Write" | "Load" => 50.,
            "Alloc" | "Free" => 100.,
            "Call" => 1000., // TODO: we could make this more accurate
            // Control
            "Program" | "Function" => 0.,
            "DoWhile" => 100., // TODO: we could make this more accurate
            "If" | "Switch" => 50.,
            // Unreachable
            "HasType" | "HasArgType" | "ContextOf" | "NoContext" | "ExpectType" => 0.,
            "ExprIsPure" | "ListExprIsPure" | "BinaryOpIsPure" | "UnaryOpIsPure" => 0.,
            "IsLeaf" | "BodyContainsExpr" | "ScopeContext" => 0.,
            "Region" | "Full" | "IntB" | "BoolB" => 0.,
            "PathNil" | "PathCons" => 0.,
            // Schema
            "Bop" | "Uop" | "Top" => 0.,
            "InContext" => 0.,
            _ if self.ignore_children(op) => 0.,
            _ => panic!("Please provide a cost for {op}"),
        }
        .try_into()
        .unwrap()
    }

    fn ignore_children(&self, op: &str) -> bool {
        matches!(op, "InLoop" | "NoContext" | "InSwitch" | "InIf")
    }
}

/** A data structure to maintain a queue of unique elements.

Notably, insert/pop operations have O(1) expected amortized runtime complexity.

Thanks @Bastacyclop for the implementation!
*/
#[derive(Clone)]
pub(crate) struct UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    set: HashSet<T>,
    queue: VecDeque<T>,
}

impl<T> Default for UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn default() -> Self {
        UniqueQueue {
            set: Default::default(),
            queue: Default::default(),
        }
    }
}

impl<T> UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn insert(&mut self, t: T) {
        if self.set.insert(t.clone()) {
            self.queue.push_back(t);
        }
    }

    #[allow(dead_code)]
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for t in iter.into_iter() {
            self.insert(t);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let res = self.queue.pop_front();
        res.as_ref().map(|t| self.set.remove(t));
        res
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        let r = self.queue.is_empty();
        debug_assert_eq!(r, self.set.is_empty());
        r
    }
}

// For a given enode,
// return the roots for sub-regions
fn enode_regions(
    egraph: &egraph_serialize::EGraph,
    enode: &egraph_serialize::Node,
) -> Vec<ClassId> {
    enode_children(egraph, enode)
        .iter()
        .filter_map(|(cid, is_region_root)| {
            if *is_region_root {
                Some(cid.clone())
            } else {
                None
            }
        })
        .collect()
}

/// For a given enode, returns a vector of children eclasses.
/// Also, for each child returns if the child is a region root.
fn enode_children(
    egraph: &egraph_serialize::EGraph,
    enode: &egraph_serialize::Node,
) -> Vec<(ClassId, bool)> {
    match (enode.op.as_str(), enode.children.as_slice()) {
        ("DoWhile", [input, body]) => vec![
            (egraph.nid_to_cid(input).clone(), false),
            (egraph.nid_to_cid(body).clone(), true),
        ],
        ("If", [pred, input, then_branch, else_branch]) => vec![
            (egraph.nid_to_cid(pred).clone(), false),
            (egraph.nid_to_cid(input).clone(), false),
            (egraph.nid_to_cid(then_branch).clone(), true),
            (egraph.nid_to_cid(else_branch).clone(), true),
        ],
        ("Switch", [pred, input, branchlist]) => {
            let mut res = vec![
                (egraph.nid_to_cid(pred).clone(), false),
                (egraph.nid_to_cid(input).clone(), false),
            ];
            res.extend(
                get_conslist_children(egraph, egraph.nid_to_cid(branchlist).clone())
                    .into_iter()
                    .map(|cid| (cid, true)),
            );
            res
        }
        ("Function", [name, args, ret, body]) => {
            vec![
                (egraph.nid_to_cid(name).clone(), false),
                (egraph.nid_to_cid(args).clone(), false),
                (egraph.nid_to_cid(ret).clone(), false),
                (egraph.nid_to_cid(body).clone(), true),
            ]
        }
        _ => {
            let mut children = vec![];
            for child in &enode.children {
                children.push((egraph.nid_to_cid(child).clone(), false));
            }
            children
        }
    }
}

fn get_conslist_children(egraph: &egraph_serialize::EGraph, class_id: ClassId) -> Vec<ClassId> {
    // assert that there is only one e-node in the eclass
    let class = egraph.classes()[&class_id].clone();
    assert_eq!(class.nodes.len(), 1);
    let node = egraph[&class.nodes[0]].clone();
    match node.op.as_str() {
        "Nil" => vec![],
        "Cons" => {
            let mut children = vec![egraph.nid_to_cid(&node.children[0]).clone()];
            children.extend(get_conslist_children(
                egraph,
                egraph.nid_to_cid(&node.children[1]).clone(),
            ));
            children
        }
        _ => panic!("Expected Cons or Nil, found {:?}", node.op),
    }
}

fn region_reachable_classes(egraph: &egraph_serialize::EGraph, root: ClassId) -> HashSet<ClassId> {
    let mut visited = HashSet::new();
    let mut queue = UniqueQueue::default();
    queue.insert(root);

    while let Some(eclass) = queue.pop() {
        if visited.insert(eclass.clone()) {
            for node in &egraph.classes()[&eclass].nodes {
                for (child, is_subregion) in enode_children(egraph, &egraph[node]) {
                    if !is_subregion {
                        queue.insert(child);
                    }
                }
            }
        }
    }

    visited
}

#[cfg(test)]
fn dag_extraction_test(prog: &TreeProgram, expected_cost: NotNan<f64>) {
    use crate::{print_with_intermediate_vars, prologue};
    let string_prog = {
        let (term, termdag) = prog.to_egglog();
        let printed = print_with_intermediate_vars(&termdag, term);
        format!("{}\n{printed}\n", prologue(),)
    };

    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(&string_prog).unwrap();
    let (serialized_egraph, unextractables) = serialized_egraph(egraph);
    let mut termdag = TermDag::default();

    let cost_set = extract(
        prog,
        &serialized_egraph,
        unextractables,
        &mut termdag,
        DefaultCostModel,
    );

    assert_eq!(cost_set.0.total, expected_cost);
}

#[test]
fn test_dag_extract() {
    use crate::ast::*;

    let prog = program!(
        function(
            "main",
            tuplet!(intt(), statet()),
            tuplet!(intt(), statet()),
            parallel!(
                add(
                    int(10),
                    get(
                        dowhile(
                            parallel!(getat(0)),
                            push(
                                add(getat(0), int(10)),
                                single(less_than(add(getat(0), int(10)), int(10)))
                            )
                        ),
                        0
                    )
                ),
                getat(1)
            )
        ),
        function(
            "niam",
            tuplet!(intt(), statet()),
            tuplet!(intt(), statet()),
            parallel!(
                add(
                    int(10),
                    get(
                        dowhile(
                            parallel!(get(arg(), 0)),
                            push(
                                add(getat(0), int(10)),
                                single(less_than(add(getat(0), int(10)), int(10)))
                            )
                        ),
                        0
                    )
                ),
                getat(1)
            )
        )
    );
    let cost_model = DefaultCostModel;

    let cost_of_one_func = cost_model.get_op_cost("Add") * 2.
        + cost_model.get_op_cost("DoWhile")
        + cost_model.get_op_cost("LessThan")
        // while the same const is used three times, it is only counted twice
        + cost_model.get_op_cost("Const") * 2.;
    let expected_cost = cost_of_one_func * 2.;
    dag_extraction_test(&prog, expected_cost);
}

#[test]
fn simple_dag_extract() {
    use crate::ast::*;
    let prog = program!(function(
        "main",
        tuplet!(intt(), statet()),
        tuplet!(intt(), statet()),
        parallel!(int(10), getat(1))
    ),);
    let cost_model = DefaultCostModel;

    let expected_cost = cost_model.get_op_cost("Const");
    dag_extraction_test(&prog, expected_cost);
}
