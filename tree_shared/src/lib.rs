pub mod ast;
pub(crate) mod deep_copy;
pub(crate) mod error_checking;
pub(crate) mod expr;
pub(crate) mod id_analysis;
pub mod interpreter;
pub(crate) mod ir;
pub(crate) mod is_valid;
pub(crate) mod purity_analysis;
pub(crate) mod simple;
pub(crate) mod subst;
pub(crate) mod util;

pub type Result = std::result::Result<(), egglog::Error>;

pub fn run_test(build: &str, check: &str) -> Result {
    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        [
            include_str!("schema.egg"),
            // analyses
            &is_valid::rules().join("\n"),
            &purity_analysis::purity_analysis_rules().join("\n"),
            &subst::subst_rules().join("\n"),
            &deep_copy::deep_copy_rules().join("\n"),
            include_str!("sugar.egg"),
            &util::rules().join("\n"),
            &error_checking::error_checking_rules().join("\n"),
            &id_analysis::id_analysis_rules().join("\n"),
            // optimizations
            include_str!("simple.egg"),
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
