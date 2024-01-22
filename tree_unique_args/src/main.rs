#![allow(clippy::useless_format)]

use main_error::MainError;

// Rust test modules
// If you don't put your Rust file here it won't get compiled!
pub(crate) mod body_contains;
pub(crate) mod conditional_invariant_code_motion;
pub(crate) mod deep_copy;
pub(crate) mod function_inlining;
pub(crate) mod ir;
pub(crate) mod is_valid;
pub(crate) mod loop_unrolling;
pub(crate) mod purity_analysis;
pub(crate) mod simple;
pub(crate) mod subst;
pub(crate) mod switch_rewrites;
pub(crate) mod util;

pub type Result = std::result::Result<(), egglog::Error>;

// Might be useful for typechecking?
fn main() -> std::result::Result<(), MainError> {
    run_test("", "").map_err(|e| e.into())
}

pub fn run_test(build: &str, check: &str) -> Result {
    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        [
            include_str!("schema.egg"),
            // analyses
            &is_valid::rules().join("\n"),
            &purity_analysis::purity_analysis_rules().join("\n"),
            &body_contains::rules().join("\n"),
            &subst::subst_rules().join("\n"),
            &deep_copy::deep_copy_rules().join("\n"),
            include_str!("sugar.egg"),
            include_str!("util.egg"),
            // optimizations
            include_str!("simple.egg"),
            include_str!("function_inlining.egg"),
            &switch_rewrites::rules(),
            &conditional_invariant_code_motion::rules().join("\n"),
            include_str!("loop_unrolling.egg"),
        ]
        .join("\n"),
        include_str!("schedule.egg"),
    );

    println!("{}", program);

    egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        })
}
