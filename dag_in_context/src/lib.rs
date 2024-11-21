use clap::ValueEnum;
use egglog::{Term, TermDag};
use greedy_dag_extractor::{extract, serialized_egraph, DefaultCostModel};
use indexmap::IndexMap;
use interpreter::Value;
use schedule::rulesets;
use schema::TreeProgram;
use std::{cmp::min, fmt::Write, usize};
use to_egglog::TreeToEgglog;

use crate::{
    add_context::ContextCache, dag2svg::tree_to_svg, interpreter::interpret_dag_prog,
    optimizations::function_inlining, schedule::parallel_schedule,
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
pub mod schema;
pub mod schema_helpers;
mod to_egglog;
pub(crate) mod type_analysis;
pub mod typechecker;
pub(crate) mod utility;
use main_error::MainError;
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
        &optimizations::loop_invariant::rules().join("\n"),
        include_str!("optimizations/loop_simplify.egg"),
        include_str!("optimizations/loop_unroll.egg"),
        include_str!("optimizations/swap_if.egg"),
        include_str!("optimizations/rec_to_loop.egg"),
        include_str!("optimizations/passthrough.egg"),
        include_str!("optimizations/loop_strength_reduction.egg"),
        include_str!("optimizations/ivt.egg"),
        include_str!("utility/debug-helper.egg"),
        &rulesets(),
    ]
    .join("\n")
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
                    print_with_intermediate_helper(termdag, termdag.get(*child), cache, res)
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
// If `inline_program` is true, it also inlines calls in `fns`.
// `inline_program` is the program to inline calls from, allowing us to inline unoptimized function bodies.
pub fn build_program(
    program: &TreeProgram,
    inline_program: Option<&TreeProgram>,
    fns: &[String],
    cache: &mut ContextCache,
    schedule: &str,
) -> String {
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
                cache,
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
        cache.get_unions_with_sharing(&mut printed, &mut tree_state, &mut term_cache);

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

    format!(
        "
; Prologue
{prologue}

; Program nodes
{printed}

; Loop context unions
{loop_context_unions}

; Function inlining unions
{function_inlining_unions}

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
    let egglog_prog = build_program(program, None, &fns, &mut ContextCache::new(), "");
    log::info!("Running egglog program...");
    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(None, &egglog_prog).unwrap();

    let (serialized, unextractables) = serialized_egraph(egraph);
    let (_res_cost, res) = extract(
        program,
        program.fns(),
        serialized,
        unextractables,
        &mut termdag,
        DefaultCostModel,
        true,
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
    pub stop_after_n_passes: usize,
    /// For debugging, disable extraction with linearity
    /// and just return the first program found.
    /// This produces unsound results but is useful for seeing the intermediate extracted result.
    pub linearity: bool,
}

impl Default for EggccConfig {
    fn default() -> Self {
        Self {
            schedule: Schedule::default(),
            stop_after_n_passes: usize::MAX,
            linearity: true,
        }
    }
}

// It is expected that program has context added
pub fn optimize(
    program: &TreeProgram,
    cache: &mut ContextCache,
    eggcc_config: &EggccConfig,
) -> std::result::Result<TreeProgram, egglog::Error> {
    let schedule_list = match eggcc_config.schedule {
        Schedule::Parallel => parallel_schedule(),
        Schedule::Sequential => schedule::mk_sequential_schedule(),
    };
    let mut res = program.clone();

    for (schedule, i) in schedule_list
        .iter()
        .zip(0..eggcc_config.stop_after_n_passes)
    {
        let mut should_maintain_linearity = true;
        if i == min(
            eggcc_config.stop_after_n_passes - 1,
            schedule_list.len() - 1,
        ) {
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
        let batches = vec![fns.clone()];

        for batch in batches {
            log::info!("Running pass {} on batch {:?}", i, batch);
            log::info!("Schedule: {:?}", schedule);
            // only inline functions on the first pass
            let egglog_prog = build_program(
                &res,
                inline_program.as_ref(),
                &batch,
                cache,
                schedule.egglog_schedule(),
            );

            log::info!("Running egglog program...");
            let mut egraph = egglog::EGraph::default();
            egraph.parse_and_run_program(None, &egglog_prog)?;

            let (serialized, unextractables) = serialized_egraph(egraph);
            let mut termdag = egglog::TermDag::default();
            let (_res_cost, iter_result) = extract(
                &res,
                batch,
                serialized,
                unextractables,
                &mut termdag,
                DefaultCostModel,
                should_maintain_linearity,
            );
            res = iter_result;
        }

        // now add context to res again for the next pass, since context might be less specific
        res = res.add_context().0;
    }
    Ok(res)
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
        parallel_schedule()
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
