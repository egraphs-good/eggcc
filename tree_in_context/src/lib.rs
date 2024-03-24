use std::collections::HashMap;

use egglog::{Term, TermDag};
use from_egglog::FromEgglog;
use interpreter::Value;
use schema::{RcExpr, TreeProgram};
use std::fmt::Write;

use crate::interpreter::interpret_tree_prog;

pub mod ast;
pub mod from_egglog;
pub mod interpreter;
pub(crate) mod interval_analysis;
mod optimizations;
pub mod schema;
pub mod schema_helpers;
mod to_egglog;
pub(crate) mod type_analysis;
pub mod typechecker;
pub(crate) mod utility;
use main_error::MainError;
pub(crate) mod add_context;

pub type Result = std::result::Result<(), MainError>;

pub fn prologue() -> String {
    [
        include_str!("schema.egg"),
        include_str!("type_analysis.egg"),
        include_str!("interval_analysis.egg"),
        include_str!("utility/util.egg"),
        &optimizations::is_valid::rules().join("\n"),
        &optimizations::body_contains::rules().join("\n"),
        &optimizations::purity_analysis::rules().join("\n"),
        &optimizations::conditional_invariant_code_motion::rules().join("\n"),
        include_str!("utility/in_context.egg"),
        include_str!("utility/subst.egg"),
        include_str!("optimizations/switch_rewrites.egg"),
        &optimizations::loop_invariant::rules().join("\n"),
        include_str!("optimizations/loop_simplify.egg"),
    ]
    .join("\n")
}

/// Adds an egglog program to `res` that adds the given term
/// to the database.
/// Returns a fresh variable referring to the program.
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
            write!(res, "(let {fresh_var} ({head} {child_vars}))").unwrap();
            cache.insert(term, fresh_var.clone());
            fresh_var
        }
    }
}

fn print_with_intermediate_vars(termdag: &TermDag, term: Term) -> String {
    let mut printed = String::new();
    let mut cache = HashMap::<Term, String>::new();
    let res = print_with_intermediate_helper(termdag, term, &mut cache, &mut printed);
    printed.push_str(&format!("(let PROG {res})\n"));
    printed
}

pub fn build_program(program: &TreeProgram) -> String {
    let (term, termdag) = program.to_egglog();
    let printed = print_with_intermediate_vars(&termdag, term);
    format!(
        "{}\n{printed}\n{}\n",
        prologue(),
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

fn check_func_type(func: RcExpr) -> Result {
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

    egglog::EGraph::default()
        .parse_and_run_program(&s)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        })?;
    Ok(())
}

fn check_program_gets_type(program: TreeProgram) -> Result {
    check_func_type(program.entry)?;
    for func in program.functions {
        check_func_type(func)?;
    }
    Ok(())
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
        let (result_val, print_log) = interpret_tree_prog(&prog, &input);
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
        include_str!("schedule.egg"),
    );

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
