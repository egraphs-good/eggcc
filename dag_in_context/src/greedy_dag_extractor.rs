use egglog::{ast::Literal, util::IndexMap, *};
use egraph_serialize::{ClassId, EGraph, NodeId};
use ordered_float::NotNan;
use rpds::HashTrieMap;
use rustc_hash::FxHashMap;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    f64::INFINITY,
};
use strum::IntoEnumIterator;

use crate::{
    from_egglog::FromEgglog,
    schema::{RcExpr, TreeProgram, Type},
    schema_helpers::Sort,
    typechecker::TypeChecker,
};

type RootId = ClassId;

pub(crate) struct EgraphInfo<'a> {
    pub(crate) egraph: EGraph,
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
    costsets: Vec<CostSet>,
    costsetmemo: FxHashMap<(NodeId, Vec<CostSetIndex>), CostSetIndex>,
    costs: FxHashMap<ClassId, FxHashMap<ClassId, CostSetIndex>>,

    // use to get the type of an expression
    pub(crate) typechecker: TypeChecker<'a>,

    // Each term must correspond to a node in the egraph. We store that here
    // Use an indexmap for deterministic order of iteration
    pub(crate) correspondence: IndexMap<Term, NodeId>,
    // Get the expression corresponding to a term.
    // This is computed after the extraction is done.
    pub(crate) term_to_expr: Option<HashMap<Term, RcExpr>>,
    pub(crate) eclass_type: Option<HashMap<ClassId, Type>>,
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
        egraph: EGraph,
        unextractables: HashSet<String>,
    ) -> Self {
        // get all the roots needed
        let mut region_roots = HashSet::new();
        for node in egraph.classes().values().flat_map(|c| &c.nodes) {
            for root in enode_regions(&egraph, &egraph[node]) {
                region_roots.insert(root);
            }
        }
        // also add the root of the egraph to region_roots
        region_roots.insert(egraph.nid_to_cid(&get_root(&egraph)).clone());

        let mut num_not_expr = 0;
        // find all the (root, child) pairs that are important
        let mut relavent_nodes: Vec<(ClassId, ClassId)> = vec![];
        for root in &region_roots {
            let reachable = region_reachable_classes(&egraph, root.clone(), cm);
            for eclass in reachable {
                // if type is not expr add to count
                if egraph.class_data[&eclass].typ.as_ref().unwrap() != "Expr" {
                    num_not_expr += 1;
                }
                relavent_nodes.push((root.clone(), eclass));
            }
        }

        if relavent_nodes.len() > egraph.classes().len() * 3 {
            eprintln!("Warning: significant sharing between region roots, {}x blowup. May cause bad extraction performance. Eclasses: {}. (Root, eclass) pairs: {}. Region roots: {}. Non-Expr: {}", relavent_nodes.len() / egraph.classes().len(), egraph.classes().len(), relavent_nodes.len(), region_roots.len(), num_not_expr);
        }

        let mut roots = vec![];
        // find all the (root, enode) pairs that are root nodes (no children)
        for (root, eclass) in &relavent_nodes {
            for enode in egraph.classes()[eclass].nodes.iter() {
                if enode_children(&egraph, &egraph[enode]).is_empty() {
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
                let node = &egraph[enode];

                // skip nodes with infinite cost
                if cm.get_op_cost(&node.op).is_infinite() {
                    continue;
                }

                // add to the parents table
                for EnodeChild {
                    child,
                    is_subregion,
                    is_assumption,
                } in enode_children(&egraph, node)
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
    pub(crate) fn is_eclass_effectful(&mut self, class_id: ClassId) -> Option<bool> {
        let node_type = self.eclass_type.as_ref().unwrap().get(&class_id)?;
        Some(node_type.contains_state())
    }

    /// Convert the extracted terms to expressions, and also
    /// store their types.
    fn terms_to_expressions(&mut self, info: &EgraphInfo, prog: Term) -> TreeProgram {
        let mut converter = FromEgglog {
            termdag: self.termdag,
            conversion_cache: Default::default(),
        };
        let mut node_to_type: HashMap<ClassId, Type> = Default::default();

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
                node_to_type.insert(eclass.clone(), ty);
            }
        }

        let converted_prog = converter.program_from_egglog(prog);

        self.eclass_type = Some(node_to_type);
        self.term_to_expr = Some(converter.conversion_cache);

        // return the converted program
        converted_prog
    }

    pub(crate) fn term_node(&self, term: &Term) -> NodeId {
        self.correspondence
            .get(term)
            .unwrap_or_else(|| panic!("Failed to find correspondence for term {:?}", term))
            .clone()
    }

    pub(crate) fn get_dummy_context(
        &mut self,
        info: &EgraphInfo,
        class_id: ClassId,
    ) -> CostSetIndex {
        // get any node in the class
        let node_id = info.egraph.classes()[&class_id].nodes.first().unwrap();
        if let Some(existing) = self.costsetmemo.get(&(node_id.clone(), vec![])) {
            *existing
        } else {
            let dummy = self
                .termdag
                .lit(Literal::String(format!("dummy{}", node_id).into()));
            Self::add_correspondence(&mut self.correspondence, dummy.clone(), node_id.clone());
            let term = self.termdag.app("InFunc".into(), vec![dummy]);
            Self::add_correspondence(&mut self.correspondence, term.clone(), node_id.clone());
            let costset = CostSet {
                costs: Default::default(),
                total: 0.0.try_into().unwrap(),
                term,
            };
            self.costsets.push(costset);
            self.costsetmemo
                .insert((node_id.clone(), vec![]), self.costsets.len() - 1);
            self.costsets.len() - 1
        }
    }

    pub(crate) fn new(original_prog: &'a TreeProgram, termdag: &'a mut TermDag) -> Self {
        Extractor {
            termdag,
            costsets: Default::default(),
            costsetmemo: Default::default(),
            correspondence: Default::default(),
            costs: Default::default(),
            term_to_expr: Default::default(),
            typechecker: TypeChecker::new(original_prog, true),
            eclass_type: Default::default(),
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
type CostSetIndex = usize;

#[derive(Clone, Debug)]
pub struct CostSet {
    /// Total cost of this term, taking sharing into account
    pub total: Cost,
    /// Maps classes to the chosen term for the eclass,
    /// along with the cost for that term (excluding child costs).
    pub costs: HashTrieMap<ClassId, (Term, Cost)>,
    /// The resulting term
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

        Self::add_correspondence(&mut self.correspondence, term.clone(), node_id.clone());

        term
    }

    fn add_correspondence(
        correspondence: &mut IndexMap<Term, NodeId>,
        term: Term,
        node_id: NodeId,
    ) {
        if let Some(existing) = correspondence.insert(term.clone(), node_id.clone()) {
            assert_eq!(existing, node_id);
        }
    }

    /// Add `term` to `current_costs`, returning
    /// 1) a new term that takes advantage of sharing
    /// with respect to `current_costs` and
    /// 2) the net cost of adding the new term
    fn add_term_to_cost_set(
        &self,
        info: &EgraphInfo,
        correspondance: &mut IndexMap<Term, NodeId>,
        termdag: &mut TermDag,
        current_costs: &mut HashTrieMap<ClassId, (Term, Cost)>,
        term: Term,
        other_costs: &HashTrieMap<ClassId, (Term, Cost)>,
    ) -> (Term, Cost) {
        let nodeid = &self.term_node(&term);
        let eclass = info.egraph.nid_to_cid(nodeid);
        if let Some(existing) = current_costs.get(eclass) {
            existing.clone()
        } else {
            let unshared_cost = match other_costs.get(eclass) {
                Some((_, cost)) => *cost,
                None => NotNan::new(0.).unwrap(),
            };
            let mut cost = unshared_cost;
            let new_term = match term {
                Term::App(head, children) => {
                    let mut new_children = vec![];
                    for child in children {
                        let child = termdag.get(child);
                        let (new_child, child_cost) = self.add_term_to_cost_set(
                            info,
                            correspondance,
                            termdag,
                            current_costs,
                            child,
                            other_costs,
                        );
                        new_children.push(new_child);
                        cost += child_cost;
                    }
                    termdag.app(head, new_children)
                }
                _ => term,
            };
            Self::add_correspondence(correspondance, new_term.clone(), nodeid.clone());
            *current_costs =
                current_costs.insert(eclass.clone(), (new_term.clone(), unshared_cost));

            (new_term, cost)
        }
    }

    fn calculate_cost_set(
        &mut self,
        nodeid: NodeId,
        child_cost_set_indecies: Vec<CostSetIndex>,
        info: &EgraphInfo,
    ) -> Option<CostSetIndex> {
        if let Some(&idx) = self
            .costsetmemo
            .get(&(nodeid.clone(), child_cost_set_indecies.clone()))
        {
            return Some(idx);
        }
        let cid = info.egraph.nid_to_cid(&nodeid);
        let node = &info.egraph[&nodeid];

        let child_cost_sets = child_cost_set_indecies
            .iter()
            .map(|idx| &self.costsets[*idx])
            .zip(
                enode_children(&info.egraph, node)
                    .iter()
                    .map(|c| c.is_subregion),
            )
            .collect::<Vec<_>>();
        // cycle detection
        if child_cost_sets
            .iter()
            .any(|(cs, _)| cs.costs.contains_key(cid))
        {
            return None;
        }

        let mut shared_total = NotNan::new(0.).unwrap();
        let mut unshared_total = info.cm.get_op_cost(&node.op);
        let mut costs: HashTrieMap<ClassId, (Term, Cost)> = Default::default();
        let index_of_biggest_child = child_cost_sets
            .iter()
            .enumerate()
            .max_by_key(
                |(_idx, (cs, is_region_root))| {
                    if *is_region_root {
                        0
                    } else {
                        cs.costs.size()
                    }
                },
            )
            .map(|(idx, _)| idx);
        if let Some(index_of_biggest_child) = index_of_biggest_child {
            let (biggest_child_set, is_region_root) = &child_cost_sets[index_of_biggest_child];
            if !is_region_root {
                costs = biggest_child_set.costs.clone();
                shared_total = biggest_child_set.total;
            }
        }

        let mut children_terms = vec![];
        let mut termdag_tmp = TermDag::default();
        let mut new_correspondence = IndexMap::default();
        // swap out the termdag and correspondance for the temporary one
        std::mem::swap(self.termdag, &mut termdag_tmp);

        if !info.cm.ignore_children(&node.op) {
            for (index, (child_set, is_region_root)) in child_cost_sets.iter().enumerate() {
                if *is_region_root {
                    children_terms.push(child_set.term.clone());
                    unshared_total += child_set.total;
                } else {
                    // costs is empty, replace it with the child one
                    if Some(index) == index_of_biggest_child {
                        // skip the biggest child's cost
                        children_terms.push(child_set.term.clone());
                    } else {
                        let (child_term, net_cost) = self.add_term_to_cost_set(
                            info,
                            &mut new_correspondence,
                            &mut termdag_tmp,
                            &mut costs,
                            child_set.term.clone(),
                            &child_set.costs,
                        );
                        shared_total += net_cost;
                        children_terms.push(child_term);
                    }
                }
            }
        }

        // swap back the termdag and correspondance
        std::mem::swap(self.termdag, &mut termdag_tmp);

        // add the new correspondence to the main correspondence
        for (term, nodeid) in new_correspondence {
            Self::add_correspondence(&mut self.correspondence, term, nodeid);
        }

        let term = self.get_term(info, nodeid.clone(), children_terms);

        // no need to add something that costs 0 to the set
        if unshared_total > NotNan::new(0.).unwrap() {
            costs = costs.insert(cid.clone(), (term.clone(), unshared_total));
        }
        let total = unshared_total + shared_total;

        self.costsets.push(CostSet { total, costs, term });
        let index = self.costsets.len() - 1;
        self.costsetmemo
            .insert((nodeid, child_cost_set_indecies), index);

        Some(index)
    }
}

/// Calculates the cost set of a node based on cost sets of children.
/// Handles cycles by returning a cost set with infinite cost.
/// Returns None when costs for children are not yet available.
fn calculate_node_cost_set(
    rootid: ClassId,
    node_id: NodeId,
    extractor: &mut Extractor,
    info: &EgraphInfo,
) -> Option<CostSetIndex> {
    let node = &info.egraph[&node_id];

    // get the cost sets for the children
    let child_cost_sets = enode_children(&info.egraph, node)
        .iter()
        .filter_map(
            |EnodeChild {
                 child,
                 is_subregion,
                 is_assumption,
             }| {
                // for assumptions, just return a dummy context every time
                if *is_assumption {
                    Some(extractor.get_dummy_context(info, child.clone()))
                } else if *is_subregion {
                    extractor.costs.get(child)?.get(child).copied()
                } else {
                    let region_costs = extractor.costs.get(&rootid).unwrap();
                    region_costs.get(child).copied()
                }
            },
        )
        .collect::<Vec<_>>();
    // if any are unavailable, we return none from this whole function
    if child_cost_sets.len() < node.children.len() {
        return None;
    }

    extractor.calculate_cost_set(node_id, child_cost_sets, info)
}

pub fn extract(
    original_prog: &TreeProgram,
    egraph: egraph_serialize::EGraph,
    unextractables: HashSet<String>,
    termdag: &mut TermDag,
    cost_model: impl CostModel,
) -> (CostSet, TreeProgram) {
    let egraph_info = EgraphInfo::new(&cost_model, egraph, unextractables);
    let extractor_not_linear = &mut Extractor::new(original_prog, termdag);

    let (_cost_res, res) = extract_with_paths(extractor_not_linear, &egraph_info, None);

    let effectful_nodes_along_path =
        extractor_not_linear.find_effectful_nodes_in_program(&res, &egraph_info);
    extractor_not_linear.costs.clear();
    let (cost_res, res) = extract_with_paths(
        extractor_not_linear,
        &egraph_info,
        Some(&effectful_nodes_along_path),
    );
    extractor_not_linear.check_program_is_linear(&res).unwrap();

    log::info!("Extracted program with cost {}", cost_res.total);
    log::info!("Created {} cost sets", extractor_not_linear.costsets.len());

    (cost_res, res)
}

pub fn extract_with_paths(
    extractor: &mut Extractor,
    info: &EgraphInfo,
    // If effectful paths are present,
    // for each region we will only consider
    // effectful nodes that are in effectful_path[rootid]
    effectful_paths: Option<&HashMap<ClassId, HashSet<NodeId>>>,
) -> (CostSet, TreeProgram) {
    if effectful_paths.is_some() {
        log::info!("Re-extracting program after linear path is found.");
    } else {
        log::info!("Extracting program for the first time.");
    }
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
        // if node is effectful, we only consider it if it is in the effectful path
        if sort_of_node == "Expr" && effectful_paths.is_some() {
            let effectful_lookup = extractor.is_eclass_effectful(classid.clone());
            if effectful_lookup.is_none() && node.op != "Function" {
                // skip when type is unknown
                continue;
            }
            if let Some(true) = effectful_lookup {
                let effectful_nodes = effectful_paths.unwrap().get(&rootid);
                if effectful_nodes.is_none() {
                    // continue when this root isn't in effectful_paths
                    continue;
                }

                // skip nodes not on the path
                if !effectful_nodes.unwrap().contains(&nodeid) {
                    continue;
                }
            }
        }

        // create a new region_costs map if it doesn't exist
        let region_costs = extractor.costs.entry(rootid.clone()).or_default();
        let lookup = region_costs.get(classid);
        let mut prev_cost: Cost = std::f64::INFINITY.try_into().unwrap();
        if let Some(lookup) = lookup {
            let costset = extractor.costsets.get(*lookup).unwrap();
            prev_cost = costset.total;
        }

        if let Some(cost_set_index) =
            calculate_node_cost_set(rootid.clone(), nodeid.clone(), extractor, info)
        {
            let cost_set = &extractor.costsets[cost_set_index];
            let region_costs = extractor.costs.get_mut(&rootid).unwrap();
            if cost_set.total < prev_cost {
                region_costs.insert(classid.clone(), cost_set_index);

                // we updated this eclass's cost, so we need to update its parents
                if let Some(parents) = info.parents.get(&(rootid.clone(), classid.clone())) {
                    for parent in parents {
                        worklist.insert(parent.clone());
                    }
                }
            }
        }
    }

    let root_eclass = n2c(&get_root(&info.egraph));

    let root_costset_index = *extractor
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
        });
    let root_costset = extractor.costsets[root_costset_index].clone();

    // now run translation to expressions
    let resulting_prog = extractor.terms_to_expressions(info, root_costset.term.clone());

    let root_cost = root_costset.total;
    if root_cost.is_infinite() {
        panic!("Failed to extract program! Found infinite cost on result node.");
    }
    if root_cost.is_sign_negative() {
        panic!("Failed to extract program! Found negative cost on result node.");
    }

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
            // Schema
            "Bop" | "Uop" | "Top" => 0.,
            _ => INFINITY,
        }
        .try_into()
        .unwrap()
    }

    fn ignore_children(&self, op: &str) -> bool {
        matches!(op, "InLoop" | "InSwitch" | "InIf" | "InFunc")
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

fn type_is_part_of_ast(ty: &str) -> bool {
    for sort in Sort::iter() {
        if sort.name() == ty {
            return true;
        }
    }
    false
}

/// Reachable eclasses in the same region as the root.
/// Does not include subregions, assumptions, or anything that does not have the correct type.
fn region_reachable_classes(
    egraph: &egraph_serialize::EGraph,
    root: ClassId,
    cm: &dyn CostModel,
) -> HashSet<ClassId> {
    let mut visited = HashSet::new();
    let mut queue = UniqueQueue::default();
    queue.insert(root);

    while let Some(eclass) = queue.pop() {
        let eclass_type = egraph.class_data[&eclass].typ.as_ref().unwrap();
        if eclass_type == "Assumption" {
            panic!("Found assumption in region reachable classes");
        }

        if !type_is_part_of_ast(eclass_type) {
            continue;
        }
        if visited.insert(eclass.clone()) {
            for node in &egraph.classes()[&eclass].nodes {
                // skip nodes with infinite cost
                if cm.get_op_cost(&egraph[node].op).is_infinite() {
                    continue;
                }

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
        serialized_egraph,
        unextractables,
        &mut termdag,
        DefaultCostModel,
    );

    assert_eq!(cost_set.0.total, expected_cost);
}

/// This only runs extract_without_linearity once
/// and check if the extracted program violates linearity.
#[cfg(test)]
fn dag_extraction_linearity_check(prog: &TreeProgram, error_message: &str) {
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

    let egraph_info = EgraphInfo::new(&DefaultCostModel, serialized_egraph, unextractables);
    let extractor_not_linear = &mut Extractor::new(prog, &mut termdag);

    let (_cost_res, prog) = extract_with_paths(extractor_not_linear, &egraph_info, None);
    let res = extractor_not_linear.check_program_is_linear(&prog);
    match res {
        Ok(_) => panic!("Expected program to be non-linear!"),
        Err(e) => assert!(e.starts_with(error_message)),
    }
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

#[test]
fn test_linearity_check_1() {
    use crate::ast::*;

    let bad_program_1 = program!(function(
        "main",
        tuplet!(intt(), statet()),
        tuplet!(intt(), statet()),
        parallel!(
            add(
                int(10),
                get(
                    dowhile(
                        parallel!(getat(0), getat(1)),
                        parallel!(
                            less_than(add(getat(0), int(10)), int(10)),
                            add(getat(0), int(10)),
                            getat(1),
                        )
                    ),
                    0
                )
            ),
            getat(1)
        )
    ),);
    dag_extraction_linearity_check(
        &bad_program_1,
        "Resulting program violated linearity! Effectful",
    );
}

#[test]
fn test_linearity_check_2() {
    use crate::ast::*;

    let bad_program_2 = program!(function(
        "main",
        tuplet!(intt(), statet()),
        tuplet!(intt()),
        parallel!(tif(
            ttrue(),
            parallel!(getat(0), getat(1)),
            getat(0),
            getat(0)
        ))
    ),);
    dag_extraction_linearity_check(
        &bad_program_2,
        "Resulting program violated linearity! There are unconsumed effectful operators.",
    );
}

///                                                    
///         val1  state1     val2  state2              
///           │       │        │      │                
///          c│      e│       e│     c│                
///          h│      x│       x│     h│                
///          e│      p│       p│     e│                
///          a│       │        │     a│                
///          p│       │        │     p│                
///       ┌───┴───────┴──┐ ┌───┴──────┴───┐            
///       │              │ │              │            
///       │              │ │              │            
///       │              │ │              │            
///       │  region1     │ │    region2   │            
///       │              │ │              │            
///       │              │ │              │            
///       │              │ │              │            
///       │              │ │              │            
///       └──────────────┘ └──────────────┘            
/// where val1 = val2, state1 = state2, cost(region1) = cost(region2)
#[test]
fn test_validity_of_extraction() {
    use crate::ast::*;
    use crate::{print_with_intermediate_vars, prologue};

    let region_1 = tif(
        ttrue(),
        parallel!(getat(0)),
        parallel!(int(0), getat(0)),
        parallel!(int(0), getat(0)),
    )
    .with_arg_types(tuplet!(statet()), tuplet!(intt(), statet()));
    let cheap_value_path = get(region_1.clone(), 0).with_arg_types(tuplet!(statet()), base(intt()));
    let expensive_state_path = {
        let alloc_expr = alloc(1, int(1000), get(region_1, 1), pointert(intt()));
        free(get(alloc_expr.clone(), 0), get(alloc_expr, 1))
            .with_arg_types(tuplet!(statet()), base(statet()))
    };
    let region_2 = tif(
        tfalse(),
        parallel!(getat(0)),
        parallel!(int(0), getat(0)),
        parallel!(int(0), getat(0)),
    )
    .with_arg_types(tuplet!(statet()), tuplet!(intt(), statet()));

    let expensive_value_path = div(mul(get(region_2.clone(), 0), int(10)), int(10))
        .with_arg_types(tuplet!(statet()), base(intt()));
    let cheap_state_path = get(region_2, 1).with_arg_types(tuplet!(statet()), base(statet()));

    let decl = format!(
        "(let cheap-value-path {})
         (let expensive-state-path {})
         (let expensive-value-path {})
         (let cheap-state-path {})
         (union cheap-value-path expensive-value-path)
         (union expensive-state-path cheap-state-path)",
        cheap_value_path, expensive_state_path, expensive_value_path, cheap_state_path,
    );

    let prog = program!(function(
        "main",
        tuplet!(statet()),
        tuplet!(intt(), statet()),
        parallel!(expensive_value_path, cheap_state_path,)
    ),);

    let string_prog = {
        let (term, termdag) = prog.to_egglog();
        let printed = print_with_intermediate_vars(&termdag, term);
        format!("{}\n{}\n{}", prologue(), decl, printed)
    };

    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(&string_prog).unwrap();
    let (serialized_egraph, unextractables) = serialized_egraph(egraph);
    let mut termdag = TermDag::default();

    let egraph_info = EgraphInfo::new(
        &DefaultCostModel,
        serialized_egraph.clone(),
        unextractables.clone(),
    );
    let extractor_not_linear = &mut Extractor::new(&prog, &mut termdag);

    let (_cost_res, res) = extract_with_paths(extractor_not_linear, &egraph_info, None);
    // first extraction should fail linearity check
    assert!(extractor_not_linear.check_program_is_linear(&res).is_err());

    // second extraction should succeed
    extract(
        &prog,
        serialized_egraph,
        unextractables,
        &mut termdag,
        DefaultCostModel,
    );
}
