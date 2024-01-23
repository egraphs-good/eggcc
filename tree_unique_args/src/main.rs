#![allow(clippy::useless_format)]

use main_error::MainError;

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
