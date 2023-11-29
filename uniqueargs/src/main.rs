// Rust test modules
// If you don't put your Rust file here it won't get compiled!
mod switch_rewrites;

pub type Result = std::result::Result<(), egglog::Error>;

pub fn run_test(a: &str, b: &str) -> Result {
    let program = format!(
        "
        {}
        (let test_a {a})
        (let test_b {b})
        {}
        (check (= test_a test_b))",
        vec![
            include_str!("schema.egg"),
            // analyses
            // repairs
            include_str!("util.egg"),
            include_str!("deep_copy.egg"),
            // optimizations
            include_str!("switch_rewrites.egg"),
        ]
        .join("\n"),
        include_str!("schedule.egg"),
    );

    egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map(|_| ())
}
