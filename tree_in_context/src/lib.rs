use from_egglog::FromEgglog;
use interpreter::Value;
use schema::{RcExpr, TreeProgram};

use crate::interpreter::interpret_tree_prog;

pub mod ast;
mod from_egglog;
pub mod interpreter;
mod optimizations;
pub mod schema;
pub mod schema_helpers;
mod to_egglog;
pub(crate) mod type_analysis;
pub mod typechecker;
pub(crate) mod utility;
use main_error::MainError;

pub type Result = std::result::Result<(), MainError>;

pub fn prologue() -> String {
    [
        include_str!("schema.egg"),
        include_str!("type_analysis.egg"),
        include_str!("utility/util.egg"),
        include_str!("utility/subst.egg"),
        &optimizations::is_valid::rules().join("\n"),
        &optimizations::body_contains::rules().join("\n"),
        &optimizations::purity_analysis::rules().join("\n"),
        &optimizations::conditional_invariant_code_motion::rules().join("\n"),
        include_str!("utility/in_context.egg"),
        include_str!("optimizations/constant_fold.egg"),
        include_str!("optimizations/switch_rewrites.egg"),
        &optimizations::loop_invariant::rules().join("\n"),
        include_str!("optimizations/loop_simplify.egg"),
    ]
    .join("\n")
}

pub fn build_program(program: &TreeProgram) -> String {
    format!(
        "{}\n(let PROG {})\n{}\n",
        prologue(),
        program.pretty(),
        include_str!("schedule.egg"),
    )
}

pub fn optimize(program: &TreeProgram) -> std::result::Result<TreeProgram, egglog::Error> {
    let program = build_program(program);
    let mut egraph = egglog::EGraph::default();
    egraph.parse_and_run_program(&program)?;
    let (sort, value) = egraph.eval_expr(&egglog::ast::Expr::Var((), "PROG".into()))?;
    let mut termdag = egglog::TermDag::default();
    let extracted = egraph.extract(value, &mut termdag, &sort);
    let from_egglog = FromEgglog { termdag };
    Ok(from_egglog.program_from_egglog(extracted.1))
}

fn check_func_type(func: RcExpr) {
    let prologue = [
        include_str!("schema.egg"),
        include_str!("type_analysis.egg"),
    ]
    .join("\n");
    let schedule = "(run-schedule
      (saturate
        (saturate type-helpers)
        type-analysis))";

    let body = func.func_body().expect("couldn't parse body");
    let out_ty = func.func_output_ty().expect("couldn't parse output type");
    let check = format!("(check (HasType BODY {out_ty}))");
    let s = format!("{prologue}\n(let BODY {body})\n{schedule}\n{check}",);

    let res = egglog::EGraph::default()
        .parse_and_run_program(&s)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        });
    assert!(
        res.is_ok(),
        "Failed to type {} with expected type {}",
        body,
        out_ty
    );
}

fn check_program_gets_type(program: TreeProgram) {
    check_func_type(program.entry);
    for func in program.functions {
        check_func_type(func);
    }
}

/// Runs an egglog test.
/// `build` is egglog code that runs before the running rules.
/// `check` is egglog code that runs after the running rules.
/// It is highly reccomended to also provide the programs used in the egglog code
/// so that they can be interpreted on the given value.
pub fn egglog_test(
    build: &str,
    check: &str,
    progs: Vec<TreeProgram>,
    input: Value,
    expected: Value,
    expected_log: Vec<String>,
) -> Result {
    // first interpret the programs on the value
    for prog in progs {
        let (result_val, print_log) = interpret_tree_prog(&prog, input.clone());
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
        check_program_gets_type(prog.clone());
    }

    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        prologue(),
        include_str!("schedule.egg"),
    );

    eprintln!("{}", program);

    let res = egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        });

    if res.is_err() {
        println!("{}", program);
        println!("{:?}", res);
    }

    Ok(res?)
}
