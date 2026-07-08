// Port of ilp.h / ilp.cpp — ILP encoding + Gurobi/CBC subprocess driver.
// Direct line-by-line translation.
//
// NOTES on translation choices that DIVERGE from a literal mirror:
//  * The C++ uses fork()/SIGTERM-style timing only indirectly via std::system
//    plus the solver's own time-limit flag. We mirror that: spawn the solver
//    with std::process::Command and use wait-timeout to enforce
//    g_config.ilp_timeout_seconds as a hard wall-clock cap on top of the
//    solver's flag. SIGCHLD handlers etc. are not needed in Rust because
//    Command::wait already synchronously reaps the child.
//  * mkstemps -> tempfile::Builder with .lp / .sol / .log suffixes.

use std::collections::{BTreeSet, VecDeque};
use std::fs::File;
use std::io::Read as _;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use indexmap::{IndexMap, IndexSet};
use tempfile::Builder as TempBuilder;
use wait_timeout::ChildExt;

use crate::config::g_config;
use crate::egraphin::{
    EClass, EClassId, EGraph, EGraphMapping, ENode, ENodeId, Extraction, ExtractionENode,
    ExtractionENodeId,
};
use crate::greedy::{compute_statewalk_cost, get_enode_cost, project_statewalk_cost, Cost};

// Forward declarations from ilp.cpp (defined later in this file):
//   pair<EClassId, ENodeId> findArg(const EGraph &g);
//   bool validExtraction(const EGraph &g, const EClassId root, const Extraction &e);

struct SolverSolution {
    values: IndexMap<String, f64>,
    infeasible: bool,
}

impl SolverSolution {
    fn new() -> Self {
        SolverSolution {
            values: IndexMap::new(),
            infeasible: false,
        }
    }
}

#[inline]
fn is_space_char(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == '\x0c' || ch == '\x0b'
}

fn trim_copy(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut start: usize = 0;
    while start < bytes.len() && is_space_char(bytes[start] as char) {
        start += 1;
    }
    let mut end: usize = bytes.len();
    while end > start && is_space_char(bytes[end - 1] as char) {
        end -= 1;
    }
    s[start..end].to_string()
}

fn lowercase_ascii(mut s: String) -> String {
    // Mirror the in-place ASCII lowercase from the C++.
    unsafe {
        let bytes = s.as_bytes_mut();
        for ch in bytes.iter_mut() {
            if *ch >= b'A' && *ch <= b'Z' {
                *ch = *ch - b'A' + b'a';
            }
        }
    }
    s
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    let hay_lower = lowercase_ascii(haystack.to_string());
    let needle_lower = lowercase_ascii(needle.to_string());
    hay_lower.find(&needle_lower).is_some()
}

// template <typename FailFn>
// static SolverSolution parse_solver_solution(...);
fn parse_solver_solution<F: FnMut(String)>(
    sol_path: &str,
    solver_log: &str,
    solver_name: &str,
    solver_uses_xml: bool,
    mut fail_with_log: F,
) -> SolverSolution {
    let _ = solver_uses_xml;
    let mut result = SolverSolution::new();
    let sol_contents = match std::fs::read_to_string(sol_path) {
        Ok(s) => s,
        Err(_) => {
            fail_with_log(format!("failed to open {} solution file", solver_name));
            return result;
        }
    };
    let mut lines: Vec<String> = Vec::new();
    let mut has_content = false;
    for line in sol_contents.split('\n') {
        let line_string = line.to_string();
        // C++ getline strips the trailing '\n' so we mirror that by splitting on '\n'.
        // However split('\n') produces an extra empty trailing element when the
        // file ends with '\n'. The C++ getline naturally stops at EOF, so we
        // detect that case by skipping a trailing empty produced by a final '\n'.
        lines.push(line_string);
    }
    // Drop the synthetic trailing empty line if the file ended with '\n'.
    if sol_contents.ends_with('\n') {
        lines.pop();
    }
    for line in &lines {
        if !line.is_empty() {
            has_content = true;
        }
        if line.find("Infeasible").is_some()
            || line.find("infeasible").is_some()
            || line.find("INFEASIBLE").is_some()
        {
            result.infeasible = true;
        }
    }
    let log_mentions_infeasible = contains_case_insensitive(solver_log, "infeasible");
    if !has_content {
        if log_mentions_infeasible {
            result.infeasible = true;
            return result;
        }
        if !result.infeasible {
            fail_with_log(format!(
                "{} produced an empty solution file",
                solver_name
            ));
        }
    }
    if result.infeasible {
        return result;
    }

    for raw_line in &lines {
        let trimmed = trim_copy(raw_line);
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.as_bytes()[0] == b'#' {
            continue;
        }
        let lower_trimmed = lowercase_ascii(trimmed.clone());
        if lower_trimmed.find("objective value").is_some()
            || lower_trimmed.find("solution status").is_some()
            || lower_trimmed.find("solution time").is_some()
        {
            continue;
        }
        let tokens: Vec<String> = trimmed.split_whitespace().map(|t| t.to_string()).collect();
        if tokens.is_empty() {
            continue;
        }
        if tokens.len() >= 2 {
            let first_byte = tokens[0].as_bytes()[0];
            let starts_alpha_or_underscore = (first_byte as char).is_ascii_alphabetic()
                || first_byte == b'_'
                || tokens[0].find('(').is_some();
            if starts_alpha_or_underscore {
                match tokens[1].parse::<f64>() {
                    Ok(value) => {
                        result.values.insert(tokens[0].clone(), value);
                        continue;
                    }
                    Err(_) => {}
                }
            }
        }
        if tokens.len() >= 3 {
            match tokens[2].parse::<f64>() {
                Ok(value) => {
                    result.values.insert(tokens[1].clone(), value);
                    continue;
                }
                Err(_) => {}
            }
        }
    }
    if result.values.is_empty() && log_mentions_infeasible {
        result.infeasible = true;
    }
    result
}

pub fn enode_to_eclass(g: &EGraph, n: ENodeId) -> EClassId {
    for c in 0..(g.eclasses.len() as EClassId) {
        for m in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            if n == m {
                return c;
            }
        }
    }
    -1
}

pub fn print_enode<W: Write>(out: &mut W, n: &ENode) {
    let _ = write!(out, "{}(", n.head);
    for i in 0..(n.ch.len() as i32) {
        if i > 0 {
            let _ = write!(out, ",");
        }
        let _ = write!(out, "{}", n.ch[i as usize]);
    }
    let _ = write!(out, ")");
}

pub fn print_eclass<W: Write>(out: &mut W, g: &EGraph, c: EClassId) {
    let _ = write!(
        out,
        "EClass {}{}:\n",
        c,
        if g.eclasses[c as usize].isEffectful {
            " (effectful)"
        } else {
            ""
        }
    );
    for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
        let _ = write!(out, "  ");
        print_enode(out, &g.eclasses[c as usize].enodes[n as usize]);
        let _ = write!(out, "\n");
    }
}

// We need Write in scope for the helpers above.
use std::io::Write;

// ---------------------------------------------------------------------------
// anonymous namespace helpers (translated as module-private items)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ChoiceVar {
    name: String,
    parent_class: EClassId,
    parent_node: ENodeId,
    child_idx: i32,
    child_class: EClassId,
    child_node: ENodeId,
}

#[inline]
fn encode_child_selection_key(cls: EClassId, node: ENodeId) -> i64 {
    ((cls as i64) << 32) | ((node as u32) as i64)
}

fn build_binary_value_map(
    pickNode: &Vec<Vec<String>>,
    choices: &Vec<ChoiceVar>,
    raw_values: &IndexMap<String, f64>,
) -> IndexMap<String, bool> {
    let mut estimate: usize = choices.len();
    for row in pickNode.iter() {
        estimate += row.len();
    }
    let mut result: IndexMap<String, bool> = IndexMap::with_capacity(estimate);
    for row in pickNode.iter() {
        for name in row.iter() {
            let mut value: f64 = 0.0;
            if let Some(v) = raw_values.get(name) {
                value = *v;
            }
            // emplace: only inserts if missing.
            if !result.contains_key(name) {
                result.insert(name.clone(), value > 0.5);
            }
        }
    }
    for cv in choices.iter() {
        let mut value: f64 = 0.0;
        if let Some(v) = raw_values.get(&cv.name) {
            value = *v;
        }
        if !result.contains_key(&cv.name) {
            result.insert(cv.name.clone(), value > 0.5);
        }
    }
    result
}

fn require_binary_value<F: FnMut(String)>(
    value_map: &IndexMap<String, bool>,
    name: &str,
    mut fail: F,
) -> bool {
    match value_map.get(name) {
        Some(v) => *v,
        None => {
            fail(format!("missing solver assignment for variable {}", name));
            false
        }
    }
}

fn build_child_selection_for_roots<F: FnMut(String) + Clone>(
    g: &EGraph,
    root_class: EClassId,
    root_enodes: &Vec<ENodeId>,
    pickSelected: &Vec<Vec<i32>>,
    choiceIndex: &Vec<Vec<Vec<Vec<i32>>>>,
    choices: &Vec<ChoiceVar>,
    pickNode: &Vec<Vec<String>>,
    value_map: &IndexMap<String, bool>,
    fail: F,
    childSelection: &mut Vec<Vec<Vec<ENodeId>>>,
) {
    let _ = root_class;
    let _ = root_enodes;
    for c in 0..(g.eclasses.len() as EClassId) {
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            if pickSelected[c as usize][n as usize] == 0 {
                continue;
            }
            let en = &g.eclasses[c as usize].enodes[n as usize];
            for child_idx in 0..(en.ch.len() as i32) {
                let choice_list = &choiceIndex[c as usize][n as usize][child_idx as usize];
                if choice_list.is_empty() {
                    continue;
                }

                let mut chosen_choice_idx: i32 = -1;
                for idx in choice_list.iter() {
                    let chosen = require_binary_value(
                        value_map,
                        &choices[*idx as usize].name,
                        fail.clone(),
                    );
                    if chosen && (chosen_choice_idx == -1 || *idx < chosen_choice_idx) {
                        chosen_choice_idx = *idx;
                    }
                }
                if chosen_choice_idx == -1 {
                    eprint!(
                        "Missing child selection for eclass {} node {} child index {} options:",
                        c, n, child_idx
                    );
                    for idx in choice_list.iter() {
                        let opt_v = require_binary_value(
                            value_map,
                            &choices[*idx as usize].name,
                            fail.clone(),
                        );
                        eprint!(
                            " {}={}",
                            choices[*idx as usize].name,
                            if opt_v { 1 } else { 0 }
                        );
                    }
                    let pick_v = require_binary_value(
                        value_map,
                        &pickNode[c as usize][n as usize],
                        fail.clone(),
                    );
                    eprintln!(" (pickNode={})", if pick_v { 1 } else { 0 });
                    let mut fail_local = fail.clone();
                    fail_local("missing child selection for picked enode".to_string());
                }
                let chosen = &choices[chosen_choice_idx as usize];
                let child_class: EClassId = chosen.child_class;
                let child_node: ENodeId = chosen.child_node;
                if child_node < 0
                    || child_node >= g.eclasses[child_class as usize].enodes.len() as ENodeId
                {
                    let mut fail_local = fail.clone();
                    fail_local("child selection index out of bounds".to_string());
                }
                if pickSelected[child_class as usize][child_node as usize] == 0 {
                    let mut fail_local = fail.clone();
                    fail_local("child enode not marked as picked".to_string());
                }
                childSelection[c as usize][n as usize][child_idx as usize] = child_node;
            }
        }
    }
}

fn build_extraction_node<F: FnMut(String) + Clone>(
    g: &EGraph,
    childSelection: &Vec<Vec<Vec<ENodeId>>>,
    c: EClassId,
    n: ENodeId,
    extraction: &mut Vec<ExtractionENode>,
    nodeIndex: &mut IndexMap<i64, ExtractionENodeId>,
    usedEffectful: &mut BTreeSet<i64>,
    visiting: &mut BTreeSet<i64>,
    fail: F,
) -> ExtractionENodeId {
    let key: i64 = ((c as i64) << 32) | ((n as u32) as i64);
    if let Some(idx) = nodeIndex.get(&key) {
        return *idx;
    }
    if visiting.contains(&key) {
        let mut fail_local = fail.clone();
        fail_local("cycle detected when building extraction".to_string());
    }
    visiting.insert(key);
    let en_ch_len: usize = g.eclasses[c as usize].enodes[n as usize].ch.len();
    let en_ch: Vec<EClassId> = g.eclasses[c as usize].enodes[n as usize].ch.clone();
    let mut ch_idx: Vec<ExtractionENodeId> = Vec::with_capacity(en_ch_len);
    for child_i in 0..(en_ch_len as i32) {
        let child_class: EClassId = en_ch[child_i as usize];
        let child_node: ENodeId =
            childSelection[c as usize][n as usize][child_i as usize];
        if child_node == -1 {
            let mut fail_local = fail.clone();
            fail_local("missing child during extraction reconstruction".to_string());
        }
        let child_ex: ExtractionENodeId = build_extraction_node(
            g,
            childSelection,
            child_class,
            child_node,
            extraction,
            nodeIndex,
            usedEffectful,
            visiting,
            fail.clone(),
        );
        ch_idx.push(child_ex);
    }
    visiting.remove(&key);
    let mut node = ExtractionENode::default();
    node.c = c;
    node.n = n;
    node.ch = ch_idx;
    let idx: ExtractionENodeId = extraction.len() as ExtractionENodeId;
    extraction.push(node);
    nodeIndex.insert(key, idx);
    if g.eclasses[c as usize].isEffectful {
        usedEffectful.insert(key);
    }
    idx
}

// ---------------------------------------------------------------------------
// extractRegionILPInner — the big one
// ---------------------------------------------------------------------------

pub fn extractRegionILPInner(
    g: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
    timed_out: &mut bool,
    infeasible: &mut bool,
    ilp_encoding_num_vars: Option<&mut usize>,
    use_gurobi: bool,
) -> Extraction {
    // C++ uses a closure `fail` that prints + exits. Mirror that behaviour
    // exactly: any failure path during ILP encoding/solver parsing is fatal.
    let fail = |msg: String| -> ! {
        eprintln!("ILP extraction error: {}", msg);
        std::process::exit(1);
    };
    // Cloneable fail for the FnMut(String) callbacks below.
    #[derive(Clone)]
    struct FailFn;
    impl FailFn {
        fn call(&self, msg: String) -> ! {
            eprintln!("ILP extraction error: {}", msg);
            std::process::exit(1);
        }
    }
    let fail_fn = |msg: String| FailFn.call(msg);

    *timed_out = false;
    *infeasible = false;

    let arg: (EClassId, ENodeId) = findArg(g);
    let _initc: EClassId = arg.0;
    let _initn: ENodeId = arg.1;

    /*
    if (root == initc) {
        StateWalk sw;
        sw.push_back(make_pair(root, initn));
        return regionExtractionWithStateWalk(g, root, sw).second;
    }
    */

    // VARIABLES
    // Picking an enode in an eclass
    let mut pickNode: Vec<Vec<String>> = vec![Vec::new(); g.eclasses.len()];
    // Choosing an eclass, enode, child index, and child enode index
    let mut choiceIndex: Vec<Vec<Vec<Vec<i32>>>> = vec![Vec::new(); g.eclasses.len()];
    // Order variables for acyclicity
    let mut orderVar: Vec<Vec<String>> = vec![Vec::new(); g.eclasses.len()];

    // COST MODEL
    // Cost of picking an enode in an eclass
    let mut pickCost: Vec<Vec<i64>> = vec![Vec::new(); g.eclasses.len()];

    // CACHES
    // For each child enode, which choice variables point to it
    let mut childParents: Vec<Vec<Vec<i32>>> = vec![Vec::new(); g.eclasses.len()];
    // Effectful child flow tracking
    // For a given eclass, which choice variables are outgoing/incoming effectful edges
    let mut effectOutgoing: Vec<Vec<i32>> = vec![Vec::new(); g.eclasses.len()];
    let mut effectIncoming: Vec<Vec<i32>> = vec![Vec::new(); g.eclasses.len()];

    let mut total_enodes: i32 = 0;
    for ec in g.eclasses.iter() {
        total_enodes += ec.enodes.len() as i32;
    }
    let maxOrder: i32 = std::cmp::max(1, total_enodes);
    for c in 0..(g.eclasses.len() as EClassId) {
        pickNode[c as usize].resize(g.eclasses[c as usize].enodes.len(), String::new());
        pickCost[c as usize].resize(g.eclasses[c as usize].enodes.len(), 0);
        choiceIndex[c as usize].resize(g.eclasses[c as usize].enodes.len(), Vec::new());
        childParents[c as usize].resize(g.eclasses[c as usize].enodes.len(), Vec::new());
        orderVar[c as usize].resize(g.eclasses[c as usize].enodes.len(), String::new());
    }

    let mut pick_variable_count: usize = 0;
    let mut order_variable_count: usize = 0;

    // All choice variables (a particular edge between an enode at a child index and another enode)
    let mut choices: Vec<ChoiceVar> = Vec::new();
    // initialize choices, pickNode, pickCost, choiceIndex, childParents
    for c in 0..(g.eclasses.len() as EClassId) {
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            pickNode[c as usize][n as usize] = format!("p_{}_{}", c, n);
            orderVar[c as usize][n as usize] = format!("o_{}_{}", c, n);
            pick_variable_count += 1;
            order_variable_count += 1;
            // Snapshot data we need before the inner loop borrows g again.
            let en_ch_len: usize = g.eclasses[c as usize].enodes[n as usize].ch.len();
            let en_ch: Vec<EClassId> = g.eclasses[c as usize].enodes[n as usize].ch.clone();
            let mut node_cost: Cost = {
                let en: &ENode = &g.eclasses[c as usize].enodes[n as usize];
                get_enode_cost(en)
            };
            if g.eclasses[c as usize].isEffectful {
                if (c as usize) >= rstatewalk_cost.len() {
                    fail(format!("statewalk cost missing for effectful eclass {}", c));
                }
                if (n as usize) >= rstatewalk_cost[c as usize].len() {
                    fail(format!(
                        "statewalk cost missing for effectful enode {}:{}",
                        c, n
                    ));
                }
                node_cost = rstatewalk_cost[c as usize][n as usize];
            }
            // C++: min(node_cost, LLONG_MAX). Cost is u64 in Rust; LLONG_MAX is i64::MAX.
            let bounded_cost: Cost = std::cmp::min(node_cost, i64::MAX as Cost);
            pickCost[c as usize][n as usize] = bounded_cost as i64;
            choiceIndex[c as usize][n as usize].resize(en_ch_len, Vec::new());
            for child_idx in 0..(en_ch_len as i32) {
                let child_class: EClassId = en_ch[child_idx as usize];
                if child_class < 0 || child_class >= g.eclasses.len() as EClassId {
                    fail("child eclass index out of bounds".to_string());
                }
                let child_ec_enodes_len: usize = g.eclasses[child_class as usize].enodes.len();
                if child_ec_enodes_len == 0 {
                    fail("child eclass has no enodes to select".to_string());
                }
                {
                    let idx_list: &mut Vec<i32> =
                        &mut choiceIndex[c as usize][n as usize][child_idx as usize];
                    idx_list.reserve(child_ec_enodes_len);
                }
                for m in 0..(child_ec_enodes_len as ENodeId) {
                    let cv = ChoiceVar {
                        name: format!("s_{}_{}_{}_{}", c, n, child_idx, m),
                        parent_class: c,
                        parent_node: n,
                        child_idx,
                        child_class,
                        child_node: m,
                    };
                    let idx: i32 = choices.len() as i32;
                    choices.push(cv);
                    choiceIndex[c as usize][n as usize][child_idx as usize].push(idx);
                    childParents[child_class as usize][m as usize].push(idx);
                    if g.eclasses[c as usize].isEffectful
                        && g.eclasses[child_class as usize].isEffectful
                    {
                        effectOutgoing[c as usize].push(idx);
                        effectIncoming[child_class as usize].push(idx);
                    }
                }
            }
        }
    }

    // Mirror the C++ mkstemps templates with tempfile::Builder.
    // The C++ uses /tmp/extract_regionXXXXXX.{lp,sol,log} — keep the same.
    let lp_named = match TempBuilder::new()
        .prefix("extract_region")
        .suffix(".lp")
        .rand_bytes(6)
        .tempfile_in("/tmp")
    {
        Ok(t) => t,
        Err(_) => {
            fail("failed to create LP temp file".to_string());
        }
    };
    let lp_path: String = lp_named.path().to_string_lossy().to_string();
    // Drop the file handle but keep the file (the solver writes into the same path).
    let lp_keep = lp_named.into_temp_path();

    let sol_named = match TempBuilder::new()
        .prefix("extract_region")
        .suffix(".sol")
        .rand_bytes(6)
        .tempfile_in("/tmp")
    {
        Ok(t) => t,
        Err(_) => {
            // unlink lp file is automatic via lp_keep drop.
            let _ = lp_keep;
            fail("failed to create solution temp file".to_string());
        }
    };
    let sol_path: String = sol_named.path().to_string_lossy().to_string();
    let sol_keep = sol_named.into_temp_path();

    let log_named = match TempBuilder::new()
        .prefix("extract_region")
        .suffix(".log")
        .rand_bytes(6)
        .tempfile_in("/tmp")
    {
        Ok(t) => t,
        Err(_) => {
            let _ = lp_keep;
            let _ = sol_keep;
            fail("failed to create log temp file".to_string());
        }
    };
    let log_path: String = log_named.path().to_string_lossy().to_string();
    let log_keep = log_named.into_temp_path();

    // The C++ FileCleaner RAII wrappers map naturally to Rust's TempPath drop.
    // We simply hold lp_keep/sol_keep/log_keep until the end of scope.

    // Open the LP file for writing.
    let mut lp = match File::create(&lp_path) {
        Ok(f) => f,
        Err(_) => {
            fail("failed to open LP file for writing".to_string());
        }
    };

    let mut firstTerm: bool = true;
    // optionally minimize sum pickCost[c][n] * pickNode[c][n]
    let _ = write!(lp, "Minimize\n");
    if g_config().ilp_minimize_objective {
        let _ = write!(lp, " obj:");
        let mut term_count: i32 = 0;
        for c in 0..(g.eclasses.len() as EClassId) {
            for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
                if !firstTerm {
                    let _ = write!(lp, " + ");
                } else {
                    let _ = write!(lp, " ");
                    firstTerm = false;
                }
                let _ = write!(
                    lp,
                    "{} {}",
                    pickCost[c as usize][n as usize],
                    pickNode[c as usize][n as usize]
                );
                term_count += 1;
                if term_count % 50 == 0 {
                    let _ = write!(lp, "\n");
                }
            }
        }
        if firstTerm {
            let _ = write!(lp, " 0");
        }
        let _ = write!(lp, "\n");
    } else {
        let _ = write!(lp, " obj: 0\n");
    }
    let _ = write!(lp, "Subject To\n");

    // Require exactly one root node
    if root < 0 || root >= g.eclasses.len() as EClassId {
        fail("root eclass out of range".to_string());
    }
    if g.eclasses[root as usize].enodes.is_empty() {
        fail("encountered eclass with no enodes".to_string());
    }
    let _ = write!(lp, " pick_sum_{}:", root);
    let mut first: bool = true;
    for n in 0..(g.eclasses[root as usize].enodes.len() as ENodeId) {
        let _ = write!(
            lp,
            "{}{}",
            if first { " " } else { " + " },
            pickNode[root as usize][n as usize]
        );
        first = false;
    }
    let _ = write!(lp, " = 1\n");

    // If you pick an enode, for every child index pick at least one child edge.
    for c in 0..(g.eclasses.len() as EClassId) {
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            // We snapshot the per-node child index lists to avoid borrow-checker issues.
            let idx_lists_len: usize = choiceIndex[c as usize][n as usize].len();
            for child_idx in 0..(idx_lists_len as i32) {
                let list_len: usize =
                    choiceIndex[c as usize][n as usize][child_idx as usize].len();
                if list_len == 0 {
                    continue;
                }
                let _ = write!(
                    lp,
                    " child_select_{}_{}_{}:",
                    c, n, child_idx
                );
                let mut first: bool = true;
                for k in 0..list_len {
                    let idx: i32 = choiceIndex[c as usize][n as usize][child_idx as usize][k];
                    let _ = write!(
                        lp,
                        "{}{}",
                        if first { " " } else { " + " },
                        choices[idx as usize].name
                    );
                    first = false;
                    // sanity check: assert that the parent eclass of the choice is c and parent_node is n
                    assert!(
                        choices[idx as usize].parent_class == c
                            && choices[idx as usize].parent_node == n
                    );
                }
                let _ = write!(
                    lp,
                    " - {} >= 0\n",
                    pickNode[c as usize][n as usize]
                );
            }
        }
    }
    if let Some(slot) = ilp_encoding_num_vars {
        *slot = choices.len() + pick_variable_count + order_variable_count;
    }

    // If you choose a child edge, you must pick the enode it points to.
    for idx in 0..(choices.len() as i32) {
        let cv = &choices[idx as usize];
        let _ = write!(
            lp,
            " child_link_{}: {} - {} <= 0\n",
            idx, cv.name, pickNode[cv.child_class as usize][cv.child_node as usize]
        );
    }

    // Linearity: effectful enodes may not be targeted by multiple effectful parents.
    for c in 0..(g.eclasses.len() as EClassId) {
        if !g.eclasses[c as usize].isEffectful {
            continue;
        }
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            let parents_len: usize = childParents[c as usize][n as usize].len();
            if parents_len == 0 {
                continue;
            }
            let mut effectful_parents: Vec<i32> = Vec::with_capacity(parents_len);
            for k in 0..parents_len {
                let idx: i32 = childParents[c as usize][n as usize][k];
                if g.eclasses[choices[idx as usize].parent_class as usize].isEffectful {
                    effectful_parents.push(idx);
                }
            }
            if effectful_parents.is_empty() {
                continue;
            }
            let _ = write!(lp, " child_unique_{}_{}:", c, n);
            let mut first: bool = true;
            for idx in effectful_parents.iter() {
                let _ = write!(
                    lp,
                    "{}{}",
                    if first { " " } else { " + " },
                    choices[*idx as usize].name
                );
                first = false;
            }
            let _ = write!(lp, " <= 1\n");
        }
    }

    // Order variables must decrease along chosen edges to prevent cycles.
    // When parent and child are the same enode, forbid taking that edge to avoid duplicate constraints.
    for idx in 0..(choices.len() as i32) {
        let cv = &choices[idx as usize];
        if cv.parent_class == cv.child_class && cv.parent_node == cv.child_node {
            let _ = write!(
                lp,
                " order_edge_{}: {} {} <= {}\n",
                idx,
                maxOrder,
                cv.name,
                maxOrder - 1
            );
        } else {
            let _ = write!(
                lp,
                " order_edge_{}: {} - {} + {} {} <= {}\n",
                idx,
                orderVar[cv.child_class as usize][cv.child_node as usize],
                orderVar[cv.parent_class as usize][cv.parent_node as usize],
                maxOrder,
                cv.name,
                maxOrder - 1
            );
        }
    }

    let _ = write!(lp, "Bounds\n");
    for c in 0..(g.eclasses.len() as EClassId) {
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            let _ = write!(
                lp,
                " 0 <= {} <= {}\n",
                orderVar[c as usize][n as usize],
                maxOrder - 1
            );
        }
    }

    let _ = write!(lp, "Binary\n");
    for c in 0..(g.eclasses.len() as EClassId) {
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            let _ = write!(lp, " {}\n", pickNode[c as usize][n as usize]);
        }
    }
    for cv in choices.iter() {
        let _ = write!(lp, " {}\n", cv.name);
    }
    let _ = write!(lp, "End\n");
    drop(lp); // close the file
    {
        // C++ copies the LP file to /tmp/tiger_last_extract.lp for debugging.
        if let Ok(mut in_debug) = File::open(&lp_path) {
            if let Ok(mut out_debug) = File::create("/tmp/tiger_last_extract.lp") {
                let mut buf: Vec<u8> = Vec::new();
                let _ = in_debug.read_to_end(&mut buf);
                let _ = out_debug.write_all(&buf);
            }
        }
    }

    let solver_name: String = if use_gurobi { "gurobi".to_string() } else { "cbc".to_string() };
    // Build and run the solver subprocess. The C++ uses std::system with a
    // formatted shell string. We use std::process::Command via /bin/sh -c so
    // shell quoting is preserved exactly.
    let timeout_arg: String = g_config().ilp_timeout_seconds.to_string();
    let cmd: String;
    if use_gurobi {
        cmd = format!(
            "gurobi_cl TimeLimit={} Threads=1 ResultFile=\"{}\" LogFile=\"{}\" {} > /dev/null 2>&1",
            timeout_arg, sol_path, log_path, lp_path
        );
    } else {
        cmd = format!(
            "cbc \"{}\" -seconds {} solve solu \"{}\" > \"{}\" 2>&1",
            lp_path, timeout_arg, sol_path, log_path
        );
    }
    let start = Instant::now();
    // Spawn via /bin/sh -c to mirror std::system semantics.
    let mut child = match Command::new("/bin/sh")
        .arg("-c")
        .arg(&cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            fail(format!("{} invocation failed (spawn)", solver_name));
        }
    };
    // Hard wall-clock cap. Solver flag should generally bring it down first.
    let hard_cap = Duration::from_secs(g_config().ilp_timeout_seconds as u64 + 30);
    let ret: i32 = match child.wait_timeout(hard_cap) {
        Ok(Some(status)) => status.code().unwrap_or(-1),
        Ok(None) => {
            // Hit the cap; kill and reap.
            child.kill().unwrap_or(());
            child.wait().ok();
            // Behave as if the solver timed out.
            -1
        }
        Err(_) => {
            child.kill().unwrap_or(());
            child.wait().ok();
            -1
        }
    };
    let elapsed = start.elapsed();
    let elapsed_seconds: f64 = elapsed.as_secs_f64();
    let solver_log: String = std::fs::read_to_string(&log_path).unwrap_or_default();
    {
        if let Ok(mut in_debug_log) = File::open(&log_path) {
            if let Ok(mut out_debug_log) = File::create("/tmp/tiger_last_extract.log") {
                let mut buf: Vec<u8> = Vec::new();
                let _ = in_debug_log.read_to_end(&mut buf);
                let _ = out_debug_log.write_all(&buf);
            }
        }
    }
    // cerr << solver_log << endl;
    let solver_timed_out: bool = contains_case_insensitive(&solver_log, "timeout")
        || contains_case_insensitive(&solver_log, "time limit");

    if solver_timed_out {
        *timed_out = true;
        if !g_config().time_ilp {
            println!("TIMEOUT");
            fail(format!(
                "{} reported a timeout after {} seconds",
                solver_name, elapsed_seconds
            ));
        }
        // Make sure tempfiles get cleaned up before returning.
        drop(lp_keep);
        drop(sol_keep);
        drop(log_keep);
        return Extraction::new();
    }
    if ret != 0 {
        eprintln!("{} log output:\n{}", solver_name, solver_log);
        fail(format!("{} invocation failed", solver_name));
    }
    if solver_log.find("ERROR").is_some() || solver_log.find("Error").is_some() {
        eprintln!("{} log output:\n{}", solver_name, solver_log);
        fail(format!("{} reported an error while solving", solver_name));
    }
    {
        if let Ok(mut in_debug_sol) = File::open(&sol_path) {
            if let Ok(mut out_debug_sol) = File::create("/tmp/tiger_last_extract.sol") {
                let mut buf: Vec<u8> = Vec::new();
                let _ = in_debug_sol.read_to_end(&mut buf);
                let _ = out_debug_sol.write_all(&buf);
            }
        }
    }
    // fail_with_log: print the solver log then exit.
    let solver_log_for_log = solver_log.clone();
    let solver_name_for_log = solver_name.clone();
    let fail_with_log = move |msg: String| {
        eprintln!("{} log output:\n{}", solver_name_for_log, solver_log_for_log);
        eprintln!("ILP extraction error: {}", msg);
        std::process::exit(1);
    };
    let solver_solution: SolverSolution = parse_solver_solution(
        &sol_path,
        &solver_log,
        &solver_name,
        use_gurobi,
        fail_with_log,
    );
    let values: &IndexMap<String, f64> = &solver_solution.values;
    if solver_solution.infeasible {
        *infeasible = true;
        drop(lp_keep);
        drop(sol_keep);
        drop(log_keep);
        return Extraction::new();
    }
    let value_map: IndexMap<String, bool> =
        build_binary_value_map(&pickNode, &choices, values);

    let mut pickSelected: Vec<Vec<i32>> = vec![Vec::new(); g.eclasses.len()];
    for c in 0..(g.eclasses.len() as EClassId) {
        pickSelected[c as usize].clear();
        pickSelected[c as usize].resize(g.eclasses[c as usize].enodes.len(), 0);
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            let selected: bool = require_binary_value(
                &value_map,
                &pickNode[c as usize][n as usize],
                fail_fn,
            );
            if selected {
                pickSelected[c as usize][n as usize] = 1;
            }
        }
    }
    let mut saw_root_assignment: bool = false;
    if !g.eclasses[root as usize].enodes.is_empty() {
        for n in 0..(g.eclasses[root as usize].enodes.len() as ENodeId) {
            let _root_selected: bool = require_binary_value(
                &value_map,
                &pickNode[root as usize][n as usize],
                fail_fn,
            );
            // Mirror the C++ pattern: read the value purely for its side
            // effect of flagging missing-variable failures.
            let _root_value: f64 = if _root_selected { 1.0 } else { 0.0 };
            if values.contains_key(&pickNode[root as usize][n as usize]) {
                saw_root_assignment = true;
            }
        }
    }
    if !saw_root_assignment {
        eprintln!("{} log output:\n{}", solver_name, solver_log);
        fail("solution file did not contain root variable assignments".to_string());
    }

    if pickSelected[root as usize].is_empty() {
        fail("root eclass has no selected enode".to_string());
    }
    let mut root_enodes: Vec<ENodeId> = Vec::new();
    for n in 0..(pickSelected[root as usize].len() as ENodeId) {
        if pickSelected[root as usize][n as usize] != 0 {
            root_enodes.push(n);
        }
    }
    if root_enodes.is_empty() {
        fail("no root enode selected".to_string());
    }
    /*if (!pickSelected[initc].empty() && !pickSelected[initc][initn]) {
        fail("init enode not selected");
    }*/

    let mut childSelection: Vec<Vec<Vec<ENodeId>>> = vec![Vec::new(); g.eclasses.len()];
    for c in 0..(g.eclasses.len() as EClassId) {
        childSelection[c as usize].resize(g.eclasses[c as usize].enodes.len(), Vec::new());
        for n in 0..(g.eclasses[c as usize].enodes.len() as ENodeId) {
            let nch: usize = g.eclasses[c as usize].enodes[n as usize].ch.len();
            childSelection[c as usize][n as usize].clear();
            childSelection[c as usize][n as usize].resize(nch, -1);
        }
    }

    build_child_selection_for_roots(
        g,
        root,
        &root_enodes,
        &pickSelected,
        &choiceIndex,
        &choices,
        &pickNode,
        &value_map,
        fail_fn,
        &mut childSelection,
    );

    let mut extraction: Vec<ExtractionENode> = Vec::new();
    let mut nodeIndex: IndexMap<i64, ExtractionENodeId> = IndexMap::new();
    let mut usedEffectful: BTreeSet<i64> = BTreeSet::new();
    let mut visiting: BTreeSet<i64> = BTreeSet::new();

    for root_node in root_enodes.iter() {
        build_extraction_node(
            g,
            &childSelection,
            root,
            *root_node,
            &mut extraction,
            &mut nodeIndex,
            &mut usedEffectful,
            &mut visiting,
            fail_fn,
        );
    }
    if extraction.is_empty() {
        fail("extraction is empty".to_string());
    }
    if !validExtraction(g, root, &extraction) {
        fail("constructed extraction is invalid".to_string());
    }

    drop(lp_keep);
    drop(sol_keep);
    drop(log_keep);
    extraction
}

pub fn validExtraction(g: &EGraph, root: EClassId, e: &Extraction) -> bool {
    if e.len() == 0 || e.last().unwrap().c != root {
        // root
        eprintln!("Error: The first element of the extraction must be the root.");
        return false;
    }
    let mut i: i32 = (e.len() as i32) - 1;
    while i >= 0 {
        let n: &ExtractionENode = &e[i as usize];
        if n.c < 0 || n.c >= g.eclasses.len() as EClassId {
            eprintln!("Error: Extraction referring to an eclass outside of bounds.");
            return false;
        }
        if n.n < 0 || n.n >= g.eclasses[n.c as usize].enodes.len() as ENodeId {
            eprintln!("Error: Extraction referring to an enode outside of bounds.");
            return false;
        }
        if n.ch.len() != g.eclasses[n.c as usize].enodes[n.n as usize].ch.len() {
            eprintln!("Error: Extraction referring to a wrong number of children.");
            return false;
        }
        for j in 0..(n.ch.len() as i32) {
            let ch: ExtractionENodeId = n.ch[j as usize];
            if ch < 0 || ch >= e.len() as ExtractionENodeId {
                // child present
                eprintln!("Error: Extraction referring to an index outside of bounds.");
                eprintln!("Found: {}", ch);
                return false;
            }
            let expected_child: EClassId =
                g.eclasses[n.c as usize].enodes[n.n as usize].ch[j as usize];
            if e[ch as usize].c != expected_child {
                eprintln!("Error: Extraction referring to a child of wrong eclass.");
                return false;
            }
            if ch >= i {
                // acyclicity
                eprintln!("Error: Extraction may contain a loop.");
                return false;
            }
        }
        i -= 1;
    }
    // reachability does not really matter
    // unique choice not required
    true
}

struct SubEGraphMap {
    eclassmp: Vec<EClassId>,
    inv: std::collections::BTreeMap<EClassId, EClassId>,
    nsubregion: Vec<Vec<i32>>,
    enode_map: Vec<Vec<ENodeId>>,
}

// Prune any nodes that refer to empty eclasses
// Then can happen after we prune nodes that have this same subregion as a child.
fn prune_region_egraph(
    g: &mut EGraph,
    nsubregion: &mut Vec<Vec<i32>>,
    enode_map: &mut Vec<Vec<ENodeId>>,
) {
    assert!(nsubregion.len() == g.eclasses.len());
    assert!(enode_map.len() == g.eclasses.len());

    let mut empty: Vec<bool> = vec![false; g.eclasses.len()];
    for i in 0..g.eclasses.len() {
        empty[i] = g.eclasses[i].enodes.is_empty();
    }

    let mut changed: bool = true;
    while changed {
        changed = false;
        for i in 0..g.eclasses.len() {
            assert!(nsubregion[i].len() == g.eclasses[i].enodes.len());
            assert!(enode_map[i].len() == g.eclasses[i].enodes.len());

            let mut write_idx: usize = 0;
            let n_enodes = g.eclasses[i].enodes.len();
            for j in 0..n_enodes {
                let mut prune: bool = false;
                let ch_len: usize = g.eclasses[i].enodes[j].ch.len();
                for k in 0..ch_len {
                    let child: EClassId = g.eclasses[i].enodes[j].ch[k];
                    if child < 0
                        || child >= g.eclasses.len() as EClassId
                        || empty[child as usize]
                    {
                        prune = true;
                        break;
                    }
                }
                if !prune {
                    if write_idx != j {
                        // enodes[write_idx] = enodes[j] (move/copy)
                        let cloned = g.eclasses[i].enodes[j].clone();
                        g.eclasses[i].enodes[write_idx] = cloned;
                        nsubregion[i][write_idx] = nsubregion[i][j];
                        enode_map[i][write_idx] = enode_map[i][j];
                    }
                    write_idx += 1;
                } else {
                    changed = true;
                }
            }
            if write_idx != g.eclasses[i].enodes.len() {
                g.eclasses[i].enodes.truncate(write_idx);
                nsubregion[i].truncate(write_idx);
                enode_map[i].truncate(write_idx);
            }
            if !empty[i] && g.eclasses[i].enodes.is_empty() {
                empty[i] = true;
                changed = true;
            }
        }
    }
}

fn get_inv_entry_or_fail(
    inv: &std::collections::BTreeMap<EClassId, EClassId>,
    key: EClassId,
    context: Option<&str>,
) -> EClassId {
    match inv.get(&key) {
        Some(v) => *v,
        None => {
            eprint!("Error: SubEGraphMap missing mapping for eclass {}", key);
            if let Some(ctx) = context {
                eprint!(" while {}", ctx);
            }
            eprintln!();
            std::process::exit(1);
        }
    }
}

fn should_skip_region_enode(
    g: &EGraph,
    parent_eclass: EClassId,
    enode: &ENode,
    region_root: EClassId,
) -> bool {
    if !g.eclasses[parent_eclass as usize].isEffectful {
        return false;
    }
    let mut saw_effectful_child: bool = false;
    for child_ref in enode.ch.iter() {
        let child: EClassId = *child_ref;
        if !g.eclasses[child as usize].isEffectful {
            continue;
        }
        if saw_effectful_child && child == region_root {
            return true;
        }
        saw_effectful_child = true;
    }
    false
}

fn createRegionEGraph(g: &EGraph, region_root: EClassId) -> (EGraph, SubEGraphMap) {
    let mut mp = SubEGraphMap {
        eclassmp: Vec::new(),
        inv: std::collections::BTreeMap::new(),
        nsubregion: Vec::new(),
        enode_map: Vec::new(),
    };
    let mut worklist: VecDeque<EClassId> = VecDeque::new();
    // enqueue closure: emulated as a helper function via a local fn-like scope.
    {
        let enqueue = |mp: &mut SubEGraphMap, worklist: &mut VecDeque<EClassId>, c: EClassId| {
            if mp.inv.contains_key(&c) {
                return;
            }
            mp.inv.insert(c, mp.eclassmp.len() as EClassId);
            mp.eclassmp.push(c);
            mp.nsubregion
                .push(vec![0; g.eclasses[c as usize].enodes.len()]);
            mp.enode_map.push(Vec::new());
            worklist.push_back(c);
        };

        enqueue(&mut mp, &mut worklist, region_root);
        while !worklist.is_empty() {
            let u: EClassId = *worklist.front().unwrap();
            worklist.pop_front();
            let u_idx: EClassId = get_inv_entry_or_fail(&mp.inv, u, Some("accessing nsubregion"));
            assert!(mp.nsubregion[u_idx as usize].len() == g.eclasses[u as usize].enodes.len());
            let parent_effectful: bool = g.eclasses[u as usize].isEffectful;
            for i in 0..(g.eclasses[u as usize].enodes.len() as i32) {
                let mut saw_effectful_child: bool = false;
                let ch_len: usize = g.eclasses[u as usize].enodes[i as usize].ch.len();
                for j in 0..(ch_len as i32) {
                    let v: EClassId = g.eclasses[u as usize].enodes[i as usize].ch[j as usize];
                    if g.eclasses[v as usize].isEffectful {
                        if saw_effectful_child {
                            if parent_effectful {
                                mp.nsubregion[u_idx as usize][i as usize] += 1;
                            }
                            continue;
                        }
                        saw_effectful_child = true;
                    }
                    enqueue(&mut mp, &mut worklist, v);
                }
            }
        }
    }

    let mut gr = EGraph::default();
    for i in 0..(mp.eclassmp.len() as i32) {
        let mut c = EClass::default();
        c.isEffectful = g.eclasses[mp.eclassmp[i as usize] as usize].isEffectful;
        let orig_enodes_len = g.eclasses[mp.eclassmp[i as usize] as usize].enodes.len();
        let mut filtered_nsubregion: Vec<i32> = Vec::new();
        let mut filtered_enode_map: Vec<ENodeId> = Vec::new();
        for j in 0..(orig_enodes_len as i32) {
            let orig_node: ENode =
                g.eclasses[mp.eclassmp[i as usize] as usize].enodes[j as usize].clone();
            if should_skip_region_enode(g, mp.eclassmp[i as usize], &orig_node, region_root) {
                continue;
            }
            let mut node = ENode::default();
            node.eclass = i;
            node.head = orig_node.head.clone();
            let mut subregionchild: bool = false;
            for k in 0..(orig_node.ch.len() as i32) {
                let cp: EClassId = orig_node.ch[k as usize];
                if g.eclasses[cp as usize].isEffectful {
                    if subregionchild {
                        continue;
                    }
                    subregionchild = true;
                }
                node.ch.push(get_inv_entry_or_fail(
                    &mp.inv,
                    cp,
                    Some("building region egraph"),
                ));
            }
            c.enodes.push(node);
            filtered_nsubregion.push(mp.nsubregion[i as usize][j as usize]);
            filtered_enode_map.push(j);
        }
        assert!(filtered_nsubregion.len() == c.enodes.len());
        assert!(filtered_enode_map.len() == c.enodes.len());
        mp.nsubregion[i as usize] = filtered_nsubregion;
        mp.enode_map[i as usize] = filtered_enode_map;
        gr.eclasses.push(c);
    }

    prune_region_egraph(&mut gr, &mut mp.nsubregion, &mut mp.enode_map);
    let root_idx: EClassId = get_inv_entry_or_fail(
        &mp.inv,
        region_root,
        Some("getting region root mapping after pruning region egraph"),
    );
    if gr.eclasses[root_idx as usize].enodes.is_empty() {
        eprintln!(
            "Error: Region root eclass {} became empty after pruning invalid enodes.",
            region_root
        );
        std::process::exit(1);
    }
    (gr, mp)
}

pub fn checkLinearRegionRec(g: &EGraph, rootid: ExtractionENodeId, e: &Extraction) -> bool {
    // cout << "Checking region linearity: " << rootid << endl;
    // Find statewalk and subregions
    let mut statewalk: Vec<ExtractionENodeId> = Vec::new();
    let mut subregions: Vec<ExtractionENodeId> = Vec::new();
    let mut vis: Vec<bool> = vec![false; e.len()];
    let mut onpath: Vec<bool> = vec![false; e.len()];
    let mut q: VecDeque<ExtractionENodeId> = VecDeque::new();
    statewalk.push(rootid);
    onpath[rootid as usize] = true;
    let mut i: usize = 0;
    while i < statewalk.len() {
        let u: i32 = statewalk[i];
        let mut nxt: i32 = -1;
        for j in 0..(e[u as usize].ch.len() as i32) {
            let che: i32 = e[u as usize].ch[j as usize];
            if g.eclasses[e[che as usize].c as usize].isEffectful {
                if nxt == -1 {
                    nxt = che;
                    statewalk.push(nxt);
                    onpath[nxt as usize] = true;
                } else {
                    subregions.push(che);
                }
            } else {
                if !vis[che as usize] {
                    vis[che as usize] = true;
                    q.push_back(che);
                }
            }
        }
        i += 1;
    }
    // Check pure enodes only depend on the effectful walk in this region
    while !q.is_empty() {
        let u: i32 = *q.front().unwrap();
        q.pop_front();
        for i in 0..(e[u as usize].ch.len() as i32) {
            let v: i32 = e[u as usize].ch[i as usize];
            // assuming pure enodes can only have one effectful child
            if g.eclasses[e[v as usize].c as usize].isEffectful {
                if !onpath[v as usize] {
                    // using a effectul enode not in this region
                    return false;
                }
            } else {
                if !vis[v as usize] {
                    vis[v as usize] = true;
                    q.push_back(v);
                }
            }
        }
    }
    // Check all the subregions
    for i in 0..(subregions.len() as i32) {
        if !checkLinearRegionRec(g, subregions[i as usize], e) {
            return false;
        }
    }
    true
}

pub fn linearExtraction(g: &EGraph, root: EClassId, e: &Extraction) -> bool {
    if !validExtraction(g, root, e) {
        return false;
    }
    assert!(g.eclasses[root as usize].isEffectful);
    let rootid: ExtractionENodeId = (e.len() as ExtractionENodeId) - 1;
    assert!(e[rootid as usize].c == root);
    checkLinearRegionRec(g, rootid, e)
}

pub fn findArg(g: &EGraph) -> (EClassId, ENodeId) {
    let mut ret: (EClassId, ENodeId) = (-1, -1);
    let mut narg: i32 = 0;
    for i in 0..(g.eclasses.len() as i32) {
        if g.eclasses[i as usize].isEffectful {
            for j in 0..(g.eclasses[i as usize].enodes.len() as i32) {
                if g.eclasses[i as usize].enodes[j as usize].ch.is_empty() {
                    narg += 1;
                    if narg == 1 {
                        ret = (i, j);
                    }
                    break;
                }
            }
        }
    }
    if narg == 0 {
        eprintln!("Error: Failed to find arg!");
        crate::egraphin::print_egraph(g);
        assert!(false);
    } else if narg > 1 {
        eprintln!("Warning: Found mulitple arg in different eclasses!!");
    }
    ret
}

pub type RegionId = i32;

fn run_ilp_extractor(
    g: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
    timed_out: &mut bool,
    infeasible: &mut bool,
    ilp_encoding_num_vars: Option<&mut usize>,
    use_gurobi: bool,
) -> (Extraction, std::time::Duration) {
    let start = Instant::now();
    *timed_out = false;
    *infeasible = false;
    let extraction: Extraction = extractRegionILPInner(
        g,
        root,
        rstatewalk_cost,
        timed_out,
        infeasible,
        ilp_encoding_num_vars,
        use_gurobi,
    );
    let elapsed = start.elapsed();
    (extraction, elapsed)
}

pub fn extract_region_ilp_with_timing(
    g: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
    out: &mut Extraction,
    timed_out: &mut bool,
    infeasible: &mut bool,
    ilp_encoding_num_vars: &mut usize,
    use_gurobi: bool,
) -> i64 {
    let result = run_ilp_extractor(
        g,
        root,
        rstatewalk_cost,
        timed_out,
        infeasible,
        Some(ilp_encoding_num_vars),
        use_gurobi,
    );
    *out = result.0;
    result.1.as_nanos() as i64
}

// the main function for getting a linear extraction from a region
// this uses ILP in ilp mode or the unguided statewalk search in the normal mode
pub fn extractRegionILP(
    g: &EGraph,
    root: EClassId,
    rstatewalk_cost: &Vec<Vec<Cost>>,
    use_gurobi: bool,
) -> Extraction {
    let mut ilp_timed_out: bool = false;
    let mut ilp_infeasible: bool = false;
    let ilp_result = run_ilp_extractor(
        g,
        root,
        rstatewalk_cost,
        &mut ilp_timed_out,
        &mut ilp_infeasible,
        None,
        use_gurobi,
    );
    if ilp_timed_out {
        println!("TIMEOUT");
        std::process::exit(1);
    }
    if ilp_infeasible {
        eprintln!("ILP solver reported infeasibility");
        std::process::exit(1);
    }
    ilp_result.0
}

pub fn reconstructExtraction(
    g: &EGraph,
    region_roots: &Vec<EClassId>,
    region_root_id: &Vec<RegionId>,
    extracted_roots: &mut Vec<ExtractionENodeId>,
    e: &mut Extraction,
    cur_region: RegionId,
    statewalk_cost: &Vec<Vec<Cost>>,
    use_gurobi: bool,
) -> ExtractionENodeId {
    if extracted_roots[cur_region as usize] != -1 {
        return extracted_roots[cur_region as usize];
    }
    let region_root: EClassId = region_roots[cur_region as usize];
    //cout << cur_region << " Region root : " << region_root << endl;
    let res: (EGraph, SubEGraphMap) = createRegionEGraph(g, region_root);
    let gr: EGraph = res.0;
    let rmap: SubEGraphMap = res.1;
    let root: EClassId = get_inv_entry_or_fail(&rmap.inv, region_root, Some("getting region root mapping"));
    let mut region_to_global = EGraphMapping::new();
    region_to_global.eclassidmp = rmap.eclassmp.clone();
    region_to_global.enodeidmp = rmap.enode_map.clone();
    let rstatewalk_cost: Vec<Vec<Cost>> = project_statewalk_cost(&region_to_global, statewalk_cost);
    let er: Extraction = extractRegionILP(&gr, root, &rstatewalk_cost, use_gurobi);
    let mut ner: Extraction = vec![ExtractionENode::default(); er.len()];
    for i in 0..(er.len() as i32) {
        // C++: ExtractionENode &en = er[i], &nen = ner[i];
        let en_c = er[i as usize].c;
        let en_n = er[i as usize].n;
        let en_ch = er[i as usize].ch.clone();
        let oric: EClassId = rmap.eclassmp[en_c as usize];
        ner[i as usize].c = oric;
        assert!((en_c as usize) < rmap.enode_map.len());
        assert!((en_n as usize) < rmap.enode_map[en_c as usize].len());
        let orin: ENodeId = rmap.enode_map[en_c as usize][en_n as usize];
        ner[i as usize].n = orin;
        let mut subregionchild: bool = false;
        let mut k: usize = 0;
        let oric_enodes_orin_ch_len: usize = g.eclasses[oric as usize].enodes[orin as usize].ch.len();
        for j in 0..(oric_enodes_orin_ch_len as i32) {
            let orichc: EClassId = g.eclasses[oric as usize].enodes[orin as usize].ch[j as usize];
            if g.eclasses[orichc as usize].isEffectful {
                if subregionchild {
                    let recursed: ExtractionENodeId = reconstructExtraction(
                        g,
                        region_roots,
                        region_root_id,
                        extracted_roots,
                        e,
                        region_root_id[orichc as usize],
                        statewalk_cost,
                        use_gurobi,
                    );
                    ner[i as usize].ch.push(recursed);
                } else {
                    subregionchild = true;
                    ner[i as usize].ch.push(en_ch[k]);
                    k += 1;
                }
            } else {
                ner[i as usize].ch.push(en_ch[k]);
                k += 1;
            }
        }
    }
    let delta: i32 = e.len() as i32;
    for i in 0..(ner.len() as i32) {
        let mut subregionchild: bool = false;
        let ner_i_c = ner[i as usize].c;
        let ner_i_n = ner[i as usize].n;
        let ch_count: usize = g.eclasses[ner_i_c as usize].enodes[ner_i_n as usize].ch.len();
        for j in 0..(ch_count as i32) {
            let chc: EClassId =
                g.eclasses[ner_i_c as usize].enodes[ner_i_n as usize].ch[j as usize];
            if g.eclasses[chc as usize].isEffectful {
                if subregionchild {
                    continue;
                } else {
                    subregionchild = true;
                    ner[i as usize].ch[j as usize] += delta;
                }
            } else {
                ner[i as usize].ch[j as usize] += delta;
            }
        }
    }
    e.extend(ner.into_iter());
    extracted_roots[cur_region as usize] = (e.len() as ExtractionENodeId) - 1;
    extracted_roots[cur_region as usize]
}

pub fn extractAllILP(
    g: EGraph,
    fun_roots: Vec<EClassId>,
    use_gurobi: bool,
) -> Vec<Extraction> {
    let statewalk_cost: Vec<Vec<Cost>> = compute_statewalk_cost(&g);
    let mut ret: Vec<Extraction> = Vec::new();
    // C++ used `_` as the loop variable; rename to `_idx` in Rust.
    for _idx in 0..(fun_roots.len() as i32) {
        let fun_root: EClassId = fun_roots[_idx as usize];
        let mut region_root_id: Vec<RegionId> = vec![-1; g.eclasses.len()];
        let mut region_roots: Vec<EClassId> = Vec::new();
        region_roots.push(fun_root);
        region_root_id[fun_root as usize] = 0;
        for i in 0..(g.eclasses.len() as i32) {
            if g.eclasses[i as usize].isEffectful {
                for j in 0..(g.eclasses[i as usize].enodes.len() as i32) {
                    let mut subregionroot: bool = false;
                    for k in 0..(g.eclasses[i as usize].enodes[j as usize].ch.len() as i32) {
                        let v: EClassId =
                            g.eclasses[i as usize].enodes[j as usize].ch[k as usize];
                        if g.eclasses[v as usize].isEffectful {
                            if subregionroot {
                                if region_root_id[v as usize] == -1 {
                                    region_root_id[v as usize] = region_roots.len() as RegionId;
                                    region_roots.push(v);
                                }
                            } else {
                                subregionroot = true;
                            }
                        }
                    }
                }
            }
        }
        let mut extracted_roots: Vec<ExtractionENodeId> = vec![-1; region_roots.len()];
        let mut e: Extraction = Extraction::new();
        reconstructExtraction(
            &g,
            &region_roots,
            &region_root_id,
            &mut extracted_roots,
            &mut e,
            region_root_id[fun_root as usize],
            &statewalk_cost,
            use_gurobi,
        );
        assert!(linearExtraction(&g, fun_root, &e));
        ret.push(e);
    }
    //write_extract_region_timings();
    ret
}

// Suppress unused-import warning for IndexSet (kept to mirror potential C++ set use).
#[allow(dead_code)]
fn _unused_indexset(_: IndexSet<i32>) {}
