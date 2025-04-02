use egglog::{ast::Literal, *};
use egraph_serialize::{ClassId, EGraph, Node, NodeId};
use indexmap::{IndexMap, IndexSet};
use ordered_float::{NotNan, OrderedFloat};
use rpds::HashTrieMap;
use smallvec::SmallVec;
use std::{
    cmp::{max, min, Ordering},
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    f64::INFINITY,
    rc::Rc,
    time::{Duration, Instant},
};
use strum::IntoEnumIterator;

use crate::{
    fastercbcextractor::FasterCbcExtractorWithTimeout,
    from_egglog::FromEgglog,
    schema::{Expr, RcExpr, TreeProgram, Type},
    schema_helpers::Sort,
    typechecker::TypeChecker,
    ExtractionTimeSample,
};

type RootId = ClassId;

pub(crate) struct EgraphInfo<'a> {
    pub(crate) egraph: &'a EGraph,
    pub(crate) _func: String,
    // For every (root, eclass) pair, store the parent
    // (root, enode) pairs that may depend on it.
    pub(crate) parents: IndexMap<(RootId, ClassId), Vec<(RootId, NodeId)>>,
    pub(crate) roots: Vec<(RootId, NodeId)>,
    pub(crate) cm: &'a dyn CostModel,
    /// Optionally, a loop with (inputs, outputs) can have an estimated number of iterations.
    /// This is found by looking at LoopNumItersGuess in the database.
    pub(crate) loop_iteration_estimates: IndexMap<(RootId, RootId), i64>,
    /// A set of names of functions that are unextractable
    unextractables: IndexSet<String>,
    /// A set of (func args) of calls that have been inlined, to indicate we shouldn't
    /// extract the corresponding (Call func args).
    inlined_calls: IndexSet<(ClassId, ClassId)>,
}

pub(crate) struct Extractor<'a> {
    pub(crate) termdag: &'a mut TermDag,
    costsets: Vec<CostSet>,
    costsetmemo: IndexMap<Term, CostSetIndex>,
    /// For a given region (based on the region's root),
    /// stores a map from classes in that region to the chosen term for the eclass.
    /// Regions are extracted separately since different regions
    /// have different allowed state edge paths.
    costs: IndexMap<ClassId, IndexMap<ClassId, CostSetIndex>>,

    // use to get the type of an expression
    pub(crate) typechecker: TypeChecker<'a>,

    // Each term must correspond to a node in the egraph.
    // This allows us to recover the node from the term for banning nodes outside
    // the stateful path.
    pub(crate) correspondence: IndexMap<Term, NodeId>,
    // Get the expression corresponding to a term.
    // This is computed after the extraction is done.
    pub(crate) term_to_expr: IndexMap<Term, RcExpr>,
    pub(crate) eclass_type: Option<IndexMap<ClassId, Type>>,
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

    pub(crate) fn n2c(&self, nid: &NodeId) -> ClassId {
        self.egraph.nid_to_cid(nid).clone()
    }

    fn get_loop_iteration_estimates(egraph: &EGraph) -> IndexMap<(ClassId, ClassId), i64> {
        // for every eclass that represents a single i64 in the egraph,
        // map the eclass to that integer
        let mut integers: IndexMap<ClassId, i64> = IndexMap::default();
        for (nodeid, node) in &egraph.nodes {
            if let Ok(integer) = node.op.parse::<i64>() {
                let eclass = egraph.nid_to_cid(nodeid);
                integers.insert(eclass.clone(), integer);
            }
        }

        let mut loop_iteration_estimates = IndexMap::default();

        // loop over all nodes, finding LoopNumItersGuess nodes
        for (_nodeid, node) in &egraph.nodes {
            if node.op == "LoopNumItersGuess" {
                // assert it has two children
                assert_eq!(
                    node.children.len(),
                    2,
                    "LoopNumItersGuess node has wrong number of children. Node: {:?}",
                    node
                );
                loop_iteration_estimates.insert(
                    (
                        egraph.nid_to_cid(&node.children[0]).clone(),
                        egraph.nid_to_cid(&node.children[1]).clone(),
                    ),
                    integers[&node.eclass],
                );
            }
        }
        loop_iteration_estimates
    }

    fn get_inlined_calls(egraph: &EGraph) -> IndexSet<(ClassId, ClassId)> {
        let mut inlined_calls = IndexSet::new();

        // loop over all nodes, finding InlinedCall nodes
        for (_nodeid, node) in &egraph.nodes {
            if node.op == "InlinedCall" {
                assert_eq!(
                    node.children.len(),
                    2,
                    "InlinedCall node has wrong number of children. Node: {:?}",
                    node
                );
                inlined_calls.insert((
                    egraph.nid_to_cid(&node.children[0]).clone(),
                    egraph.nid_to_cid(&node.children[1]).clone(),
                ));
            }
        }

        inlined_calls
    }

    pub(crate) fn new(
        func: &str,
        func_root: ClassId,
        cm: &'a dyn CostModel,
        egraph: &'a EGraph,
        unextractables: IndexSet<String>,
    ) -> Self {
        let loop_iteration_estimates = Self::get_loop_iteration_estimates(egraph);
        let inlined_calls = Self::get_inlined_calls(egraph);

        // get all the roots needed
        let mut region_roots = find_reachable(egraph, func_root.clone(), cm, false, true);
        // also add the function as a root
        region_roots.insert(func_root);

        log::info!("Found {} regions", region_roots.len());

        let mut num_not_expr = 0;
        // find all the (root, child) pairs that are important
        let mut relavent_eclasses: Vec<(ClassId, ClassId)> = vec![];
        for root in region_roots.iter() {
            let reachable = find_reachable(egraph, root.clone(), cm, true, false);
            for eclass in reachable {
                // if type is not expr add to count
                if egraph.class_data[&eclass].typ.as_ref().unwrap() != "Expr" {
                    num_not_expr += 1;
                }
                relavent_eclasses.push((root.clone(), eclass));
            }
        }

        log::info!("Found {} relavent eclasses", relavent_eclasses.len());
        if relavent_eclasses.len() > egraph.classes().len() * 3 {
            eprintln!("Warning: significant sharing between region roots, {}x blowup. May cause bad extraction performance. Eclassnes: {}. (Root, eclass) pairs: {}. Region roots: {}. Non-Expr: {}", relavent_eclasses.len() / egraph.classes().len(), egraph.classes().len(), relavent_eclasses.len(), region_roots.len(), num_not_expr);
        }

        let mut roots = vec![];
        // find all the (root, enode) pairs that are root nodes (no children)
        for (root, eclass) in &relavent_eclasses {
            for enode in egraph.classes()[eclass].nodes.iter() {
                if enode_children(egraph, &egraph[enode]).is_empty() {
                    roots.push((root.clone(), enode.clone()));
                }
            }
        }

        // sort roots for determinism
        roots.sort();
        log::info!("Found {} roots", roots.len());

        let mut parents: IndexMap<(RootId, ClassId), IndexSet<(RootId, NodeId)>> =
            IndexMap::default();
        for (root, eclass) in relavent_eclasses {
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
                    is_if_inputs: _is_inputs,
                } in enode_children(egraph, node)
                {
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
        log::info!(
            "Found {} parents entries",
            parents.values().map(|v| v.len()).sum::<usize>()
        );

        let mut parents_sorted = IndexMap::default();
        for (key, parents) in parents {
            let mut parents_vec = parents.into_iter().collect::<Vec<_>>();
            parents_vec.sort();
            parents_sorted.insert(key, parents_vec);
        }

        EgraphInfo {
            cm,
            _func: func.to_string(),
            egraph,
            unextractables,
            parents: parents_sorted,
            roots,
            loop_iteration_estimates,
            inlined_calls,
        }
    }
}

impl<'a> Extractor<'a> {
    pub(crate) fn term_to_expr(&mut self, term: &Term) -> RcExpr {
        let mut converter = FromEgglog {
            termdag: self.termdag,
            conversion_cache: Default::default(),
        };
        std::mem::swap(&mut self.term_to_expr, &mut converter.conversion_cache);
        let converted_prog = converter.expr_from_egglog(term.clone());

        self.term_to_expr = converter.conversion_cache;
        converted_prog
    }

    pub(crate) fn typecheck_term(&mut self, term: &Term) -> Type {
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
    fn compute_eclass_types(&mut self, info: &EgraphInfo, prog: Term) -> RcExpr {
        let res = self.term_to_expr(&prog);

        let mut node_to_type: IndexMap<ClassId, Type> = Default::default();

        for (term, node_id) in &self.correspondence.clone() {
            let node = info.egraph.nodes.get(node_id).unwrap();
            let eclass = info.egraph.nid_to_cid(node_id);
            let sort_of_eclass = info.get_sort_of_eclass(eclass);
            // only convert expressions (that are not functions)
            if sort_of_eclass == "Expr" && node.op != "Function" {
                let ty = self.typecheck_term(term);
                node_to_type.insert(eclass.clone(), ty);
            }
        }

        self.eclass_type = Some(node_to_type);

        // return the converted program
        res
    }

    pub(crate) fn term_node(&self, term: &Term) -> NodeId {
        self.correspondence
            .get(term)
            .unwrap_or_else(|| panic!("Failed to find correspondence for term {:?}", term))
            .clone()
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

pub(crate) fn get_root(egraph: &egraph_serialize::EGraph, func: &str) -> NodeId {
    let root_nodes = egraph
        .nodes
        .iter()
        .filter(|(_nid, node)| node.op == "Function");
    let mut found = root_nodes.filter(|(_nid, node)| {
        let child_id = node.children[0].clone();
        let child_str = &egraph.nodes[&child_id].op;
        // remove extra quotes from child_str
        assert!(child_str.starts_with('\"') && child_str.ends_with('\"'));
        let child_str = &child_str[1..child_str.len() - 1];
        child_str == func
    });
    let res = found.next().unwrap();
    assert!(found.next().is_none());
    res.0.clone()
}

pub fn get_unextractables(egraph: &egglog::EGraph) -> IndexSet<String> {
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
) -> (egraph_serialize::EGraph, IndexSet<String>) {
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
    pub args_used: HashSet<usize>,
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
            } else if let Ok(f) = op.parse::<f64>() {
                self.termdag.lit(ast::Literal::Float(OrderedFloat::from(f)))
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

        self.add_correspondence(term.clone(), node_id.clone());

        term
    }

    fn add_correspondence(&mut self, term: Term, node_id: NodeId) {
        if let Some(existing) = self.correspondence.insert(term.clone(), node_id.clone()) {
            assert_eq!(existing, node_id, "Congruence invariant violated! Found two different nodes for the same term. Perhaps we used delete in egglog, which could cause this problem. Term: {:?}", term);
        }
    }

    /// Add `term` to `current_costs`, returning
    /// 1) a new term that takes advantage of sharing
    /// with respect to `current_costs` and
    /// 2) the net cost of adding the new term
    ///
    /// Ex:
    /// Suppose we are extracting a term `Add(a, Neg(b))` where
    /// a and Neg(b) were extracted from sub-eclasses separately.
    /// However, a and b could have the same eclass if they are equal, but we chose different terms,
    /// violating the invariant that we only extract one term per eclass.
    /// This function would be called with `Neg(b)` as `term`, and would return `Neg(a)` as the new term.
    /// This restores the invariant that we only extract one term per eclass.
    ///
    /// When is_free is true, return 0 for the cost and don't add new nodes to the cost set.
    fn add_term_to_cost_set(
        &mut self,
        info: &EgraphInfo,
        current_costs: &mut HashTrieMap<ClassId, (Term, Cost)>,
        term: Term,
        other_costs: &HashTrieMap<ClassId, (Term, Cost)>,
        is_free: bool,
    ) -> (Term, Cost) {
        match &term {
            Term::Lit(_) => {
                // literals are always unique
                (term, NotNan::new(0.).unwrap())
            }
            Term::App(head, children) => {
                if is_type_operator(&head.to_string()) {
                    // types are not unioned, so they should be unique
                    return (term, NotNan::new(0.).unwrap());
                }

                let nodeid = &self.term_node(&term);
                let eclass = info.egraph.nid_to_cid(nodeid);
                if let Some((existing_term, _existing_cost)) = current_costs.get(eclass) {
                    (existing_term.clone(), NotNan::new(0.).unwrap())
                } else {
                    let unshared_cost = if is_free {
                        NotNan::new(0.).unwrap()
                    } else {
                        match other_costs.get(eclass) {
                            Some((_, cost)) => *cost,
                            // no cost stored, so it's free
                            None => NotNan::new(0.).unwrap(),
                        }
                    };

                    let mut cost = unshared_cost;

                    let new_term = {
                        let mut new_children = vec![];
                        for child in children {
                            let child = self.termdag.get(*child);
                            let (new_child, child_cost) = self.add_term_to_cost_set(
                                info,
                                current_costs,
                                child.clone(),
                                other_costs,
                                is_free,
                            );
                            new_children.push(new_child);
                            cost += child_cost;
                        }
                        self.termdag.app(*head, new_children)
                    };
                    self.add_correspondence(new_term.clone(), nodeid.clone());

                    if !is_free {
                        *current_costs =
                            current_costs.insert(eclass.clone(), (new_term.clone(), unshared_cost));
                    }

                    (new_term, cost)
                }
            }
            Term::Var(_) => {
                panic!("Found variable in term during extraction");
            }
        }
    }

    // Get the cost of a subregion
    // For DoWhile nodes, use special logic to calculate the cost based on iteration count
    fn subregions_cost(
        &self,
        info: &EgraphInfo,
        nodeid: NodeId,
        child_set: SmallVec<[&CostSet; 3]>,
    ) -> Cost {
        let node = info.egraph.nodes.get(&nodeid).unwrap();

        if node.op == "DoWhile" {
            assert!(child_set.len() == 1);
            let child_set = child_set[0];
            let inputs = info.egraph.nid_to_cid(&node.children[0]);
            let outputs = info.egraph.nid_to_cid(&node.children[1]);

            let loop_num_iters_guess = info
                .loop_iteration_estimates
                .get(&(inputs.clone(), outputs.clone()))
                .cloned()
                .unwrap_or(1000);

            child_set.total * NotNan::new(loop_num_iters_guess as f64).unwrap()
        } else if node.op == "If" {
            // Currently we don't do this for "Switch"
            // because the branches of Switch is hidden
            // behind an ListExpr
            assert!(child_set.len() == 2);
            let thn = child_set[0];
            let els = child_set[1];
            max(thn.total, els.total) + min(thn.total, els.total) * 0.3
        } else {
            child_set.iter().map(|cs| cs.total).sum()
        }
    }

    fn try_break_up_term(&self, term: &Term) -> Option<Vec<Term>> {
        match term {
            Term::App(head, children) => {
                if head.to_string() == "Concat" {
                    let child_terms = children.iter().map(|child| self.termdag.get(*child));
                    let mut child_broken_up = vec![];
                    for child_term in child_terms {
                        let broken_up = self.try_break_up_term(child_term)?;
                        child_broken_up.extend(broken_up);
                    }
                    Some(child_broken_up)
                } else if head.to_string() == "Empty" {
                    Some(vec![])
                } else if head.to_string() == "Single" {
                    Some(vec![self.termdag.get(children[0]).clone()])
                } else {
                    return None;
                }
            }
            Term::Lit(_) => None,
            Term::Var(_) => None,
        }
    }

    /// Replaces the leafs of the model_term with children
    /// Also adds to the `correspondence` map based on the model term.
    fn build_concat(&mut self, model_term: Term, children: &Vec<Term>) -> (Term, usize) {
        let existing_node = self
            .correspondence
            .get(&model_term)
            .unwrap_or_else(|| {
                panic!(
                    "Failed to find correspondence for term {:?} in build_concat",
                    model_term
                )
            })
            .clone();
        match model_term.clone() {
            Term::Lit(literal) => panic!("Unexpected literal in model term: {:?}", literal),
            Term::Var(op) => panic!("Unexpected variable in model term: {:?}", op),
            Term::App(op, vec) => match (op.as_str(), vec.as_slice()) {
                ("Concat", [left, right]) => {
                    let (left_term, left_size) =
                        self.build_concat(self.termdag.get(*left).clone(), children);
                    assert!(left_size < children.len());
                    let new_children = children.split_at(left_size).1.to_vec();
                    let (right_term, right_size) =
                        self.build_concat(self.termdag.get(*right).clone(), &new_children);
                    assert!(right_size <= new_children.len());
                    let new_term = self
                        .termdag
                        .app("Concat".into(), vec![left_term, right_term]);

                    self.correspondence
                        .insert(new_term.clone(), existing_node.clone());

                    (new_term, left_size + right_size)
                }
                ("Single", [_single]) => {
                    let new_term = self.termdag.app("Single".into(), vec![children[0].clone()]);
                    self.correspondence
                        .insert(new_term.clone(), existing_node.clone());
                    (new_term, 1)
                }
                ("Empty", [_arg_ty, _ctx]) => (model_term.clone(), 0),
                _ => panic!("Unexpected app in model term: {:?}", op),
            },
        }
    }

    /// Given a node and cost sets for children, calculate the cost set for the node.
    /// This function is cached so that we don't re-calculate cost sets.
    /// If a cycle is detected, we return None.
    fn calculate_cost_set(
        &mut self,
        nodeid: NodeId,
        child_cost_set_indicies: Vec<CostSetIndex>,
        info: &EgraphInfo,
    ) -> Option<CostSetIndex> {
        let original_term = self.get_term(
            info,
            nodeid.clone(),
            child_cost_set_indicies
                .iter()
                .map(|idx| self.costsets[*idx].term.clone())
                .collect(),
        );

        if let Some(&idx) = self.costsetmemo.get(&original_term) {
            return Some(idx);
        }
        let cid = info.egraph.nid_to_cid(&nodeid);
        let node = &info.egraph[&nodeid];

        let enode_children = enode_children(info.egraph, node);

        // we need to borrow cost sets, so swap them out
        // we mutate self for typechecking and termdag throughout this code
        let mut cost_sets_tmp = Default::default();
        std::mem::swap(&mut self.costsets, &mut cost_sets_tmp);

        let child_cost_sets = child_cost_set_indicies
            .iter()
            .map(|idx| &cost_sets_tmp[*idx])
            .zip(enode_children)
            .collect::<Vec<_>>();
        // cycle detection
        if child_cost_sets
            .iter()
            .any(|(cs, _)| cs.costs.contains_key(cid))
        {
            // remember to swap costsets back!
            std::mem::swap(&mut self.costsets, &mut cost_sets_tmp);
            return None;
        }

        let mut shared_total = NotNan::new(0.).unwrap();
        let mut unshared_total = info.cm.get_op_cost(&node.op);
        let mut args_used = HashSet::new();

        // special case: when the call is recursive, set super high cost
        if node.op == "Call" {
            let func_name = &node.children[0];
            let func_name_str = &info.egraph[func_name].op;
            if !func_name_str.starts_with('\"') && func_name_str.ends_with('\"') {
                panic!("Function name not a string: {:?}", func_name_str);
            }
            let func_name_str_without_quotes = &func_name_str[1..func_name_str.len() - 1];
            if func_name_str_without_quotes == info._func {
                unshared_total = NotNan::new(100000000000.0).unwrap();
            }
        }

        let mut costs: HashTrieMap<ClassId, (Term, Cost)> = Default::default();

        let mut children_terms = vec![];

        if !info.cm.ignore_children(&node.op) {
            for (child_set, enode_child) in child_cost_sets.iter() {
                let (mut new_child, should_add) = if enode_child.is_subregion {
                    (child_set.term.clone(), false)
                } else if enode_child.is_if_inputs {
                    // special case- try to only add cost for inputs that are used

                    // first, get all the indices of the children that are used
                    let mut used_children: HashSet<usize> = HashSet::new();
                    for (child_set, enode_child) in child_cost_sets.iter() {
                        if enode_child.is_subregion {
                            used_children.extend(child_set.args_used.iter());
                        }
                    }

                    // now that we have which children are used, try to break up the inputs
                    if let Some(broken_up_terms) = self.try_break_up_term(&child_set.term) {
                        let mut new_input_children = vec![];
                        for (idx, input_tuple_term) in broken_up_terms.iter().enumerate() {
                            let (child_term, net_cost) = self.add_term_to_cost_set(
                                info,
                                &mut costs,
                                input_tuple_term.clone(),
                                &child_set.costs,
                                !used_children.contains(&idx),
                            );
                            shared_total += net_cost;
                            new_input_children.push(child_term);
                        }
                        let (new_term, children_used) =
                            self.build_concat(child_set.term.clone(), &new_input_children);
                        assert_eq!(children_used, new_input_children.len());
                        (new_term, false)
                    } else {
                        (child_set.term.clone(), true)
                    }
                } else {
                    (child_set.term.clone(), true)
                };

                if should_add {
                    let (new_new_child_term, net_cost) = self.add_term_to_cost_set(
                        info,
                        &mut costs,
                        new_child.clone(),
                        &child_set.costs,
                        false,
                    );
                    shared_total += net_cost;
                    new_child = new_new_child_term;
                }
                children_terms.push(new_child);

                // if it's not a subregion, add to args_used
                if !enode_child.is_subregion {
                    args_used.extend(child_set.args_used.iter());
                }
            }

            // We separately compute the cost of all the subregions
            let css: SmallVec<[&CostSet; 3]> = child_cost_sets
                .iter()
                .filter(|(_, child)| child.is_subregion)
                .map(|(cs, _)| *cs)
                .collect();
            unshared_total += self.subregions_cost(info, nodeid.clone(), css);
        }

        let term = self.get_term(info, nodeid.clone(), children_terms);

        // no need to add something that costs 0 to the set
        if unshared_total > NotNan::new(0.).unwrap() {
            costs = costs.insert(cid.clone(), (term.clone(), unshared_total));
        }
        let total = unshared_total + shared_total;

        // for an argument, add all indicies
        if node.op == "Arg" {
            // first argument is type
            let ty = self.typecheck_term(&term);
            if let Type::TupleT(base_types) = ty {
                for i in 0..base_types.len() {
                    args_used.insert(i);
                }
            }
        }

        // for a get of an arg, clear args used except for the one used
        if node.op == "Get" {
            let arg_term = &child_cost_sets[0].0.term;
            match arg_term {
                Term::App(symbol, _items) => {
                    if symbol.to_string() == "Arg" {
                        let arg_index = child_cost_sets[1].0.term.clone();
                        match arg_index {
                            Term::Lit(Literal::Int(i)) => {
                                args_used.clear();
                                args_used.insert(i as usize);
                            }
                            _ => panic!("Unexpected term in Get: {:?}", arg_term),
                        }
                    }
                }
                _ => panic!("Unexpected term in Get: {:?}", arg_term),
            }
        }

        // swap borrowed costsets back!
        std::mem::swap(&mut self.costsets, &mut cost_sets_tmp);

        self.costsets.push(CostSet {
            total,
            costs,
            term,
            args_used,
        });
        let index = self.costsets.len() - 1;
        self.costsetmemo.insert(original_term, index);

        Some(index)
    }
}

/// This function handles finding children cost sets for a node in a particular region.
/// It then calculates the resulting cost set using `calculate_cost_set`.
/// Returns `None` when a cycle is found.
fn node_cost_in_region(
    rootid: ClassId,
    node_id: NodeId,
    extractor: &mut Extractor,
    info: &EgraphInfo,
) -> Option<CostSetIndex> {
    let node = &info.egraph[&node_id];

    // get the cost sets for the children
    let child_cost_sets = enode_children(info.egraph, node)
        .iter()
        .filter_map(
            |EnodeChild {
                 child,
                 is_subregion,
                 is_if_inputs: _is_inputs,
             }| {
                if *is_subregion {
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

#[allow(clippy::too_many_arguments)]
fn extract_fn(
    original_prog: &TreeProgram,
    func: &str,
    rootid: ClassId,
    egraph: egraph_serialize::EGraph,
    unextractables: IndexSet<String>,
    termdag: &mut TermDag,
    cost_model: &impl CostModel,
    should_maintain_linearity: bool,
) -> (CostSet, RcExpr) {
    // prune egraph
    let egraph = prune_egraph(&egraph, rootid.clone(), cost_model);

    log::info!("Building extraction info");
    let egraph_info = EgraphInfo::new(func, rootid.clone(), cost_model, &egraph, unextractables);
    let extractor_not_linear = &mut Extractor::new(original_prog, termdag);

    let (cost_res, res) = extract_with_paths(
        func,
        rootid.clone(),
        extractor_not_linear,
        &egraph_info,
        None,
    );

    if !should_maintain_linearity {
        (cost_res, res)
    } else {
        let effectful_nodes_along_path =
            extractor_not_linear.find_effectful_nodes_in_function(&res, &egraph_info);
        extractor_not_linear.costs.clear();
        let (cost_res, res) = extract_with_paths(
            func,
            rootid,
            extractor_not_linear,
            &egraph_info,
            Some(&effectful_nodes_along_path),
        );
        extractor_not_linear.check_function_is_linear(&res).unwrap();

        (cost_res, res)
    }
}

/// Returns the roots of DebugExpr relation and fresh names
/// for the extracted functions.
fn find_debug_roots(egraph: egraph_serialize::EGraph) -> Vec<(ClassId, String)> {
    let mut debug_roots = vec![];
    for (ith, (_nodeid, node)) in egraph.nodes.iter().enumerate() {
        if node.op == "DebugExpr" {
            let child_id = node.children[0].clone();
            let child_eclass = egraph.nid_to_cid(&child_id);
            let ith_name = format!("debug_{}", ith);
            debug_roots.push((child_eclass.clone(), ith_name));
        }
    }
    log::info!("Found {} debug roots", debug_roots.len());
    debug_roots
}

/// Inputs: a program, serialized egraph, and a set of functions to extract.
/// Also needs to know a set of unextractable functions and a cost model.
/// Produces a new program with the functions specified replaced by their extracted versions.
#[allow(clippy::too_many_arguments)]
pub fn extract(
    original_prog: &TreeProgram,
    fns: Vec<String>,
    egraph: egraph_serialize::EGraph,
    unextractables: IndexSet<String>,
    termdag: &mut TermDag,
    cost_model: impl CostModel,
    should_maintain_linearity: bool,
    extract_debug_exprs: bool,
) -> (Cost, TreeProgram) {
    let (cost, mut prog) = if extract_debug_exprs {
        log::info!("Extracting debug expressions.");
        let debug_roots = find_debug_roots(egraph.clone());
        let mut extracted_fns = vec![];
        let mut total_cost = NotNan::new(0.).unwrap();
        let mut typechecker = TypeChecker::new(original_prog, true);
        for (root, name) in debug_roots {
            let (cost, extracted) = extract_fn(
                original_prog,
                &name,
                root,
                egraph.clone(),
                unextractables.clone(),
                termdag,
                &cost_model,
                false,
            );
            total_cost += cost.total;
            let output_ty = typechecker
                .add_arg_types_to_expr(extracted.clone(), &None)
                .0;
            let input_ty = TypeChecker::get_arg_type(&extracted);
            // make a function out of the expr
            let func = Expr::Function(name.clone(), input_ty, output_ty, extracted);
            extracted_fns.push(Rc::new(func));
        }
        assert!(!extracted_fns.is_empty());
        let new_prog = TreeProgram {
            entry: extracted_fns[0].clone(),
            functions: extracted_fns[1..].to_vec(),
        };
        (total_cost, new_prog)
    } else {
        let mut new_prog = original_prog.clone();
        let mut cost = NotNan::new(0.).unwrap();
        for func in fns {
            let (fn_cost, extracted) = extract_fn(
                &new_prog,
                &func,
                egraph.nid_to_cid(&get_root(&egraph, &func)).clone(),
                egraph.clone(),
                unextractables.clone(),
                termdag,
                &cost_model,
                should_maintain_linearity,
            );
            new_prog.replace_fn(&func, extracted);
            cost += fn_cost.total;
        }
        (cost, new_prog)
    };

    prog.remove_dead_code_nodes();
    (cost, prog)
}

// Returns a duration or None if any extractions timed out
pub fn extract_ilp(
    original_prog: &TreeProgram,
    fns: Vec<String>,
    egraph: egraph_serialize::EGraph,
    unextractables: IndexSet<String>,
    cost_model: impl CostModel,
    timeout: Duration,
) -> Vec<ExtractionTimeSample> {
    let mut termdag = egglog::TermDag::default();
    log::info!("Extracting functions with ILP {:?}", fns);

    let mut samples = vec![];
    for name in fns {
        log::info!("Extracting function {} with ILP", name);
        let root = egraph.nid_to_cid(&get_root(&egraph, &name)).clone();

        let egraph = prune_egraph(&egraph, root.clone(), &cost_model);
        let egraph_size = egraph.nodes.len();

        let ilp_time = extract_fn_ilp(&name, root.clone(), egraph.clone(), &cost_model, timeout);

        let time_before = Instant::now();
        let _res2 = extract_fn(
            original_prog,
            &name,
            root,
            egraph.clone(),
            unextractables.clone(),
            &mut termdag,
            &cost_model,
            false,
        );
        let eggcc_time = time_before.elapsed();

        samples.push(ExtractionTimeSample {
            egraph_size,
            ilp_time,
            eggcc_time,
        });
    }
    samples
}

// returns how long impl extraction took
pub fn extract_fn_ilp(
    func: &str,
    rootid: ClassId,
    egraph: egraph_serialize::EGraph,
    cost_model: &impl CostModel,
    timeout: Duration,
) -> Option<Duration> {
    // prune egraph
    let egraph = prune_egraph(&egraph, rootid.clone(), cost_model);

    // run ILP extraction, timing it
    let ilp_extractor = FasterCbcExtractorWithTimeout::new(timeout.as_secs() as u32);

    let before = Instant::now();

    let res = ilp_extractor.extract(&egraph, &[rootid.clone()]);

    let elapsed = before.elapsed();

    log::info!(
        "ILP extraction for {} took {} seconds",
        func,
        elapsed.as_secs_f64()
    );
    eprintln!("elapsed: {:?}", elapsed);
    if res.is_none() {
        None
    } else {
        eprintln!("time taken: {:?}", elapsed);
        Some(min(elapsed, timeout))
    }
}

/// Extract the function specified by `func` from the egraph.
pub fn extract_with_paths(
    func: &str,
    func_root: ClassId,
    extractor: &mut Extractor,
    info: &EgraphInfo,
    // If effectful paths are present,
    // for each region we will only consider
    // effectful nodes that are in effectful_path[rootid]
    effectful_paths: Option<&IndexMap<ClassId, IndexSet<NodeId>>>,
) -> (CostSet, RcExpr) {
    if effectful_paths.is_some() {
        log::info!("Re-extracting program after linear path is found.");
    } else {
        log::info!("Extracting program for the first time.");
    }
    let mut worklist: PriorityQueue<(ClassId, ClassId)> = PriorityQueue::default();

    let mut parents = info.parents.clone();
    let root_node = (ClassId::from("ROOT_NODE".to_string()), ClassId::from("ROOT_NODE".to_string()));
    let _ = parents.insert(root_node.clone(), info.roots.clone());
    worklist.insert(root_node, NotNan::new(0.0).unwrap());

    while let Some((rootid, classid)) = worklist.pop() {
        dbg!(&rootid);
        if let Some(parents) = parents.get(&(rootid.clone(), classid.clone())) {
            dbg!(parents);
            for parent in parents {
                let (rootid, nodeid) = parent.clone();

                let classid = info.n2c(&nodeid);
                let node = info.egraph.nodes.get(&nodeid).unwrap();

                // skip nodes with infinite cost
                if info.cm.get_op_cost(&node.op).is_infinite() {
                    continue;
                }

                if info.unextractables.contains(&node.op) {
                    continue;
                }

                // Skip inlined calls
                if node.op == "Call"
                    && info.inlined_calls.contains(&(
                        info.n2c(&node.children[0]).clone(),
                        info.n2c(&node.children[1]).clone(),
                    ))
                {
                    continue;
                }

                let sort_of_node = info.get_sort_of_eclass(&classid);
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

                let region_costs = extractor.costs.entry(rootid.clone()).or_default();
                let lookup = region_costs.get(&classid);
                let mut prev_cost: Cost = std::f64::INFINITY.try_into().unwrap();
                if let Some(lookup) = lookup {
                    let costset = extractor.costsets.get(*lookup).unwrap();
                    prev_cost = costset.total;
                }

                if let Some(cost_set_index) =
                    node_cost_in_region(rootid.clone(), nodeid.clone(), extractor, info)
                {
                    dbg!(node);
                    let cost_set = &extractor.costsets[cost_set_index];
                    let region_costs = extractor.costs.get_mut(&rootid).unwrap();
                    if cost_set.total < prev_cost {
                        region_costs.insert(classid.clone(), cost_set_index);
                        worklist.insert((rootid, classid), cost_set.total);
                    }
                }
            }
        }
    }

    dbg!(&func_root);
    dbg!(extractor
        .costs
        .get(&func_root));
    dbg!(&extractor.costs);
    let root_costset_index = *extractor
        .costs
        .get(&func_root)
        .unwrap_or_else(|| panic!("Failed to extract function {}!", func))
        .get(&func_root)
        .unwrap_or_else(|| {
            if effectful_paths.is_some() {
                panic!(
                    "Failed to extract function {} after linear path is found!",
                    func
                );
            } else {
                panic!(
                    "Failed to extract function {} during initial extraction!",
                    func
                );
            }
        });
    let root_costset = extractor.costsets[root_costset_index].clone();

    // now run translation to expressions and compute eclass types
    let resulting_prog = extractor.compute_eclass_types(info, root_costset.term.clone());

    let root_cost = root_costset.total;
    if root_cost.is_infinite() {
        panic!("Failed to extract program! Found infinite cost on result node.");
    }
    if root_cost.is_sign_negative() {
        panic!("Failed to extract program! Found negative cost on result node.");
    }

    log::info!("extracted with cost {}", root_cost);

    (root_costset, resulting_prog)
}

pub trait CostModel {
    /// TODO: we could do better with type info
    fn get_op_cost(&self, op: &str) -> Cost;

    /// if true, the op's children are ignored in calculating the cost
    fn ignore_children(&self, op: &str) -> bool;
}

pub struct DefaultCostModel;
pub struct TestCostModel;

impl CostModel for TestCostModel {
    fn get_op_cost(&self, op: &str) -> Cost {
        match op {
            "Get" => (0.).try_into().unwrap(),
            _ => DefaultCostModel.get_op_cost(op),
        }
    }

    fn ignore_children(&self, op: &str) -> bool {
        DefaultCostModel.ignore_children(op)
    }
}

impl CostModel for DefaultCostModel {
    fn get_op_cost(&self, op: &str) -> Cost {
        match op {
            // Tmp contexts
            x if is_dummy_ctx(x) => 0.,
            x if is_type_operator(x) => 0.,
            // constant types
            "Int" | "Bool" | "Float" => 0.,
            // Leaves
            "Const" => 1.,
            "Arg" => 0.,
            _ if op.parse::<i64>().is_ok() || op.parse::<f64>().is_ok() || op.starts_with('"') => {
                0.
            }
            "true" | "false" | "()" => 0.,
            // Lists
            "Empty" | "Single" | "Concat" | "Nil" | "Cons" => 0.,
            // small cost for get to encourage canonicalization
            // enables state edge passthrough to work as a pass
            "Get" => 0.01,
            // Algebra
            "Abs" | "Bitand" | "Neg" | "Add" | "PtrAdd" | "Sub" | "And" | "Or" | "Not" | "Shl"
            | "Shr" => 10.,
            "FAdd" | "FSub" | "Fmax" | "Fmin" => 50.,
            "Mul" => 30.,
            "FMul" => 150.,
            "Div" => 50.,
            "FDiv" => 250.,
            // Comparisons
            "Eq" | "LessThan" | "GreaterThan" | "LessEq" | "GreaterEq" => 10.,
            "Select" | "Smax" | "Smin" => 10.,
            "FEq" => 10.,
            "FLessThan" | "FGreaterThan" | "FLessEq" | "FGreaterEq" => 100.,
            // Effects
            "Print" | "Write" | "Load" => 50.,
            "Alloc" | "Free" => 100.,
            "Call" => 1000000., // This (very roughly) bounds the size of an expression we inline
            // Control
            "Program" | "Function" => 0.,
            // custom logic for DoWhile will multiply the body by the LoopNumItersGuess
            "DoWhile" => 1.,
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

#[derive(Clone)]
struct PriorityQueuePayload<D, C> {
    data: D,
    cost: C,
}

impl<D, C> PartialEq for PriorityQueuePayload<D, C>
where
    D: Eq,
    C: PartialOrd,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl<D, C> Eq for PriorityQueuePayload<D, C>
where
    D: Eq,
    C: PartialOrd,
{
}
impl<D, C> PartialOrd for PriorityQueuePayload<D, C>
where
    D: Eq,
    C: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}
impl<D, C> Ord for PriorityQueuePayload<D, C>
where
    D: Eq,
    C: PartialOrd,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone)]
pub(crate) struct PriorityQueue<T>
where
    T: Eq + std::hash::Hash + Ord + Clone,
{
    set: HashMap<T, Cost>,
    queue: BTreeSet<PriorityQueuePayload<T, Cost>>,
}

impl<T> Default for PriorityQueue<T>
where
    T: Eq + std::hash::Hash + Ord + Clone,
{
    fn default() -> Self {
        PriorityQueue {
            set: Default::default(),
            queue: Default::default(),
        }
    }
}

// This implementation is Dijkstra-specific
impl<T> PriorityQueue<T>
where
    T: Eq + std::hash::Hash + Ord + Clone,
{
    pub fn insert(&mut self, t: T, c: Cost) {
        if let Some(old_cost) = self.set.get(&t) {
            let old_cost = *old_cost;
            if old_cost <= c {
                return;
            }
            let _ = self.set.insert(t.clone(), c);
            // if the cost is lower, we need to remove the old one
            self.queue.remove(&PriorityQueuePayload {
                data: t.clone(),
                cost: old_cost,
            });
            // and add the new one
            self.queue.insert(PriorityQueuePayload {
                data: t.clone(),
                cost: c,
            });
        } else {
            let _ = self.set.insert(t.clone(), c);
            // if the cost is new, we need to add it
            self.queue.insert(PriorityQueuePayload {
                data: t.clone(),
                cost: c,
            });
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let res = self.queue.pop_first();
        // In the UniqueQueue, we remove the element from the set,
        // but here we don't because we want to guard against the same data
        // being added in the future (property of Dijkstra)
        res.as_ref().map(|t| {
            // self.set.remove(&t.data);
            t.data.clone()
        })
    }
}

struct EnodeChild {
    child: ClassId,
    is_subregion: bool,
    // cost of inputs is calculated specially- dead code is not included in cost
    is_if_inputs: bool,
}

impl EnodeChild {
    fn new(child: ClassId, is_subregion: bool, _is_assumption: bool, if_if_inputs: bool) -> Self {
        EnodeChild {
            child,
            is_subregion,
            is_if_inputs: if_if_inputs,
        }
    }
}

pub(crate) fn is_dummy_ctx(op: &str) -> bool {
    op.starts_with("DumC")
}

/// For a given enode, returns a vector of children eclasses.
/// Also, for each child returns if the child is a region root.
fn enode_children(
    egraph: &egraph_serialize::EGraph,
    enode: &egraph_serialize::Node,
) -> Vec<EnodeChild> {
    match (enode.op.as_str(), enode.children.as_slice()) {
        ("DoWhile", [input, body]) => vec![
            EnodeChild::new(egraph.nid_to_cid(input).clone(), false, false, false),
            EnodeChild::new(egraph.nid_to_cid(body).clone(), true, false, false),
        ],
        ("If", [pred, input, then_branch, else_branch]) => vec![
            EnodeChild::new(egraph.nid_to_cid(pred).clone(), false, false, false),
            EnodeChild::new(egraph.nid_to_cid(input).clone(), false, false, true),
            EnodeChild::new(egraph.nid_to_cid(then_branch).clone(), true, false, false),
            EnodeChild::new(egraph.nid_to_cid(else_branch).clone(), true, false, false),
        ],
        ("Switch", [pred, input, branchlist]) => {
            let mut res = vec![
                EnodeChild::new(egraph.nid_to_cid(pred).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(input).clone(), false, false, true),
            ];
            res.extend(
                get_conslist_children(egraph, egraph.nid_to_cid(branchlist).clone())
                    .into_iter()
                    .map(|cid| EnodeChild::new(cid, true, false, false)),
            );
            res
        }
        ("Function", [name, args, ret, body]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(name).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(args).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(ret).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(body).clone(), true, false, false),
            ]
        }
        ("Arg", [ty, ctx]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(ty).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(ctx).clone(), false, true, false),
            ]
        }
        ("Const", [c, ty, ctx]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(c).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(ty).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(ctx).clone(), false, true, false),
            ]
        }
        ("Empty", [ty, ctx]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(ty).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(ctx).clone(), false, true, false),
            ]
        }
        // We mark operators like (Add) and (Mul) as region roots
        // because we want their cost to be counted every time they
        // are referenced at a different place, just like a region.
        ("Uop", [op, a]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(op).clone(), true, false, false),
                EnodeChild::new(egraph.nid_to_cid(a).clone(), false, false, false),
            ]
        }
        ("Bop", [op, a, b]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(op).clone(), true, false, false),
                EnodeChild::new(egraph.nid_to_cid(a).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(b).clone(), false, false, false),
            ]
        }
        ("Top", [op, a, b, c]) => {
            vec![
                EnodeChild::new(egraph.nid_to_cid(op).clone(), true, false, false),
                EnodeChild::new(egraph.nid_to_cid(a).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(b).clone(), false, false, false),
                EnodeChild::new(egraph.nid_to_cid(c).clone(), false, false, false),
            ]
        }
        _ => {
            let mut children = vec![];
            for child in &enode.children {
                children.push(EnodeChild::new(
                    egraph.nid_to_cid(child).clone(),
                    false,
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
fn find_reachable(
    egraph: &egraph_serialize::EGraph,
    root: ClassId,
    cm: &dyn CostModel,
    include_non_roots: bool,
    recursive: bool,
) -> IndexSet<ClassId> {
    let mut visited = IndexSet::new();
    let mut result = IndexSet::new();
    let mut queue = UniqueQueue::default();
    queue.insert(root);

    while let Some(eclass) = queue.pop() {
        let eclass_type = egraph.class_data[&eclass].typ.as_ref().unwrap();

        if !type_is_part_of_ast(eclass_type) {
            continue;
        }
        if visited.insert(eclass.clone()) {
            if include_non_roots {
                result.insert(eclass.clone());
            }
            for node in &egraph.classes()[&eclass].nodes {
                // skip nodes with infinite cost
                if cm.get_op_cost(&egraph[node].op).is_infinite() {
                    continue;
                }

                for EnodeChild {
                    child,
                    is_subregion,
                    is_if_inputs: _is_inputs,
                } in enode_children(egraph, &egraph[node])
                {
                    if is_subregion {
                        if recursive {
                            queue.insert(child.clone());
                            result.insert(child);
                        }
                    } else {
                        queue.insert(child);
                    }
                }
            }
        }
    }

    result
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
    egraph.parse_and_run_program(None, &string_prog).unwrap();
    let (serialized_egraph, unextractables) = serialized_egraph(egraph);
    let mut termdag = TermDag::default();

    let cost_set = extract(
        prog,
        prog.fns(),
        serialized_egraph,
        unextractables,
        &mut termdag,
        TestCostModel,
        true,
        false,
    );

    assert_eq!(
        cost_set.0, expected_cost,
        "Expected cost to be {}",
        expected_cost
    );
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
    egraph.parse_and_run_program(None, &string_prog).unwrap();
    let (serialized_egraph, unextractables) = serialized_egraph(egraph);
    let mut termdag = TermDag::default();

    let mut err = Ok(());
    for func in prog.fns() {
        let root = serialized_egraph.nid_to_cid(&get_root(&serialized_egraph, &func));
        let pruned = prune_egraph(&serialized_egraph, root.clone(), &DefaultCostModel);

        let egraph_info = EgraphInfo::new(
            &func,
            root.clone(),
            &DefaultCostModel,
            &pruned,
            unextractables.clone(),
        );
        let extractor_not_linear = &mut Extractor::new(prog, &mut termdag);

        let (_cost_res, prog) = extract_with_paths(
            &func,
            root.clone(),
            extractor_not_linear,
            &egraph_info,
            None,
        );
        let res = extractor_not_linear.check_function_is_linear(&prog);
        if let Err(e) = res {
            err = Err(e);
            break;
        }
    }
    match err {
        Ok(_) => panic!("Expected program to be non-linear!"),
        Err(e) => {
            if !e.starts_with(error_message) {
                panic!(
                    "Expected error message to start with '{}', got '{}'",
                    error_message, e
                );
            }
        }
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
    let cost_model = TestCostModel;
    let cost_inside_loop = cost_model.get_op_cost("LessThan")
    // while the same const is used several times, it is only counted twice
    + cost_model.get_op_cost("Const")
    + cost_model.get_op_cost("Add");

    let cost_of_one_func = cost_model.get_op_cost("DoWhile")
        + NotNan::new(1000.).unwrap() * cost_inside_loop
        + cost_model.get_op_cost("Const")
        + cost_model.get_op_cost("Add");
    // two of the same function
    let expected_cost = cost_of_one_func * 2.;
    dag_extraction_test(&prog, expected_cost);
}

#[test]
fn test_dag_extract_if() {
    use crate::ast::*;
    let prog = program!(function(
        "func_if",
        tuplet!(intt(), statet()),
        tuplet!(intt(), statet()),
        parallel!(
            get(
                tif(
                    less_than(int(10), int(10)),
                    parallel!(getat(0)),
                    parallel!(mul(int(0), int(0))),
                    parallel!(int(1))
                ),
                0
            ),
            getat(1)
        )
    ),);
    let cost_model = TestCostModel;
    let cost_then = cost_model.get_op_cost("Mul") + cost_model.get_op_cost("Const");
    let cost_else = cost_model.get_op_cost("Const");
    let cost_if = cost_then
        + cost_else * 0.3
        + cost_model.get_op_cost("LessThan")
        + cost_model.get_op_cost("Const")
        + cost_model.get_op_cost("Get")
        + cost_model.get_op_cost("If");
    let cost_total = cost_if + cost_model.get_op_cost("Get");
    dag_extraction_test(&prog, cost_total);
}

#[test]
fn test_cost_dead_code_to_if() {
    use crate::ast::*;

    let prog = program!(function(
        "main",
        tuplet!(intt(), statet()),
        tuplet!(intt(), statet()),
        tif(
            ttrue(),
            parallel!(int(10), int(20), getat(1)),
            parallel!(add(getat(0), getat(0)), getat(2)),
            parallel!(add(getat(0), getat(0)), getat(2))
        ),
    ),);
    let cost_model = TestCostModel;
    // count the constant 10 and the constant true
    // don't count the constant 20
    let expected_cost = cost_model.get_op_cost("Const") * 2.
        + cost_model.get_op_cost("Add") * 1.3 // if cost model
        + cost_model.get_op_cost("If");

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
    let cost_model = TestCostModel;

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
        "Resulting program violated linearity! There are unconsumed effectful operators",
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
    egraph.parse_and_run_program(None, &string_prog).unwrap();
    let (serialized_egraph, unextractables) = serialized_egraph(egraph);
    let mut termdag = TermDag::default();

    let root = serialized_egraph.nid_to_cid(&get_root(&serialized_egraph, "main"));

    let pruned = prune_egraph(&serialized_egraph, root.clone(), &DefaultCostModel);
    let egraph_info = EgraphInfo::new(
        "main",
        root.clone(),
        &TestCostModel,
        &pruned,
        unextractables.clone(),
    );
    let extractor_not_linear = &mut Extractor::new(&prog, &mut termdag);

    let (_cost_res, res) = extract_with_paths(
        "main",
        root.clone(),
        extractor_not_linear,
        &egraph_info,
        None,
    );
    // first extraction should fail linearity check
    assert!(extractor_not_linear.check_function_is_linear(&res).is_err());

    // second extraction should succeed
    extract(
        &prog,
        vec!["main".to_string()],
        serialized_egraph,
        unextractables,
        &mut termdag,
        TestCostModel,
        true,
        false,
    );
}

pub(crate) fn has_debug_exprs(serialized_egraph: &egraph_serialize::EGraph) -> bool {
    for (_, node) in &serialized_egraph.nodes {
        if node.op == "DebugExpr" {
            return true;
        }
    }
    false
}

/// Given an egraph where some nodes are dropped,
/// fix up the node ids to be valid.
fn rename_to_valid_children(
    original_egraph: &egraph_serialize::EGraph,
    egraph: &egraph_serialize::EGraph,
) -> egraph_serialize::EGraph {
    let mut new_egraph = egraph_serialize::EGraph::default();

    let mut new_representative = HashMap::new();

    for node in egraph.nodes.iter() {
        new_representative.insert(egraph.nid_to_cid(node.0), node.0);
    }

    for (nodeid, node) in egraph.nodes.iter() {
        let new_children = node.children.iter().map(|child| {
            (*new_representative
                .get(&original_egraph.nid_to_cid(child))
                .unwrap_or_else(|| panic!("Failed to find child {:?} in new egraph", child)))
            .clone()
        });

        let new_node = egraph_serialize::Node {
            children: new_children.collect(),
            ..node.clone()
        };

        new_egraph.add_node(nodeid.clone(), new_node);
    }

    // copy over class data for each class
    for (class, data) in &egraph.class_data {
        new_egraph.class_data.insert(class.clone(), data.clone());
    }

    new_egraph.root_eclasses.clone_from(&egraph.root_eclasses);

    new_egraph
}

// Prunes an egraph to only reachable nodes, removing context and types
// replaces types with "UnknownT" and replaces context with unique identifiers
fn prune_egraph(
    egraph: &egraph_serialize::EGraph,
    root: ClassId,
    cost_model: &impl CostModel,
) -> egraph_serialize::EGraph {
    let mut new_egraph = egraph_serialize::EGraph::default();
    let mut visited = HashSet::new();

    let mut todo = vec![root.clone()];

    while let Some(class) = todo.pop() {
        if !visited.insert(class.clone()) {
            continue;
        }

        let mut has_non_inf_cost = false;
        for nodeid in &egraph.classes()[&class].nodes {
            let node = &egraph[nodeid];
            // if the op is a ctx, replace it with a fresh DumC
            if is_ctx_operator(&node.op) {
                let new_node = Node {
                    op: format!("DumC{}", nodeid),
                    children: vec![],
                    eclass: node.eclass.clone(),
                    cost: NotNan::new(0.).unwrap(),
                    subsumed: false,
                };

                new_egraph.add_node(nodeid.clone(), new_node);
                has_non_inf_cost = true;
            // if it has infinite cost, skip it
            } else if cost_model.get_op_cost(&node.op).is_infinite() {
            }
            // if the node is an integer, we copy it over later
            else if let Ok(_parse) = node.op.parse::<i64>() {
                has_non_inf_cost = true;
            }
            // if the node stores a string, we copy it over later
            else if node.op.starts_with('"') {
                has_non_inf_cost = true;
            } else {
                // copy node to new_egraph
                new_egraph.add_node(nodeid.clone(), node.clone());

                // add children to queue
                for child in &node.children {
                    todo.push(egraph.nid_to_cid(child).clone());
                }
                has_non_inf_cost = true;
            }
        }

        // It's possible that no nodes in the eclass had finite cost
        // because DropAt can fail to finish for some eclasses.
        // This is expected, so add any of the nodes in the eclass to the new egraph.
        if !has_non_inf_cost {
            let nodeid = &egraph.classes()[&class].nodes[0];
            let node = &egraph[nodeid];
            new_egraph.add_node(nodeid.clone(), node.clone());
        }
    }

    // copy over "InlinedCall" and "LoopNumItersGuess" nodes, which depend on integers and strings
    for (nodeid, node) in &egraph.nodes {
        if node.op == "InlinedCall" && visited.contains(egraph.nid_to_cid(&node.children[1])) {
            new_egraph.add_node(nodeid.clone(), node.clone());
        }

        if node.op == "LoopNumItersGuess"
            && visited.contains(egraph.nid_to_cid(&node.children[0]))
            && visited.contains(egraph.nid_to_cid(&node.children[1]))
        {
            // copy over the node
            new_egraph.add_node(nodeid.clone(), node.clone());
        }

        // if the node is an integer, copy it over
        if node.op.parse::<i64>().is_ok() {
            new_egraph.add_node(nodeid.clone(), node.clone());
        }

        // if the node stores a string, copy it over
        if node.op.starts_with('"') {
            new_egraph.add_node(nodeid.clone(), node.clone());
        }
    }

    // copy over class data for each class
    for (class, data) in &egraph.class_data {
        if visited.contains(class) {
            new_egraph.class_data.insert(class.clone(), data.clone());
        }
    }

    new_egraph.root_eclasses = vec![root];

    rename_to_valid_children(egraph, &new_egraph)
}

fn is_type_operator(op: &str) -> bool {
    op == "TupleT"
        || op == "Base"
        || op == "IntT"
        || op == "BoolT"
        || op == "FloatT"
        || op == "PointerT"
        || op == "StateT"
        || op == "TNil"
        || op == "TCons"
}

fn is_ctx_operator(op: &str) -> bool {
    op == "InFunc" || op == "InLoop" || op == "InSwitch" || op == "InIf" || is_dummy_ctx(op)
}
