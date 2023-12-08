// Rust test modules
// If you don't put your Rust file here it won't get compiled!
mod switch_rewrites;
mod subst;
mod loop_strength_reduction;

pub type Result = std::result::Result<(), egglog::Error>;

pub fn main() -> () {}

pub fn run_test(build: &str, check: &str) -> Result {
    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        vec![
            include_str!("schema.egg"),
            // analyses
            include_str!("fast_analyses.egg"),
            include_str!("id_analysis.egg"),
            // repairs
            include_str!("util.egg"),
            include_str!("deep_copy.egg"),
            include_str!("subst.egg"),
            // optimizations
            include_str!("switch_rewrites.egg"),
            include_str!("loop_strength_reduction.egg"),
        ]
        .join("\n"),
        include_str!("schedule.egg"),
    );

    egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map(|_| ())
}
