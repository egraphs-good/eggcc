use std::path::PathBuf;

use eggcc::*;
use insta::assert_snapshot;
use libtest_mimic::Trial;

#[derive(Clone)]
struct Run {
    path: PathBuf,
}

impl Run {
    fn run(&self) {
        let program_name = self.path.file_stem().unwrap().to_str().unwrap();
        let program_read = std::fs::read_to_string(self.path.clone()).unwrap();
        let parsed = Optimizer::parse_bril(&program_read).unwrap();

        let mut optimizer_nothing = Optimizer::default().with_num_iters(0);
        let res_nothing = optimizer_nothing.optimize(&parsed).unwrap();

        assert_snapshot!(format!("{program_name}_no_opt"), format!("{}", res_nothing));

        let mut optimizer = Optimizer::default();
        let res = optimizer.optimize(&parsed).unwrap();

        // TODO test res and res_nothing to make sure
        // they evaluate the same as the original program
        // The cost of evaluating both should also go down
        // compared to original

        assert_snapshot!(format!("{program_name}"), format!("{}", res));
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
