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
    pub(crate) fn get_sort_of_eclass(&self, eclass: &ClassId) -> &String {
        self.egraph
            .class_data
            .get(eclass)
            .unwrap()
            .typ
            .as_ref()
            .unwrap()
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
                for EnodeChild {
                    child,
                    is_subregion,
                    is_assumption,
                } in enode_children(egraph, &egraph[enode])
                {
                    if is_assumption {
                        continue;
                    }
                    let child_region = if is_subregion {
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

    /// If the type of the node is known, checks if an already extracted node is effectful.
    /// There are cases where the type of the node is not known, for reasons unknown to us.
    pub(crate) fn is_node_effectful(&mut self, node_id: NodeId) -> Option<bool> {
        let node_type = self.node_to_type.as_ref().unwrap().get(&node_id)?;
        Some(node_type.contains_state())
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
            let sort_of_eclass = info.get_sort_of_eclass(eclass);
            // only convert expressions (that are not functions)
            if sort_of_eclass == "Expr" && node.op != "Function" {
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

pub(crate) fn get_root(egraph: &egraph_serialize::EGraph) -> NodeId {
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

    let noctx = CostSet {
        costs: Default::default(),
        total: 0.0.try_into().unwrap(),
        term: extractor.termdag.app("NoContext".into(), vec![]),
    };

    // get the cost sets for the children
    let child_cost_sets = enode_children(info.egraph, node)
        .iter()
        .filter_map(
            |EnodeChild {
                 child,
                 is_subregion,
                 is_assumption,
             }| {
                // for assumptions, just return (NoContext) every time
                if *is_assumption {
                    Some((&noctx, *is_subregion))
                } else if *is_subregion {
                    Some((extractor.costs.get(child)?.get(child)?, *is_subregion))
                } else {
                    region_costs
                        .get(child)
                        .map(|cost_set| (cost_set, *is_subregion))
                }
            },
        )
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
    let egraph_info = EgraphInfo::new(&cost_model, egraph, unextractables);
    let extractor_not_linear = &mut Extractor::new(original_prog, termdag);

    let (_cost_res, res) = extract_without_linearity(extractor_not_linear, &egraph_info, None);
    let effectful_nodes_along_path =
        extractor_not_linear.find_effectful_nodes_in_program(&res, &egraph_info);
    extractor_not_linear.costs.clear();
    let (cost_res, res) = extract_without_linearity(
        extractor_not_linear,
        &egraph_info,
        Some(&effectful_nodes_along_path),
    );
    (cost_res, res)
}

pub fn extract_without_linearity(
    extractor: &mut Extractor,
    info: &EgraphInfo,
    // If effectful paths are present,
    // for each region we will only consider
    // effectful nodes that are in effectful_path[rootid]
    effectful_paths: Option<&HashMap<ClassId, HashSet<NodeId>>>,
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

        let sort_of_node = info.get_sort_of_eclass(classid);
        if sort_of_node == "Expr"
            && effectful_paths.is_some()
            && effectful_paths.unwrap().contains_key(&rootid)
        {
            let effectful_nodes = effectful_paths.as_ref().unwrap().get(&rootid).unwrap();
            if let Some(is_stateful) = extractor.is_node_effectful(nodeid.clone()) {
                if is_stateful && !effectful_nodes.contains(&nodeid) {
                    continue;
                }
            }
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
        .unwrap_or_else(|| {
            if effectful_paths.is_some() {
                panic!("Failed to extract program after linear path is found!");
            } else {
                panic!("Failed to extract program during initial extraction!");
            }
        })
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
            "BodyContainsExpr" | "ScopeContext" => 0.,
            "Region" | "Full" | "IntB" | "BoolB" => 0.,
            "PathNil" | "PathCons" => 0.,
            // Schema
            "Bop" | "Uop" | "Top" => 0.,
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
        .filter_map(
            |EnodeChild {
                 child,
                 is_subregion,
                 ..
             }| {
                if *is_subregion {
                    Some(child.clone())
                } else {
                    None
                }
            },
        )
        .collect()
}

struct EnodeChild {
    child: ClassId,
    is_subregion: bool,
    is_assumption: bool,
}

impl EnodeChild {
    fn new(child: ClassId, is_subregion: bool, is_assumption: bool) -> Self {
        EnodeChild {
            child,
            is_subregion,
            is_assumption,
        }
    }
}

/// For a given enode, returns a vector of children eclasses.
/// Also, for each child returns if the child is a region root.
fn enode_children(
    egraph: &egraph_serialize::EGraph,
    enode: &egraph_serialize::Node,
) -> Vec<EnodeChild> {
    match (enode.op.as_str(), enode.children.as_slice()) {
        ("DoWhile", [input, body]) => vec![
            EnodeChild::new(egraph.nid_to_cid(input).clone(), false, false),
            EnodeChild::new(egraph.nid_to_cid(body).clone(), true, false),
        ],
        ("If", [pred, input, then_branch, else_branch]) => vec![
            EnodeChild::new(egraph.nid_to_cid(pred).clone(), false, false),
            EnodeChild::new(egraph.nid_to_cid(input).clone(), false, false),
            EnodeChild::new(egraph.nid_to_cid(then_branch).clone(), true, false),
            EnodeChild::new(egraph.nid_to_cid(else_branch).clone(), true, false),
        ],
        ("Switch", [pred, input, branchlist]) => {
            let mut res = vec![
                EnodeChild::new(egraph.nid_to_cid(pred).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(input).clone(), false, false),
            ];
            res.extend(
                get_conslist_children(egraph, egraph.nid_to_cid(branchlist).clone())
                    .into_iter()
                    .map(|cid| EnodeChild::new(cid, true, false)),
            );
            res
        }
        ("Function", [name, args, ret, body]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(name).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(args).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(ret).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(body).clone(), true, false),
            ]
        }
        ("Arg", [ty, ctx]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(ty).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(ctx).clone(), false, true),
            ]
        }
        ("Const", [c, ty, ctx]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(c).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(ty).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(ctx).clone(), false, true),
            ]
        }
        ("Empty", [ty, ctx]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(ty).clone(), false, false),
                EnodeChild::new(egraph.nid_to_cid(ctx).clone(), false, true),
            ]
        }
        _ => {
            let mut children = vec![];
            for child in &enode.children {
                children.push(EnodeChild::new(
                    egraph.nid_to_cid(child).clone(),
                    false,
                    false,
                ));
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
                for EnodeChild {
                    child,
                    is_subregion,
                    is_assumption,
                } in enode_children(egraph, &egraph[node])
                {
                    if !is_subregion && !is_assumption {
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
