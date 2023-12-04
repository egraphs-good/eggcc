use egglog::EGraph;

fn main() {
    let program = vec![
        // header
        include_str!("schema.egg"),
        // optimizations
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
