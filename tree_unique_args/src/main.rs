// Rust test modules
// If you don't put your Rust file here it won't get compiled!
pub(crate) mod ir;
pub(crate) mod purity_analysis;

pub type Result = std::result::Result<(), egglog::Error>;

// Might be useful for typechecking?
fn main() -> Result {
    run_test("", "")
}

pub fn run_test(build: &str, check: &str) -> Result {
    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        vec![
            include_str!("schema.egg"),
            include_str!("sugar.egg"),
            // analyses
            &purity_analysis::purity_analysis_rules().join("\n"),
            // repairs
            // optimizations
        ]
        .join("\n"),
        include_str!("schedule.egg"),
    );

    println!("{}", program);

    egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map(|_| ())
}
