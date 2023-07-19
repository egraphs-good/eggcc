use std::path::PathBuf;

use eggcc::*;
use libtest_mimic::Trial;

#[derive(Clone)]
struct Run {
    path: PathBuf,
}

impl Run {
    fn run(&self) {
        let program_read = std::fs::read_to_string(self.path.clone()).unwrap();
        let optimizer = Optimizer {};
        let res = optimizer.optimize(&program_read).unwrap();
        println!("{}", res);
    }
}

fn generate_tests(glob: &str) -> Vec<Trial> {
    let mut trials = vec![];
    let mut mk_trial = |name: String, run: Run| {
        trials.push(Trial::test(name, move || {
            run.run();
            Ok(())
        }))
    };

    for entry in glob::glob(glob).unwrap() {
        let f = entry.unwrap();
        let name = f
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace(['.', '-', ' '], "_");

        mk_trial(name.clone(), Run { path: f.clone() });
    }

    trials
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let tests = generate_tests("tests/**/*.bril");
    libtest_mimic::run(&args, tests).exit();
}
