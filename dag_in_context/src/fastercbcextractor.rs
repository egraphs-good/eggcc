/*

Taken from https://github.com/egraphs-good/extraction-gym/blob/main/src/extract/faster_ilp_cbc.rs
With commit 8de5131

Produces a dag-cost optimal extraction of an Egraph.

This can take >10 hours to run on some egraphs, so there's the option to provide a timeout.

To operate:
1) It simplifies the egraph by removing nodes that can't be selected in the optimal
solution, as well as collapsing other classes down.
2) It then sends the problem to the COIN-OR CBC solver to find an extraction (or timeout).
It allows the solver to generate solutions that contain cycles
3) The solution from the solver is checked, and if the extraction contains a cycle, extra
constraints are added to block the cycle and the solver is called again.

In SAT solving, it's common to call a solver incrementally. Each time you call the SAT
solver with more clauses to the SAT solver (constraining the solution further), and
allowing the SAT solver to reuse its previous work.

So there are two uses of "incremental", one is gradually sending more of the problem to the solver,
and the other is the solver being able to re-use the previous work when it receives additional parts
of the problem. In the case here, we're just referring to sending extra pieces of the problem to
the solver. COIN-OR CBC doesn't provide an interface that allows us to call it and reuse what it
has discovered previously.

In the case of COIN-OR CBC, we're sending extra constraints each time we're solving, these
extra constraints are prohibiting cycles that were found in the solutions that COIN-OR CBC
previously produced.

Obviously, we could add constraints to block all the cycles the first time we call COIN-OR CBC,
so we'd only need to call the solver once. However, for the problems in our test-set, lots of these
constraints don't change the answer, they're removing cycles from high-cost extractions.  These
extra constraints do slow down solving though - and for our test-set it gives a faster runtime when
we incrementally add constraints that break cycles when they occur in the lowest cost extraction.

We've experimented with two ways to break cycles.

One approach is by enforcing a topological sort on nodes. Each node has a level, and each edge
can only connect from a lower level to a higher level node.

Another approach, is by explicity banning cycles. Say in an extraction that the solver generates
we find a cycle A->B->A. Say there are two edges, edgeAB, and edgeBA, which connect A->B, then B->A.
Then any solution that contains both edgeAB, and edgeBA will contain a cycle.  So we add a constraint
that at most one of these two edges can be active. If we check through the whole extraction for cycles,
and ban each cycle that we find, then try solving again, we'll get a new solution which, if it contains
cycles, will not contain any of the cycles we've previously seen. We repeat this until timeout, or until
we get an optimal solution without cycles.


*/

use crate::extractiongymfastergreedydag::FasterGreedyDagExtractor;

use super::*;
use coin_cbc::{Col, Model};
use egraph_serialize::*;
use indexmap::IndexSet;
use ordered_float::NotNan;
use rustc_hash::FxHashSet;
use std::fmt;

#[derive(Debug)]
pub struct Config {
    pub pull_up_costs: bool,
    pub remove_self_loops: bool,
    pub remove_high_cost_nodes: bool,
    pub remove_more_expensive_subsumed_nodes: bool,
    pub remove_unreachable_classes: bool,
    pub pull_up_single_parent: bool,
    pub take_intersection_of_children_in_class: bool,
    pub move_min_cost_of_members_to_class: bool,
    pub find_extra_roots: bool,
    pub remove_empty_classes: bool,
    pub return_improved_on_timeout: bool,
    pub remove_single_zero_cost: bool,
}

impl Config {
    pub const fn default() -> Self {
        Self {
            pull_up_costs: true,
            remove_self_loops: true,
            remove_high_cost_nodes: true,
            remove_more_expensive_subsumed_nodes: true,
            remove_unreachable_classes: true,
            pull_up_single_parent: true,
            take_intersection_of_children_in_class: true,
            move_min_cost_of_members_to_class: false,
            find_extra_roots: true,
            remove_empty_classes: true,
            return_improved_on_timeout: true,
            remove_single_zero_cost: true,
        }
    }
}

struct NodeILP {
    variable: Col,
    cost: Cost,
    member: NodeId,
    children_classes: IndexSet<ClassId>,
}

struct ClassILP {
    active: Col,
    members: Vec<NodeId>,
    variables: Vec<Col>,
    costs: Vec<Cost>,
    // Initially this contains the children of each member (respectively), but
    // gets edited during the run, so mightn't match later on.
    childrens_classes: Vec<IndexSet<ClassId>>,
}

impl fmt::Debug for ClassILP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "classILP[{}] {{ node: {:?}, children: {:?},  cost: {:?} }}",
            self.members(),
            self.members,
            self.childrens_classes,
            self.costs
        )
    }
}

impl ClassILP {
    fn remove(&mut self, idx: usize) {
        self.variables.remove(idx);
        self.costs.remove(idx);
        self.members.remove(idx);
        self.childrens_classes.remove(idx);
    }

    fn remove_node(&mut self, node_id: &NodeId) {
        if let Some(idx) = self.members.iter().position(|n| n == node_id) {
            self.remove(idx);
        }
    }

    fn members(&self) -> usize {
        self.variables.len()
    }

    fn as_nodes(&self) -> Vec<NodeILP> {
        self.variables
            .iter()
            .zip(&self.costs)
            .zip(&self.members)
            .zip(&self.childrens_classes)
            .map(|(((variable, &cost_), member), children_classes)| NodeILP {
                variable: *variable,
                cost: cost_,
                member: member.clone(),
                children_classes: children_classes.clone(),
            })
            .collect()
    }

    fn get_children_of_node(&self, node_id: &NodeId) -> &IndexSet<ClassId> {
        let idx = self.members.iter().position(|n| n == node_id).unwrap();
        &self.childrens_classes[idx]
    }

    fn get_variable_for_node(&self, node_id: &NodeId) -> Option<Col> {
        if let Some(idx) = self.members.iter().position(|n| n == node_id) {
            return Some(self.variables[idx]);
        }
        None
    }
}

pub struct FasterCbcExtractorWithTimeout {
    timeout_in_seconds: u32,
}

// Some problems take >36,000 seconds to optimise.
impl FasterCbcExtractorWithTimeout {
    pub fn new(timeout_in_seconds: u32) -> Self {
        Self { timeout_in_seconds }
    }

    pub fn extract(&self, egraph: &EGraph, roots: &[ClassId]) -> Option<ExtractionResult> {
        extract(egraph, roots, &Config::default(), self.timeout_in_seconds)
    }
}

// Modified from extraction gym to return none on timeout
fn extract(
    egraph: &EGraph,
    roots_slice: &[ClassId],
    config: &Config,
    timeout: u32,
) -> Option<ExtractionResult> {
    let start_time = Instant::now();

    // todo from now on we don't use roots_slice - be good to prevent using it any more.
    let mut roots = roots_slice.to_vec();
    roots.sort();
    roots.dedup();

    let simp_start_time = std::time::Instant::now();

    let mut model = Model::default();
    //silence verbose stdout output
    model.set_parameter("loglevel", "0");

    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);

    let mut vars: IndexMap<ClassId, ClassILP> = egraph
        .classes()
        .values()
        .map(|class| {
            let cvars = ClassILP {
                active: model.add_binary(),
                variables: class.nodes.iter().map(|_| model.add_binary()).collect(),
                costs: class.nodes.iter().map(|n| egraph[n].cost).collect(),
                members: class.nodes.clone(),
                childrens_classes: class
                    .nodes
                    .iter()
                    .map(|n| {
                        egraph[n]
                            .children
                            .iter()
                            .map(|c| n2c(c).clone())
                            .collect::<IndexSet<ClassId>>()
                    })
                    .collect(),
            };
            (class.id.clone(), cvars)
        })
        .collect();

    let initial_result = FasterGreedyDagExtractor.extract(egraph, &roots);
    let initial_result_cost = initial_result.dag_cost(egraph, &roots);

    // For classes where we know the choice already, we set the nodes early.
    let mut result = ExtractionResult::default();

    //This could be much more efficient, but it only takes less than 5 seconds for all our benchmarks.
    //The ILP solver takes the time.
    for _i in 1..3 {
        remove_with_loops(&mut vars, &roots, config);
        remove_high_cost(&mut vars, initial_result_cost, &roots, config);
        remove_more_expensive_subsumed_nodes(&mut vars, config);
        remove_unreachable_classes(&mut vars, &roots, config);
        pull_up_with_single_parent(&mut vars, &roots, config);
        pull_up_costs(&mut vars, &roots, config);
        remove_single_zero_cost(&mut vars, &mut result, &roots, config);
        find_extra_roots(&mut vars, &mut roots, config);
        remove_empty_classes(&mut vars, config);
    }

    for (classid, class) in &vars {
        if class.members() == 0 {
            if roots.contains(classid) {
                log::info!("Infeasible, root has no possible children, returning empty solution");
                return Some(ExtractionResult::default());
            }

            model.set_col_upper(class.active, 0.0);
            continue;
        }

        if class.members() == 1 && class.childrens_classes[0].is_empty() && class.costs[0] == 0.0 {
            continue;
        }

        // class active == some node active
        // sum(for node_active in class) == class_active

        let row = model.add_row();
        model.set_row_equal(row, 0.0);
        model.set_weight(row, class.active, -1.0);
        for &node_active in &class.variables.iter().collect::<IndexSet<_>>() {
            model.set_weight(row, *node_active, 1.0);
        }

        let childrens_classes_var =
            |cc: &IndexSet<ClassId>| cc.iter().map(|n| vars[n].active).collect::<IndexSet<_>>();

        let mut intersection: IndexSet<Col> = Default::default();

        if config.take_intersection_of_children_in_class {
            // otherwise the intersection is empty (i.e. disabled.)
            intersection = childrens_classes_var(&class.childrens_classes[0].clone());
        }

        for childrens_classes in &class.childrens_classes[1..] {
            intersection = intersection
                .intersection(&childrens_classes_var(childrens_classes))
                .cloned()
                .collect();
        }

        // A class being active implies that all in the intersection
        // of it's children are too.
        for c in &intersection {
            let row = model.add_row();
            model.set_row_upper(row, 0.0);
            model.set_weight(row, class.active, 1.0);
            model.set_weight(row, *c, -1.0);
        }

        for (childrens_classes, &node_active) in
            class.childrens_classes.iter().zip(&class.variables)
        {
            for child_active in childrens_classes_var(childrens_classes) {
                // node active implies child active, encoded as:
                //   node_active <= child_active
                //   node_active - child_active <= 0
                if !intersection.contains(&child_active) {
                    let row = model.add_row();
                    model.set_row_upper(row, 0.0);
                    model.set_weight(row, node_active, 1.0);
                    model.set_weight(row, child_active, -1.0);
                }
            }
        }
    }

    for root in &roots {
        model.set_col_lower(vars[root].active, 1.0);
    }

    let mut objective_fn_terms = 0;

    for (_class_id, c_var) in &vars {
        let mut min_cost = 0.0;

        /* Moves the minimum of all the nodes up onto the class.
        Most helpful when the members of the class all have the same cost.
        For example if the members' costs are [1,1,1], three terms get
        replaced by one in the objective function.
        */

        if config.move_min_cost_of_members_to_class {
            min_cost = c_var
                .costs
                .iter()
                .min()
                .unwrap_or(&Cost::default())
                .into_inner();
        }

        if min_cost != 0.0 {
            model.set_obj_coeff(c_var.active, min_cost);
            objective_fn_terms += 1;
        }

        for (&node_active, &node_cost) in c_var.variables.iter().zip(c_var.costs.iter()) {
            if *node_cost - min_cost != 0.0 {
                model.set_obj_coeff(node_active, *node_cost - min_cost);
            }
        }
    }

    log::info!("Objective function terms: {}", objective_fn_terms);

    log::info!(
        "Time spent before solving: {}ms",
        simp_start_time.elapsed().as_millis()
    );

    loop {
        // Set the solver limit based on how long has passed already.
        let difference = start_time.elapsed().as_secs();
        let seconds = timeout.saturating_sub(difference.try_into().unwrap());
        model.set_parameter("seconds", &seconds.to_string());

        //This starts from scratch solving each time. I've looked quickly
        //at the API and didn't see how to call it incrementally.
        let solution = model.solve();
        log::info!(
            "CBC status {:?}, {:?}, obj = {}",
            solution.raw().status(),
            solution.raw().secondary_status(),
            solution.raw().obj_value(),
        );

        if solution.raw().is_proven_infeasible() {
            log::info!("Infeasible, returning empty solution");
            return Some(ExtractionResult::default());
        }

        let stopped_without_finishing = solution.raw().status() != coin_cbc::raw::Status::Finished;

        if stopped_without_finishing {
            log::info!("CBC stopped before finishing");

            return None;
        }

        let mut cost = 0.0;
        for (id, var) in &vars {
            let active = solution.col(var.active) > 0.0;

            if active {
                assert!(var.members() > 0);
                let mut node_idx = 0;
                if var.members() != 1 {
                    assert_eq!(
                        1,
                        var.variables
                            .iter()
                            .filter(|&n| solution.col(*n) > 0.0)
                            .count()
                    );

                    node_idx = var
                        .variables
                        .iter()
                        .position(|&n| solution.col(n) > 0.0)
                        .unwrap();
                }

                let node_id = var.members[node_idx].clone();
                cost += var.costs[node_idx].into_inner();
                result.choose(id.clone(), node_id);
            }
        }

        let cycles = find_cycles_in_result(&result, &vars, &roots);

        log::info!("Cost of solution {cost}");
        log::info!("Initial result {}", initial_result_cost.into_inner());
        log::info!("Cost of extraction {}", result.dag_cost(egraph, &roots));
        log::info!("Cost from solver {}", solution.raw().obj_value());

        if stopped_without_finishing {
            log::info!("Timed out");
            return None;
        }

        if cycles.is_empty() {
            assert!(cost <= initial_result_cost.into_inner() + EPSILON_ALLOWANCE);
            assert!((result.dag_cost(egraph, &roots) - cost).abs() < EPSILON_ALLOWANCE);
            assert!((cost - solution.raw().obj_value()).abs() < EPSILON_ALLOWANCE);

            return Some(result);
        } else {
            log::info!("Refining by blocking cycles: {}", cycles.len());
            for c in &cycles {
                block_cycle(&mut model, c, &vars);
            }
        }

        if false {
            //config.initialise_with_previous_solution

            // This is a bit complicated.

            //First, The COIN-OR CBC interface has this function
            //model.set_initial_solution(&solution);
            //But it crashes if the model has more columns than the solution does, which
            //happens if we've just blocked cycles.

            // Second, when used before solving, the ILP solver was sometimes unsound.
            // I didn't see unsound results from the ILP solver using this function here, but
            // it makes me wary, plus it doesn't speed up things noticeably.
            set_initial_solution(&vars, &mut model, &result);
        }
    }
}

/*
Using this caused wrong results from the solver. I don't have a good idea why.
*/
fn set_initial_solution(
    vars: &IndexMap<ClassId, ClassILP>,
    model: &mut Model,
    initial_result: &ExtractionResult,
) {
    for (class, class_vars) in vars {
        for col in class_vars.variables.clone() {
            model.set_col_initial_solution(col, 0.0);
        }

        if let Some(node_id) = initial_result.choices.get(class) {
            model.set_col_initial_solution(class_vars.active, 1.0);
            if let Some(var) = vars[class].get_variable_for_node(node_id) {
                model.set_col_initial_solution(var, 1.0);
            }
        } else {
            model.set_col_initial_solution(class_vars.active, 0.0);
        }
    }
}

/* If a class has one node, and that node is zero cost, and it has no children, then we
can fill the answer into the extraction result without doing any more work. If it
has children, we need to setup the dependencies.

Intuitively, whenever we find a class that has a single node that is zero cost, our work
is done, we can't do any better for that class, so we can select it. Additionally, we
don't care if any other node depends on this class, because this class is zero cost,
we can ignore all references to it.

This is really like deleting empty classes, except there we delete the parent classes,
and here we delete just children of nodes in the parent classes.

*/
fn remove_single_zero_cost(
    vars: &mut IndexMap<ClassId, ClassILP>,
    extraction_result: &mut ExtractionResult,
    roots: &[ClassId],
    config: &Config,
) {
    if config.remove_single_zero_cost {
        let mut zero: FxHashSet<ClassId> = Default::default();
        for (class_id, details) in &*vars {
            if details.childrens_classes.len() == 1
                && details.childrens_classes[0].is_empty()
                && details.costs[0] == 0.0
                && !roots.contains(&class_id.clone())
            {
                zero.insert(class_id.clone());
            }
        }

        if zero.is_empty() {
            return;
        }

        let mut removed = 0;
        let mut extras = 0;
        let fresh = IndexSet::<ClassId>::new();
        let child_to_parents = child_to_parents(vars);

        // Remove all references to those in zero.
        for e in &zero {
            let parents = child_to_parents.get(e).unwrap_or(&fresh);
            for parent in parents {
                for i in (0..vars[parent].childrens_classes.len()).rev() {
                    if vars[parent].childrens_classes[i].contains(e) {
                        vars[parent].childrens_classes[i].swap_remove(e);
                        removed += 1;
                    }
                }

                // Like with empty classes, we might have discovered a new candidate class.
                // It's rare in our benchmarks so I haven't implemented it yet.
                if vars[parent].childrens_classes.len() == 1
                    && vars[parent].childrens_classes[0].is_empty()
                    && vars[parent].costs[0] == 0.0
                    && !roots.contains(&e.clone())
                {
                    extras += 1;
                    // this should be called in a loop like we delete empty classes.
                }
            }
        }
        // Add into the extraction result
        for e in &zero {
            extraction_result.choose(e.clone(), vars[e].members[0].clone());
        }

        // Remove the classes themselves.
        vars.retain(|class_id, _| !zero.contains(class_id));

        log::info!(
            "Zero cost & zero children removed: {} links removed: {removed}, extras:{extras}",
            zero.len()
        );
    }
}

fn child_to_parents(vars: &IndexMap<ClassId, ClassILP>) -> IndexMap<ClassId, IndexSet<ClassId>> {
    let mut child_to_parents: IndexMap<ClassId, IndexSet<ClassId>> = IndexMap::new();

    for (class_id, class_vars) in vars.iter() {
        for kids in &class_vars.childrens_classes {
            for child_class in kids {
                child_to_parents
                    .entry(child_class.clone())
                    .or_default()
                    .insert(class_id.clone());
            }
        }
    }
    child_to_parents
}

/* If a node in a class has (a) equal or higher cost compared to another in that same class, and (b) its
  children are a superset of the other's, then it can be removed.
*/
fn remove_more_expensive_subsumed_nodes(vars: &mut IndexMap<ClassId, ClassILP>, config: &Config) {
    if config.remove_more_expensive_subsumed_nodes {
        let mut removed = 0;

        for class in vars.values_mut() {
            let mut children = class.as_nodes();
            children.sort_by_key(|e| (e.children_classes.len(), e.cost));

            let mut i = 0;
            while i < children.len() {
                for j in ((i + 1)..children.len()).rev() {
                    let node_b = &children[j];

                    // This removes some extractions with the same cost.
                    if children[i].cost <= node_b.cost
                        && children[i]
                            .children_classes
                            .is_subset(&node_b.children_classes)
                    {
                        class.remove_node(&node_b.member.clone());
                        children.remove(j);
                        removed += 1;
                    }
                }
                i += 1;
            }
        }

        log::info!("Removed more expensive subsumed nodes: {removed}");
    }
}

// Remove any classes that can't be reached from a root.
fn remove_unreachable_classes(
    vars: &mut IndexMap<ClassId, ClassILP>,
    roots: &[ClassId],
    config: &Config,
) {
    if config.remove_unreachable_classes {
        let mut reachable_classes: IndexSet<ClassId> = IndexSet::default();
        reachable(&*vars, roots, &mut reachable_classes);
        let initial_size = vars.len();
        vars.retain(|class_id, _| reachable_classes.contains(class_id));
        log::info!("Unreachable classes: {}", initial_size - vars.len());
    }
}

// Any node that has an empty class as a child, can't be selected, so remove the node,
// if that makes another empty class, then remove its parents
fn remove_empty_classes(vars: &mut IndexMap<ClassId, ClassILP>, config: &Config) {
    if config.remove_empty_classes {
        let mut empty_classes: std::collections::VecDeque<ClassId> = Default::default();
        for (classid, detail) in vars.iter() {
            if detail.members() == 0 {
                empty_classes.push_back(classid.clone());
            }
        }

        let mut removed = 0;
        let fresh = IndexSet::<ClassId>::new();

        let mut child_to_parents: IndexMap<ClassId, IndexSet<ClassId>> = IndexMap::new();

        for (class_id, class_vars) in vars.iter() {
            for kids in &class_vars.childrens_classes {
                for child_class in kids {
                    child_to_parents
                        .entry(child_class.clone())
                        .or_default()
                        .insert(class_id.clone());
                }
            }
        }

        let mut done = FxHashSet::<ClassId>::default();

        while let Some(e) = empty_classes.pop_front() {
            if !done.insert(e.clone()) {
                continue;
            }
            let parents = child_to_parents.get(&e).unwrap_or(&fresh);
            for parent in parents {
                for i in (0..vars[parent].childrens_classes.len()).rev() {
                    if vars[parent].childrens_classes[i].contains(&e) {
                        vars[parent].remove(i);
                        removed += 1;
                    }
                }

                if vars[parent].members() == 0 {
                    empty_classes.push_back(parent.clone());
                }
            }
        }

        log::info!("Nodes removed that point to empty classes: {}", removed);
    }
}

// Any class that is a child of each node in a root, is also a root.
fn find_extra_roots(
    vars: &mut IndexMap<ClassId, ClassILP>,
    roots: &mut Vec<ClassId>,
    config: &Config,
) {
    if config.find_extra_roots {
        let mut extra = 0;
        let mut i = 0;
        // newly added roots will also be processed in one pass through.
        while i < roots.len() {
            let r = roots[i].clone();

            let details = vars.get(&r).unwrap();
            if details.childrens_classes.is_empty() {
                continue;
            }

            let mut intersection = details.childrens_classes[0].clone();

            for childrens_classes in &details.childrens_classes[1..] {
                intersection = intersection
                    .intersection(childrens_classes)
                    .cloned()
                    .collect();
            }

            for r in &intersection {
                if !roots.contains(r) {
                    roots.push(r.clone());
                    extra += 1;
                }
            }
            i += 1;
        }

        log::info!("Extra roots discovered: {extra}");
    }
}

/*
For each class with one parent, move the minimum costs of the members to each node in the parent that points to it.

if we iterated through these in order, from child to parent, to parent, to parent.. it could be done in one pass.
*/
fn pull_up_costs(vars: &mut IndexMap<ClassId, ClassILP>, roots: &[ClassId], config: &Config) {
    if config.pull_up_costs {
        let mut count = 0;
        let mut changed = true;
        let child_to_parent = classes_with_single_parent(&*vars);

        while (count < 10) && changed {
            log::info!("Classes with a single parent: {}", child_to_parent.len());
            changed = false;
            count += 1;
            for (child, parent) in &child_to_parent {
                if child == parent {
                    continue;
                }
                if roots.contains(child) {
                    continue;
                }
                if vars[child].members() == 0 {
                    continue;
                }

                // Get the minimum cost of members of the children
                let min_cost = vars[child]
                    .costs
                    .iter()
                    .min()
                    .unwrap_or(&Cost::default())
                    .into_inner();

                assert!(min_cost >= 0.0);
                if min_cost == 0.0 {
                    continue;
                }
                changed = true;

                // Now remove it from each member
                for c in &mut vars[child].costs {
                    *c -= min_cost;
                    assert!(c.into_inner() >= 0.0);
                }
                // Add it onto each node in the parent that refers to this class.
                let indices: Vec<_> = vars[parent]
                    .childrens_classes
                    .iter()
                    .enumerate()
                    .filter(|&(_, c)| c.contains(child))
                    .map(|(id, _)| id)
                    .collect();

                assert!(!indices.is_empty());

                for id in indices {
                    vars[parent].costs[id] += min_cost;
                }
            }
        }
    }
}

/* If a class has a single parent class,
then move the children from the child to the parent class.

There could be a long chain of single parent classes - which this handles
(badly) by looping through a few times.

*/

fn pull_up_with_single_parent(
    vars: &mut IndexMap<ClassId, ClassILP>,
    roots: &[ClassId],
    config: &Config,
) {
    if config.pull_up_single_parent {
        for _i in 0..10 {
            let child_to_parent = classes_with_single_parent(&*vars);
            log::info!("Classes with a single parent: {}", child_to_parent.len());

            let mut pull_up_count = 0;
            for (child, parent) in &child_to_parent {
                if child == parent {
                    continue;
                }

                if roots.contains(child) {
                    continue;
                }

                if vars[child].members.len() != 1 {
                    continue;
                }

                if vars[child].childrens_classes.first().unwrap().is_empty() {
                    continue;
                }

                let found = vars[parent]
                    .childrens_classes
                    .iter()
                    .filter(|c| c.contains(child))
                    .count();

                if found != 1 {
                    continue;
                }

                let idx = vars[parent]
                    .childrens_classes
                    .iter()
                    .position(|e| e.contains(child))
                    .unwrap();

                let child_descendants = vars
                    .get(child)
                    .unwrap()
                    .childrens_classes
                    .first()
                    .unwrap()
                    .clone();

                let parent_descendants: &mut IndexSet<ClassId> = vars
                    .get_mut(parent)
                    .unwrap()
                    .childrens_classes
                    .get_mut(idx)
                    .unwrap();

                for e in &child_descendants {
                    parent_descendants.insert(e.clone());
                }

                vars.get_mut(child)
                    .unwrap()
                    .childrens_classes
                    .first_mut()
                    .unwrap()
                    .clear();

                pull_up_count += 1;
            }
            log::info!("Pull up count: {pull_up_count}");
            if pull_up_count == 0 {
                break;
            }
        }
    }
}

// Remove any nodes that alone cost more than the total of a solution.
// For example, if the lowest the sum of roots can be is 12, and we've found an approximate
// solution already that is 15, then any non-root node that costs more than 3 can't be selected
// in the optimal solution.

fn remove_high_cost(
    vars: &mut IndexMap<ClassId, ClassILP>,
    initial_result_cost: NotNan<f64>,
    roots: &[ClassId],
    config: &Config,
) {
    if config.remove_high_cost_nodes {
        debug_assert_eq!(
            roots.len(),
            roots.iter().collect::<std::collections::HashSet<_>>().len(),
            "All ClassId in roots must be unique"
        );

        let lowest_root_cost_sum: Cost = roots
            .iter()
            .filter_map(|root| vars[root].costs.iter().min())
            .sum();

        let mut removed = 0;

        for (class_id, class_details) in vars.iter_mut() {
            for i in (0..class_details.costs.len()).rev() {
                let cost = &class_details.costs[i];
                let this_root: Cost = if roots.contains(class_id) {
                    *class_details.costs.iter().min().unwrap()
                } else {
                    Cost::default()
                };

                if cost
                    > &(initial_result_cost - lowest_root_cost_sum + this_root + EPSILON_ALLOWANCE)
                {
                    class_details.remove(i);
                    removed += 1;
                }
            }
        }
        log::info!("Removed high-cost nodes: {}", removed);
    }
}

// Remove nodes with any (a) child pointing back to its own class,
// or (b) any child pointing to the sole root class.
fn remove_with_loops(vars: &mut IndexMap<ClassId, ClassILP>, roots: &[ClassId], config: &Config) {
    if config.remove_self_loops {
        let mut removed = 0;
        for (class_id, class_details) in vars.iter_mut() {
            for i in (0..class_details.childrens_classes.len()).rev() {
                if class_details.childrens_classes[i]
                    .iter()
                    .any(|cid| *cid == *class_id || (roots.len() == 1 && roots[0] == *cid))
                {
                    class_details.remove(i);
                    removed += 1;
                }
            }
        }

        log::info!("Omitted looping nodes: {}", removed);
    }
}

// Mapping from child class to parent classes
fn classes_with_single_parent(vars: &IndexMap<ClassId, ClassILP>) -> IndexMap<ClassId, ClassId> {
    let mut child_to_parents: IndexMap<ClassId, IndexSet<ClassId>> = IndexMap::new();

    for (class_id, class_vars) in vars.iter() {
        for kids in &class_vars.childrens_classes {
            for child_class in kids {
                child_to_parents
                    .entry(child_class.clone())
                    .or_default()
                    .insert(class_id.clone());
            }
        }
    }

    // return classes with only one parent
    child_to_parents
        .into_iter()
        .filter_map(|(child_class, parents)| {
            if parents.len() == 1 {
                Some((child_class, parents.into_iter().next().unwrap()))
            } else {
                None
            }
        })
        .collect()
}

//Set of classes that can be reached from the [classes]
fn reachable(
    vars: &IndexMap<ClassId, ClassILP>,
    classes: &[ClassId],
    is_reachable: &mut IndexSet<ClassId>,
) {
    for class in classes {
        if is_reachable.insert(class.clone()) {
            let class_vars = vars.get(class).unwrap();
            for kids in &class_vars.childrens_classes {
                for child_class in kids {
                    reachable(vars, &[child_class.clone()], is_reachable);
                }
            }
        }
    }
}

// Adds constraints to stop the cycle.
fn block_cycle(model: &mut Model, cycle: &[ClassId], vars: &IndexMap<ClassId, ClassILP>) {
    if cycle.is_empty() {
        return;
    }
    let mut blocking = Vec::new();
    for i in 0..cycle.len() {
        let current_class_id = &cycle[i];
        let next_class_id = &cycle[(i + 1) % cycle.len()];

        let mut this_level = Vec::default();
        for node in &vars[current_class_id].as_nodes() {
            if node.children_classes.contains(next_class_id) {
                this_level.push(node.variable);
            }
        }

        assert!(!this_level.is_empty());

        if this_level.len() == 1 {
            blocking.push(this_level[0]);
        } else {
            let blocking_var = model.add_binary();
            blocking.push(blocking_var);
            for n in this_level {
                let row = model.add_row();
                model.set_row_upper(row, 0.0);
                model.set_weight(row, n, 1.0);
                model.set_weight(row, blocking_var, -1.0);
            }
        }
    }

    //One of the edges between nodes in the cycle shouldn't be activated:
    let row = model.add_row();
    model.set_row_upper(row, blocking.len() as f64 - 1.0);
    for b in blocking {
        model.set_weight(row, b, 1.0)
    }
}

#[derive(Clone)]
enum TraverseStatus {
    Doing,
    Done,
}

/*
Returns the simple cycles possible from the roots.

Because the number of simple cycles can be factorial in the number
of nodes, this can be very slow.

Imagine a 20 node complete graph with one root. From the first node you have
19 choices, then from the second 18 choices, etc.  When you get to the second
last node you go back to the root. There are about 10^17 length 18 cycles.

So we limit how many can be found.
*/
const CYCLE_LIMIT: usize = 1000;

fn find_cycles_in_result(
    extraction_result: &ExtractionResult,
    vars: &IndexMap<ClassId, ClassILP>,
    roots: &[ClassId],
) -> Vec<Vec<ClassId>> {
    let mut status = IndexMap::<ClassId, TraverseStatus>::default();
    let mut cycles = vec![];
    for root in roots {
        let mut stack = vec![];
        cycle_dfs(
            extraction_result,
            vars,
            root,
            &mut status,
            &mut cycles,
            &mut stack,
        )
    }
    cycles
}

fn cycle_dfs(
    extraction_result: &ExtractionResult,
    vars: &IndexMap<ClassId, ClassILP>,
    class_id: &ClassId,
    status: &mut IndexMap<ClassId, TraverseStatus>,
    cycles: &mut Vec<Vec<ClassId>>,
    stack: &mut Vec<ClassId>,
) {
    match status.get(class_id).cloned() {
        Some(TraverseStatus::Done) => (),
        Some(TraverseStatus::Doing) => {
            // Get the part of the stack between the first visit to the class and now.
            let mut cycle = vec![];
            if let Some(pos) = stack.iter().position(|id| id == class_id) {
                cycle.extend_from_slice(&stack[pos..]);
            }
            cycles.push(cycle);
        }
        None => {
            if cycles.len() > CYCLE_LIMIT {
                return;
            }
            status.insert(class_id.clone(), TraverseStatus::Doing);
            stack.push(class_id.clone());
            let node_id = &extraction_result.choices[class_id];
            for child_cid in vars[class_id].get_children_of_node(node_id) {
                cycle_dfs(extraction_result, vars, child_cid, status, cycles, stack)
            }
            let last = stack.pop();
            assert_eq!(*class_id, last.unwrap());
            status.insert(class_id.clone(), TraverseStatus::Done);
        }
    }
}

#[derive(Default, Clone)]
pub struct ExtractionResult {
    pub choices: IndexMap<ClassId, NodeId>,
}

impl ExtractionResult {
    pub fn choose(&mut self, class_id: ClassId, node_id: NodeId) {
        self.choices.insert(class_id, node_id);
    }

    // this will loop if there are cycles
    pub fn dag_cost(&self, egraph: &EGraph, roots: &[ClassId]) -> Cost {
        let mut costs: IndexMap<ClassId, Cost> = IndexMap::new();
        let mut todo: Vec<ClassId> = roots.to_vec();
        while let Some(cid) = todo.pop() {
            let node_id = &self.choices[&cid];
            let node = &egraph[node_id];
            if costs.insert(cid.clone(), node.cost).is_some() {
                continue;
            }
            for child in &node.children {
                todo.push(egraph.nid_to_cid(child).clone());
            }
        }
        costs.values().sum()
    }

    pub fn check(&self, egraph: &EGraph) {
        // should be a root
        assert!(!egraph.root_eclasses.is_empty());

        // All roots should be selected.
        for cid in egraph.root_eclasses.iter() {
            assert!(self.choices.contains_key(cid));
        }

        // No cycles
        assert!(self.find_cycles(egraph, &egraph.root_eclasses).is_empty());

        // Nodes should match the class they are selected into.
        for (cid, nid) in &self.choices {
            let node = &egraph[nid];
            assert!(node.eclass == *cid);
        }

        // All the nodes the roots depend upon should be selected.
        let mut todo: Vec<ClassId> = egraph.root_eclasses.to_vec();
        let mut visited: FxHashSet<ClassId> = Default::default();
        while let Some(cid) = todo.pop() {
            if !visited.insert(cid.clone()) {
                continue;
            }
            assert!(self.choices.contains_key(&cid));

            for child in &egraph[&self.choices[&cid]].children {
                todo.push(egraph.nid_to_cid(child).clone());
            }
        }
    }

    pub fn find_cycles(&self, egraph: &EGraph, roots: &[ClassId]) -> Vec<ClassId> {
        // let mut status = vec![Status::Todo; egraph.classes().len()];
        let mut status = IndexMap::<ClassId, Status>::default();
        let mut cycles = vec![];
        for root in roots {
            // let root_index = egraph.classes().get_index_of(root).unwrap();
            self.cycle_dfs(egraph, root, &mut status, &mut cycles)
        }
        cycles
    }

    fn cycle_dfs(
        &self,
        egraph: &EGraph,
        class_id: &ClassId,
        status: &mut IndexMap<ClassId, Status>,
        cycles: &mut Vec<ClassId>,
    ) {
        match status.get(class_id).cloned() {
            Some(Status::Done) => (),
            Some(Status::Doing) => cycles.push(class_id.clone()),
            None => {
                status.insert(class_id.clone(), Status::Doing);
                let node_id = &self.choices[class_id];
                let node = &egraph[node_id];
                for child in &node.children {
                    let child_cid = egraph.nid_to_cid(child);
                    self.cycle_dfs(egraph, child_cid, status, cycles)
                }
                status.insert(class_id.clone(), Status::Done);
            }
        }
    }
}

// Allowance for floating point values to be considered equal
pub const EPSILON_ALLOWANCE: f64 = 0.00001;

#[derive(Clone, Copy)]
enum Status {
    Doing,
    Done,
}
