use interpreter::Value;
use schema::TreeProgram;

use crate::interpreter::interpret;

pub mod ast;
pub mod interpreter;
mod optimizations;
pub mod schema;
pub mod schema_helpers;
mod to_egglog;

pub type Result = std::result::Result<(), egglog::Error>;

/// Runs an egglog test.
/// `build` is egglog code that runs before the running rules.
/// `check` is egglog code that runs after the running rules.
/// It is highly reccomended to also provide the programs used in the egglog code
/// so that they can be interpreted on the given value.
pub fn egglog_test(
    build: &str,
    check: &str,
    progs: Vec<TreeProgram>,
    input: Value,
    expected: Value,
) -> Result {
    // first interpret the programs on the value
    for prog in progs {
        let result = interpret(&prog, input.clone());
        assert_eq!(
            result, expected,
            "Program {:?} produced {} instead of expected {}",
            prog, result, expected
        );
    }

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
