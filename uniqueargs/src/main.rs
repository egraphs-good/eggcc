use egglog::EGraph;

fn main() {
    let prog = include_str!("schema.egg");

    let mut egraph = EGraph::default();
    match egraph.parse_and_run_program(prog) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {}", e),
    }
}
