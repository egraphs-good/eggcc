use clap::ValueEnum;
use egglog::{ast::Symbol, Term, TermDag};
use egraph_serialize::Cost;
use greedy_dag_extractor::{
    extract_ilp, greedy_dag_extract, has_debug_exprs, serialized_egraph, DefaultCostModel,
};
use indexmap::IndexMap;
use indexmap::IndexSet;
use interpreter::Value;
use schedule::{rulesets, CompilerPass};
use schema::{Expr, RcExpr, TreeProgram};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use to_egglog::TreeToEgglog;

use crate::{
    dag2svg::tree_to_svg, from_egglog::FromEgglog, interpreter::interpret_dag_prog,
    optimizations::function_inlining, schedule::parallel_schedule, util::run_cmd_line,
};

pub mod add_context;
pub mod ast;
mod config;
pub mod dag2svg;
pub mod dag_typechecker;
pub mod from_egglog;
mod greedy_dag_extractor;
pub mod interpreter;
pub(crate) mod interval_analysis;
mod linearity;
mod optimizations;
mod remove_dead_code_nodes;
pub mod schema;
pub mod schema_helpers;
mod to_egglog;
pub(crate) mod type_analysis;
pub mod typechecker;
pub mod util;
pub(crate) mod utility;
use main_error::MainError;
pub mod extractiongymfastergreedydag;
pub mod fastercbcextractor;
pub mod pretty_print;
pub mod schedule;

pub type Result = std::result::Result<(), MainError>;

pub fn prologue() -> String {
    [
        include_str!("schema.egg"),
        include_str!("type_analysis.egg"),
        include_str!("utility/util.egg"),
        include_str!("utility/terms.egg"),
        &optimizations::is_valid::rules().join("\n"),
        &optimizations::is_resolved::rules().join("\n"),
        &optimizations::body_contains::rules().join("\n"),
        include_str!("optimizations/purity_analysis.egg"),
        // TODO cond inv code motion with regions
        //&optimizations::conditional_invariant_code_motion::rules().join("\n"),
        include_str!("utility/add_context.egg"),
        include_str!("utility/context-prop.egg"),
        include_str!("utility/term-subst.egg"),
        include_str!("utility/subst.egg"),
        include_str!("utility/context_of.egg"),
        include_str!("utility/canonicalize.egg"),
        include_str!("utility/expr_size.egg"),
        include_str!("utility/drop_at.egg"),
        include_str!("interval_analysis.egg"),
        include_str!("optimizations/switch_rewrites.egg"),
        include_str!("optimizations/select.egg"),
        include_str!("optimizations/peepholes.egg"),
        &optimizations::memory::rules(),
        include_str!("optimizations/memory.egg"),
        include_str!("optimizations/mem_simple.egg"),
        &optimizations::loop_invariant::rules().join("\n"),
        include_str!("optimizations/loop_simplify.egg"),
        include_str!("optimizations/loop_unroll.egg"),
        include_str!("optimizations/swap_if.egg"),
        include_str!("optimizations/rec_to_loop.egg"),
        include_str!("optimizations/passthrough.egg"),
        include_str!("optimizations/loop_strength_reduction.egg"),
        include_str!("optimizations/ivt.egg"),
        include_str!("optimizations/conditional_invariant_code_motion.egg"),
        include_str!("optimizations/conditional_push_in.egg"),
        include_str!("utility/debug-helper.egg"),
        include_str!("optimizations/hackers_delight.egg"),
        include_str!("optimizations/non_weakly_linear.egg"),
        &rulesets(),
    ]
    .join("\n")
}

fn ablate_prologue(prologue: &str, ablate: &str) -> String {
    let mut found_ruleset = false;
    let lines: Vec<String> = prologue
        .lines()
        .map(|line| {
            if line.contains(&format!("(ruleset {})", ablate)) {
                found_ruleset = true;
                line.replace(&format!("(ruleset {})", ablate), "")
            } else if line.contains(&format!(":ruleset {}", ablate)) {
                line.replace(&format!(":ruleset {}", ablate), ":ruleset never")
            } else {
                line.to_string()
            }
        })
        .collect();

    assert!(
        found_ruleset,
        "No ruleset {} found in prologue to ablate",
        ablate
    );
    lines.join("\n")
}

fn ablate_schedule(schedule: &str, ablate: &str) -> String {
    let mut found_schedule = false;
    let lines: Vec<String> = schedule
        .lines()
        .map(|line| {
            if line.contains(ablate) {
                found_schedule = true;
                line.replace(ablate, "never")
            } else {
                line.to_string()
            }
        })
        .collect();

    assert!(found_schedule, "No schedule {} found to ablate", ablate);
    lines.join("\n")
}

/// Adds an egglog program to `res` that adds the given term
/// to the database.
/// Returns a fresh variable referring to the program.
/// Note that because the cache caches based on a term, which
/// references the termdag, this cache **cannot** be reused
/// across different TermDags. Make sure to update the term dag
/// for a new term (using TreeToEgglog), rather than creating a
/// new term dag.
fn print_with_intermediate_helper(
    termdag: &TermDag,
    term: Term,
    cache: &mut IndexMap<Term, String>,
    res: &mut String,
) -> String {
    if let Some(var) = cache.get(&term) {
        return var.clone();
    }

    match &term {
        Term::Lit(_) => termdag.to_string(&term),
        Term::Var(_) => termdag.to_string(&term),
        Term::App(head, children) => {
            let child_vars = children
                .iter()
                .map(|child| {
                    print_with_intermediate_helper(termdag, termdag.get(*child).clone(), cache, res)
                })
                .collect::<Vec<String>>()
                .join(" ");
            let fresh_var = format!("__tmp{}", cache.len());
            writeln!(res, "(let {fresh_var} ({head} {child_vars}))").unwrap();
            cache.insert(term, fresh_var.clone());

            fresh_var
        }
    }
}

pub fn print_with_intermediate_vars(termdag: &TermDag, term: Term) -> String {
    let mut printed = String::new();
    let mut cache = IndexMap::<Term, String>::new();
    let res = print_with_intermediate_helper(termdag, term, &mut cache, &mut printed);
    printed.push_str(&format!("(let PROG {res})\n"));
    printed
}

// Build an egglog program that optimizes a particular batch of functions `fns`
// with a schedule `schedule`.
// Adds context to the program before optimizing.
// If `inline_program` is true, it also inlines calls in `fns`.
// `inline_program` is the program to inline calls from, allowing us to inline unoptimized function bodies.
pub fn build_program(
    program: &TreeProgram,
    inline_program: Option<&TreeProgram>,
    fns: &[String],
    schedule: &str,
    ablate: Option<&str>,
) -> String {
    let (program, mut context_cache) = program.add_context();
    let mut printed = String::new();

    // Create a global cache for generating intermediate variables
    let mut tree_state = TreeToEgglog::new();
    let mut term_cache = IndexMap::<Term, String>::new();

    // Generate function inlining egglog
    let function_inlining_unions = if let Some(inline_program) = inline_program {
        let mut pairs = vec![];
        for func in fns {
            pairs.extend(function_inlining::function_inlining_pairs(
                inline_program,
                vec![func.clone()],
                config::FUNCTION_INLINING_ITERATIONS,
                &mut context_cache,
            ));
        }

        function_inlining::print_function_inlining_pairs(
            pairs,
            &mut printed,
            &mut tree_state,
            &mut term_cache,
        )
    } else {
        "".to_string()
    };

    // Generate program egglog
    for func in fns {
        let func = program.get_function(func).unwrap();
        let term = func.to_egglog_with(&mut tree_state);
        let _func_var = print_with_intermediate_helper(
            &tree_state.termdag,
            term,
            &mut term_cache,
            &mut printed,
        );
    }

    let loop_context_unions =
        context_cache.get_unions_with_sharing(&mut printed, &mut tree_state, &mut term_cache);

    // set the type of each function
    for func in program.fns() {
        let func = program.get_function(&func).unwrap();
        let func_name = func.func_name().unwrap();
        let input_ty = func.func_input_ty().unwrap();
        let func_ty = func.func_output_ty().unwrap();
        writeln!(
            &mut printed,
            "(FunctionHasType \"{func_name}\" {input_ty} {func_ty})",
        )
        .unwrap();
    }

    let prologue = prologue();
    let (prologue, schedule) = if let Some(ablate) = ablate {
        (
            ablate_prologue(&prologue, ablate),
            ablate_schedule(schedule, ablate),
        )
    } else {
        (prologue, schedule.to_string())
    };

    format!(
        "
; Prologue
{prologue}

; required by function_inlining_unoins
; Function inlining unions
(relation InlinedCall (String Expr))

(ruleset initialization)
(rule () (
    ; Program nodes
    {printed}

    ; Loop context unions
    {loop_context_unions}

    ; Function inlining unions
    {function_inlining_unions}
) :ruleset initialization)
(run initialization 1) 

; Schedule
{schedule}
"
    )
}

pub fn are_progs_eq(program1: TreeProgram, program2: TreeProgram) -> bool {
    let mut converter = TreeToEgglog::new();
    let term1 = program1.to_egglog_with(&mut converter);
    let term2 = program2.to_egglog_with(&mut converter);
    term1 == term2
}

/// Adds the program to the egraph and extracts it.
/// Checks that the extracted program is the same as the input program.
pub fn check_roundtrip_egraph(program: &TreeProgram) {
    let mut termdag = egglog::TermDag::default();
    let fns = program.fns();
    let egglog_prog = build_program(program, None, &fns, "", None);
    log::info!("Running egglog program...");
    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(None, &egglog_prog).unwrap();

    let (serialized, unextractables) = serialized_egraph(egraph);
    let (_res_cost, res) = greedy_dag_extract(
        program,
        program.fns(),
        serialized,
        unextractables,
        &mut termdag,
        DefaultCostModel,
        true,
        false,
    );

    let (original_with_ctx, _) = program.add_dummy_ctx();
    let (res_with_ctx, _) = res.add_dummy_ctx();

    if !are_progs_eq(original_with_ctx.clone(), res_with_ctx.clone()) {
        eprintln!("Original program: {}", tree_to_svg(&original_with_ctx));
        eprintln!("Result program: {}", tree_to_svg(&res_with_ctx));
        panic!("Check failed. Programs should be equal before and after roundtrip to egraph.");
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug, ValueEnum)]
pub enum Schedule {
    #[default]
    Parallel,
    Sequential,
}

#[derive(Clone, Debug)]
pub struct EggccConfig {
    pub schedule: Schedule,
    /// Stop after this many passes.
    /// If stop_after_n_passes is negative,
    /// run [0 ... schedule.len() + stop_after_n_passes] passes.
    pub stop_after_n_passes: i64,
    /// For debugging, disable extraction with linearity
    /// and just return the first program found.
    /// This produces unsound results but is useful for seeing the intermediate extracted result.
    pub linearity: bool,
    /// When Some, optimize only the functions in this set.
    pub optimize_functions: Option<HashSet<String>>,
    pub ablate: Option<String>,
    pub ilp_extraction_test_timeout: Option<Duration>,
    pub non_weakly_linear: bool,
    /// If true, use the experimental tiger extractor format output instead of greedy extractor.
    pub use_tiger: bool,
    /// If use_tiger is true and tiger_ilp is true, use ILP extraction in tiger instead of greedy extraction.
    pub tiger_ilp: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExtractionTimeSample {
    pub egraph_size: usize,
    pub ilp_time: Option<Duration>,
    pub eggcc_time: Duration,
}

pub struct EggccTimeStatistics {
    pub eggcc_extraction_time: Duration,
    pub eggcc_serialization_time: Duration,
    // if ilp didn't time out, what portion of the time was spent in the extraction gym code
    pub ilp_test_times: Vec<ExtractionTimeSample>,
}

impl Default for EggccTimeStatistics {
    fn default() -> Self {
        Self {
            eggcc_extraction_time: Duration::from_millis(0),
            eggcc_serialization_time: Duration::from_millis(0),
            ilp_test_times: vec![],
        }
    }
}

impl EggccConfig {
    pub fn get_schedule_list(&self) -> Vec<CompilerPass> {
        match self.schedule {
            Schedule::Parallel => parallel_schedule(self),
            Schedule::Sequential => schedule::mk_sequential_schedule(),
        }
    }

    pub fn get_normalized_cutoff(&self, schedule_len: usize) -> usize {
        if self.stop_after_n_passes < 0 {
            (schedule_len as i64 + self.stop_after_n_passes) as usize
        } else if self.stop_after_n_passes > schedule_len as i64 {
            schedule_len
        } else {
            self.stop_after_n_passes as usize
        }
    }
}

impl Default for EggccConfig {
    fn default() -> Self {
        Self {
            schedule: Schedule::default(),
            stop_after_n_passes: i64::MAX,
            linearity: true,
            optimize_functions: None,
            ablate: None,
            ilp_extraction_test_timeout: None,
            non_weakly_linear: false,
            use_tiger: false,
            tiger_ilp: false,
        }
    }
}

fn find_tiger_binary(binary: &str) -> Option<PathBuf> {
    let binary_name = if cfg!(windows) {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut candidate_dirs = Vec::new();
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            candidate_dirs.push(dir.to_path_buf());
        }
    }

    let target_root = manifest_dir
        .parent()
        .map(|workspace_root| workspace_root.join("target"))
        .unwrap_or_else(|| manifest_dir.join("target"));

    candidate_dirs.push(target_root.join("release"));
    candidate_dirs.push(target_root.join("debug"));

    let mut seen = HashSet::new();
    for dir in candidate_dirs {
        if !seen.insert(dir.clone()) {
            continue;
        }
        let candidate = dir.join(&binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

// This function is a helper for extracting using egglog's built-in extraction, which doesn't consider linearity.
// We currently only use it for extracting from the egraph tiger produces, which doesn't do any unions (it just encodes a single program).
fn extract_program_with_egglog(
    original_prog: &TreeProgram,
    batch: &[String],
    egraph: &mut egglog::EGraph,
) -> TreeProgram {
    let function_symbol: Symbol = "Function".into();
    let (rows, termdag) = match egraph.function_to_dag(function_symbol, usize::MAX) {
        Ok(res) => res,
        Err(err) => {
            panic!("Failed to convert egglog egraph to term dag: {err}");
        }
    };

    let target_names: IndexSet<String> = batch.iter().cloned().collect();

    let mut converter = FromEgglog {
        termdag: &termdag,
        conversion_cache: IndexMap::new(),
    };

    let mut extracted: IndexMap<String, RcExpr> = IndexMap::new();

    for (_func_term, value_term) in rows {
        // For constructors, the extracted term is in the output position.
        let expr = converter.expr_from_egglog(value_term);
        match expr.as_ref() {
            Expr::Function(func_name, _, _, _) => {
                if !target_names.contains(func_name) {
                    continue;
                }
                if extracted.insert(func_name.clone(), expr.clone()).is_some() {
                    panic!(
                        "Duplicate function {func_name} encountered during extraction; overwriting previous result"
                    );
                }
            }
            other => {
                panic!("Expected extracted expression to be a function, got {other:?}");
            }
        }
    }
    let mut res = original_prog.clone();

    for name in &target_names {
        res.replace_fn(name, extracted.get(name).unwrap().clone());
    }

    res
}

// Run tiger extractor pipeline using the tiger binaries built from c++.
// See the build.rs file.
fn run_tiger_pipeline(
    eggcc_config: &EggccConfig,
    original_prog: &TreeProgram,
    batch: &[String],
    egraph: &egraph_serialize::EGraph,
    _should_maintain_linearity: bool,
) -> TreeProgram {
    let json = serde_json::to_string_pretty(egraph)
        .map_err(|err| format!("failed to serialize egraph: {err}"))
        .unwrap();
    let json_input = format!("{json}\n");

    let json2egraph_bin = find_tiger_binary("json2egraph")
        .ok_or_else(|| "json2egraph binary not found; build the tiger tools first".to_string())
        .unwrap();

    let egraph_text = run_cmd_line(
        json2egraph_bin.as_os_str(),
        std::iter::empty::<&std::ffi::OsStr>(),
        &json_input,
    )
    .map_err(|err| format!("json2egraph invocation failed: {err}"))
    .unwrap();

    let tiger_bin = find_tiger_binary("tiger")
        .ok_or_else(|| "tiger binary not found; build the tiger tools first".to_string())
        .unwrap();

    let tiger_args: Vec<&std::ffi::OsStr> = if eggcc_config.tiger_ilp {
        vec![std::ffi::OsStr::new("--ilp-mode")]
    } else {
        Vec::new()
    };

    let tiger_output = run_cmd_line(tiger_bin.as_os_str(), tiger_args, &egraph_text)
        .map_err(|err| format!("tiger invocation failed: {err}"))
        .unwrap();

    // Tiger returns an egglog file containing just one program, run the egglog program
    let mut tiger_egraph = egglog::EGraph::default();
    tiger_egraph
        .parse_and_run_program(None, &tiger_output)
        .map_err(|err| format!("failed to run tiger egglog program: {err}"))
        .unwrap();

    extract_program_with_egglog(original_prog, batch, &mut tiger_egraph).override_arg_types()
}

#[allow(clippy::too_many_arguments)]
fn extract(
    eggcc_config: &EggccConfig,
    original_prog: &TreeProgram,
    batch: Vec<String>,
    egraph: &egraph_serialize::EGraph,
    unextractables: &IndexSet<String>,
    termdag: &mut TermDag,
    should_maintain_linearity: bool,
    extract_debug_exprs: bool,
) -> TreeProgram {
    if eggcc_config.use_tiger {
        run_tiger_pipeline(
            eggcc_config,
            original_prog,
            &batch,
            egraph,
            should_maintain_linearity,
        )
    } else {
        greedy_dag_extract(
            original_prog,
            batch.clone(),
            egraph.clone(),
            unextractables.clone(),
            termdag,
            DefaultCostModel,
            should_maintain_linearity,
            extract_debug_exprs,
        )
        .1
    }
}

// Optimizes a tree program using the given schedule.
// Adds context to the program before optimizing.
// If successful, returns the optimized program and the time
// it takes for serialization and extraction
pub fn optimize(
    program: &TreeProgram,
    eggcc_config: &EggccConfig,
) -> std::result::Result<(TreeProgram, EggccTimeStatistics), egglog::Error> {
    let mut eggcc_serialization_time = Duration::from_millis(0);
    let mut eggcc_extraction_time = Duration::from_millis(0);
    let schedule_list = eggcc_config.get_schedule_list();
    let mut res = program.clone();
    let mut ilp_test_times = vec![];

    let cutoff = eggcc_config.get_normalized_cutoff(schedule_list.len());
    for (i, schedule) in schedule_list[..cutoff].iter().enumerate() {
        let mut should_maintain_linearity = true;
        if i == cutoff - 1 {
            should_maintain_linearity = eggcc_config.linearity;
        }

        log::info!("Running pass {}...", i);
        let fns = res.fns();

        // if we are inlining, save the program
        // TODO we inline on the first pass, but this should be configurable from the schedule
        let inline_program = match schedule {
            schedule::CompilerPass::Schedule(_) => None,
            schedule::CompilerPass::InlineWithSchedule(_) => Some(res.clone()),
        };

        // TODO experiment with different batches of optimizing functions together
        // currently we use the whole program
        let batches = match &eggcc_config.optimize_functions {
            Some(allowed_fns) => {
                // check that all allowed_fns are in fns
                for allowed_fn in allowed_fns {
                    if !fns.contains(allowed_fn) {
                        panic!(
                            "Told to optimize function {}, but not found in program",
                            allowed_fn
                        );
                    }
                }

                vec![allowed_fns.iter().cloned().collect()]
            }
            None => vec![fns.clone()],
        };

        for batch in batches {
            log::info!("Running pass {} on batch {:?}", i, batch);
            log::info!("Schedule: {:?}", schedule);
            // only inline functions on the first pass
            let egglog_prog = build_program(
                &res,
                inline_program.as_ref(),
                &batch,
                schedule.egglog_schedule(),
                eggcc_config.ablate.as_deref(),
            );

            log::info!("Running egglog program...");
            let mut egraph = egglog::EGraph::default();
            egraph.parse_and_run_program(None, &egglog_prog)?;

            let serialization_start = Instant::now();
            let (serialized, unextractables) = serialized_egraph(egraph);

            let extraction_start = Instant::now();
            let mut termdag = egglog::TermDag::default();
            let has_debug_exprs = has_debug_exprs(&serialized);
            if has_debug_exprs {
                log::info!(
                    "Program has debug expressions, extracting them instead of original program."
                );
            }
            let iter_result = extract(
                eggcc_config,
                &res,
                batch.clone(),
                &serialized,
                &unextractables,
                &mut termdag,
                should_maintain_linearity,
                has_debug_exprs,
            );

            let extraction_end = Instant::now();

            eggcc_extraction_time += extraction_end - extraction_start;
            eggcc_serialization_time += extraction_start - serialization_start;

            // now extract with ILP if we were told to
            if let Some(timeout) = eggcc_config.ilp_extraction_test_timeout {
                let times = extract_ilp(
                    &res,
                    batch,
                    serialized,
                    unextractables,
                    DefaultCostModel,
                    timeout,
                );

                ilp_test_times.extend(times);
            }

            // typecheck the program as a sanity check
            iter_result.typecheck();

            res = iter_result;

            if has_debug_exprs {
                log::info!("Program has debug expressions, stopping pass {}.", i);
                return Ok((
                    res,
                    EggccTimeStatistics {
                        eggcc_extraction_time,
                        eggcc_serialization_time,
                        ilp_test_times,
                    },
                ));
            }
        }

        // now add context to res again for the next pass, since context might be less specific
        res = res.add_context().0;
    }
    Ok((
        res,
        EggccTimeStatistics {
            eggcc_extraction_time,
            eggcc_serialization_time,
            ilp_test_times,
        },
    ))
}

fn check_program_gets_type(program: TreeProgram) -> Result {
    let prologue = [
        include_str!("schema.egg"),
        include_str!("type_analysis.egg"),
    ]
    .join("\n");

    // We need to include the whole program, since function bodies could
    // include calls to other functions.
    let (term, termdag) = program.to_egglog();
    let printed_program = print_with_intermediate_vars(&termdag, term);

    let schedule = "(run-schedule
      (saturate
        (saturate type-helpers)
        type-analysis))";

    // For each function body, we check its type
    let mut bodies = Vec::new();
    let mut checks = Vec::new();

    let mut funcs = program.functions.clone();
    funcs.push(program.entry.clone());
    for func in funcs {
        let body = func.func_body().expect("couldn't parse body");
        let name = func.func_name().expect("couldn't parse function name");
        bodies.push(format!("(let BODY_{name} {body})"));
        let out_ty = func.func_output_ty().expect("couldn't parse output type");
        checks.push(format!("(check (HasType BODY_{name} {out_ty}))"));
    }

    let s = format!(
        "{prologue}\n{printed_program}\n{}\n{schedule}\n{}",
        bodies.join("\n"),
        checks.join("\n")
    );

    egglog::EGraph::default()
        .parse_and_run_program(None, &s)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        })?;
    Ok(())
}

/// Run the egglog test and print the program for debugging.
/// Also add the debug_helper.egg file to the end
pub fn egglog_test_and_print_program(
    build: &str,
    check: &str,
    progs: Vec<TreeProgram>,
    input: Value,
    expected: Value,
    expected_log: Vec<String>,
) -> Result {
    egglog_test_internal(build, check, progs, input, expected, expected_log, true)
}

pub fn egglog_test(
    build: &str,
    check: &str,
    progs: Vec<TreeProgram>,
    input: Value,
    expected: Value,
    expected_log: Vec<String>,
) -> Result {
    egglog_test_internal(build, check, progs, input, expected, expected_log, false)
}

/// Runs an egglog test.
/// `build` is egglog code that runs before the running rules.
/// `check` is egglog code that runs after the running rules.
/// It is highly reccomended to also provide the programs used in the egglog code
/// so that they can be interpreted on the given value.
fn egglog_test_internal(
    build: &str,
    check: &str,
    progs: Vec<TreeProgram>,
    input: Value,
    expected: Value,
    expected_log: Vec<String>,
    print_program: bool,
) -> Result {
    // first interpret the programs on the value
    for prog in progs {
        let (result_val, print_log) = interpret_dag_prog(&prog, &input);
        assert_eq!(
            result_val, expected,
            "Program {:?}\nproduced:\n{}\ninstead of expected:\n{}",
            prog, result_val, expected
        );
        assert_eq!(
            print_log, expected_log,
            "Program {:?}\nproduced log:\n{:?}\ninstead of expected log:\n{:?}",
            prog, print_log, expected_log
        );

        // Check that the input program gets a type by the type analysis
        match check_program_gets_type(prog.clone()) {
            Ok(_) => (),
            Err(e) => {
                println!("Error in type analysis for program {:?}: {:?}", prog, e);
                return Err(e);
            }
        }
    }

    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        prologue(),
        parallel_schedule(&EggccConfig::default())
            .iter()
            .map(|pass| pass.egglog_schedule().to_string())
            .collect::<Vec<String>>()
            .join("\n"),
    );

    if print_program {
        eprintln!("{program}");
    }

    let res = egglog::EGraph::default()
        .parse_and_run_program(None, &program)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        });

    if res.is_err() {
        eprintln!("{:?}", res);
    }

    Ok(res?)
}
