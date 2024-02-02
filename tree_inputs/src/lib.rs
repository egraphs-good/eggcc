use schema::Program;

pub mod schema;

pub type Result = std::result::Result<(), egglog::Error>;

pub fn run_test(build: &str, check: &str, progs: Vec<Program>) -> Result {
    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        [include_str!("schema.egg"),].join("\n"),
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
