use egglog::EGraph;

fn main() {
    let program = vec![
        // headers
        include_str!("schema.egg"),
        include_str!("util.egg"),
        include_str!("deep_copy.egg"),
        // optimizations
        include_str!("switch_rewrites.egg"),
        // execution
        include_str!("schedule.egg"),
        include_str!("tests.egg"),
    ]
    .join("\n");

    let mut egraph = EGraph::default();
    match egraph.parse_and_run_program(&program) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {}", e),
    }
}
