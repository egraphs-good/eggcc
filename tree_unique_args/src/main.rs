// Rust test modules
// If you don't put your Rust file here it won't get compiled!
pub(crate) mod body_contains;
pub(crate) mod deep_copy;
pub(crate) mod ir;
pub(crate) mod purity_analysis;
pub(crate) mod subst;

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
            // analyses
            &purity_analysis::purity_analysis_rules().join("\n"),
            &body_contains::rules().join("\n"),
            &subst::subst_rules().join("\n"),
            &deep_copy::deep_copy_rules().join("\n"),
            // repairs
            // optimizations
            // sugar
            include_str!("sugar.egg"),
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
