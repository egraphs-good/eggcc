use std::{collections::HashMap, convert, fmt::Error, rc::Rc};

use egglog::{ast::Expr, Term, TermDag};
use from_egglog::{program_from_egglog, FromEgglog};
use greedy_dag_extractor::{extract, serialized_egraph, DefaultCostModel};
use interpreter::Value;
use schema::{RcExpr, TreeProgram};
use std::fmt::Write;
use to_egglog::TreeToEgglog;

use crate::{
    dag2svg::tree_to_svg, interpreter::interpret_dag_prog, optimizations::function_inlining,
    schedule::mk_schedule,
};

pub(crate) mod add_context;
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
pub mod schedule;

pub type Result = std::result::Result<(), MainError>;

pub fn prologue() -> String {
    [
        include_str!("schema.egg"),
        include_str!("type_analysis.egg"),
        include_str!("utility/util.egg"),
        &optimizations::is_valid::rules().join("\n"),
        &optimizations::is_resolved::rules().join("\n"),
        &optimizations::body_contains::rules().join("\n"),
        &optimizations::purity_analysis::rules().join("\n"),
        // TODO cond inv code motion with regions
        //&optimizations::conditional_invariant_code_motion::rules().join("\n"),
        include_str!("utility/add_context.egg"),
        include_str!("utility/context-prop.egg"),
        include_str!("utility/subst.egg"),
        include_str!("utility/context_of.egg"),
        include_str!("utility/canonicalize.egg"),
        include_str!("utility/expr_size.egg"),
        include_str!("utility/drop_at.egg"),
        include_str!("interval_analysis.egg"),
        include_str!("optimizations/switch_rewrites.egg"),
        include_str!("optimizations/peepholes.egg"),
        &optimizations::memory::rules(),
        include_str!("optimizations/memory.egg"),
        &optimizations::loop_invariant::rules().join("\n"),
        include_str!("optimizations/loop_simplify.egg"),
        include_str!("optimizations/loop_unroll.egg"),
        include_str!("optimizations/passthrough.egg"),
        include_str!("optimizations/loop_strength_reduction.egg"),
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
    cache: &mut HashMap<Term, String>,
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

pub(crate) fn print_with_intermediate_vars(termdag: &TermDag, term: Term) -> String {
    let mut printed = String::new();
    let mut cache = HashMap::<Term, String>::new();
    let res = print_with_intermediate_helper(termdag, term, &mut cache, &mut printed);
    printed.push_str(&format!("(let PROG {res})\n"));
    printed
}

// It is expected that program has context added
pub fn build_program(program: &TreeProgram, optimize: bool) -> String {
    let mut printed = String::new();

    // Create a global cache for generating intermediate variables
    let mut tree_state = TreeToEgglog::new();
    let mut term_cache = HashMap::<Term, String>::new();

    // Generate function inlining egglog
    let function_inlining = if !optimize {
        "".to_string()
    } else {
        function_inlining::print_function_inlining_pairs(
            function_inlining::function_inlining_pairs(
                program,
                config::FUNCTION_INLINING_ITERATIONS,
            ),
            &mut printed,
            &mut tree_state,
            &mut term_cache,
        )
    };

    // Generate program egglog
    let term = program.to_egglog_with(&mut tree_state);
    let res =
        print_with_intermediate_helper(&tree_state.termdag, term, &mut term_cache, &mut printed);

    format!(
        "{}\n{printed}\n(let PROG {res})\n{function_inlining}\n{}\n",
        prologue(),
        if optimize {
            mk_schedule()
        } else {
            "".to_string()
        }
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
    let egglog_prog = build_program(program, false);
    log::info!("Running egglog program...");
    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(&egglog_prog).unwrap();

    let (serialized, unextractables) = serialized_egraph(egraph);
    let (_res_cost, res) = extract(
        program,
        serialized,
        unextractables,
        &mut termdag,
        DefaultCostModel,
    );

    let original_with_ctx = program.add_dummy_ctx();
    let res_with_ctx = res.add_dummy_ctx();

    if !are_progs_eq(original_with_ctx.clone(), res_with_ctx.clone()) {
        eprintln!("Original program: {}", tree_to_svg(&original_with_ctx));
        eprintln!("Result program: {}", tree_to_svg(&res_with_ctx));
        panic!("Check failed. Programs should be equal before and after roundtrip to egraph.");
    }
}

// It is expected that program has context added
pub fn optimize(program: &TreeProgram) -> std::result::Result<TreeProgram, egglog::Error> {
    let egglog_prog = build_program(program, true);
    log::info!("Running egglog program...");
    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(&egglog_prog)?;

    let (serialized, unextractables) = serialized_egraph(egraph);
    let mut termdag = egglog::TermDag::default();
    let (_res_cost, res) = extract(
        program,
        serialized,
        unextractables,
        &mut termdag,
        DefaultCostModel,
    );
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
        .parse_and_run_program(&s)
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

    let program = format!("{}\n{build}\n{}\n{check}\n", prologue(), mk_schedule(),);

    if print_program {
        eprintln!("{}\n{}", program, include_str!("utility/debug-helper.egg"));
    }

    let res = egglog::EGraph::default()
        .parse_and_run_program(&program)
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

pub fn pretty_print(str_expr: String) -> std::result::Result<RcExpr, egglog::Error> {
    let bounded_expr = format!("(let EXPR___ {})", str_expr);
    let prog = prologue().to_owned() + &bounded_expr;
    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(&prog).unwrap();
    let mut termdag = TermDag::default();
    let (sort, value) = egraph
        .eval_expr(&egglog::ast::Expr::Var((), "EXPR___".into()))
        .unwrap();
    let (_, extracted) = egraph.extract(value, &mut termdag, &sort);
    let mut converter = FromEgglog {
        termdag: &termdag,
        conversion_cache: HashMap::new(),
    };
    let expr = converter.expr_from_egglog(extracted);
    Ok(expr)
}

fn pretty_print_helper(expr:RcExpr, cash: HashMap<RcExpr, String>) -> String {
    use schema::Expr;
    match expr.as_ref() {
        Expr::Function(name, inty, outty, body) => {""}
            Expr::Const(c, ty) => {""}
            // Expr::Top(_, x, y, z) => 
            // Expr::Bop(_, x, y) => 
            // Expr::Uop(_, x) => 
            // Expr::Get(x, _) => 
            // Expr::Alloc(_, x, y, _) => 
            // Expr::Call(_, x) =>
            // Expr::Empty(_) => 
            // Expr::Single(x) => 
            // Expr::Concat(x, y) => 
            // Expr::Switch(x, inputs, _branches) => {
                
            // }
            // Expr::If(x, inputs, _y, _z) => {
               
            // }
            // Expr::DoWhile(inputs, _body) =>
            // Expr::Arg(_) => 
            // Expr::InContext(_, x) => 
    }
}

#[test]
fn test_pretty_print() {
    use crate::ast::*;
    let output_ty = tuplet!(intt(), intt(), intt(), intt(), statet());
    let inner_inv = sub(getat(2), getat(1)).with_arg_types(output_ty.clone(), base(intt()));
    let inv = add(inner_inv.clone(), int(0)).with_arg_types(output_ty.clone(), base(intt()));
    let pred = less_than(getat(0), getat(3)).with_arg_types(output_ty.clone(), base(boolt()));
    let not_inv = add(getat(0), inv.clone()).with_arg_types(output_ty.clone(), base(intt()));
    let print = tprint(inv.clone(), getat(4)).with_arg_types(output_ty.clone(), base(statet()));

    let my_loop = dowhile(
        parallel!(int(1), int(2), int(3), int(4), getat(0)),
        concat(
            parallel!(pred.clone(), not_inv.clone(), getat(1)),
            concat(parallel!(getat(2), getat(3)), single(print.clone())),
        ),
    )
    .with_arg_types(tuplet!(statet()), output_ty.clone());

    let expr_str = my_loop.to_string();
    let expr = pretty_print(expr_str).unwrap();
    print!("{}", expr);
}
