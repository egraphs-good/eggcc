use std::path::PathBuf;

use eggcc::*;
use insta::assert_snapshot;
use libtest_mimic::Trial;

#[derive(Clone)]
struct Run {
    path: PathBuf,
    test_structured: bool,
    no_opt: bool,
    interp: bool,
}

impl Run {
    fn name(&self) -> String {
        let mut name = self.path.file_stem().unwrap().to_str().unwrap().to_string();
        if self.test_structured {
            name = format!("{}_structured", name);
        }
        if self.no_opt {
            name = format!("{}_no_opt", name);
        }
        if self.interp {
            name = format!("{}_interp", name)
        }
        name
    }

    fn run(&self) {
        let program_read = std::fs::read_to_string(self.path.clone()).unwrap();
        if self.test_structured {
            let structured = Optimizer::parse_to_structured(&program_read).unwrap();
            assert_snapshot!(self.name(), format!("{}", structured));
        } else if self.interp {
            let parsed = Optimizer::parse_bril(&program_read).unwrap();
            let mut optimizer = Optimizer::default();
            let res = optimizer.optimize(&parsed).unwrap();

            assert_eq!(
                Optimizer::interp(&program_read),
                Optimizer::interp(&format!("{}", res))
            );
        } else {
            let parsed = Optimizer::parse_bril(&program_read).unwrap();

            let mut optimizer = Optimizer::default();
            if self.no_opt {
                optimizer.num_iters = 0;
            }
            let res = optimizer.optimize(&parsed).unwrap();

            assert_snapshot!(self.name(), format!("{}", res));
        }
    }
}

fn generate_tests(glob: &str) -> Vec<Trial> {
    let mut trials = vec![];
    let mut mk_trial = |run: Run| {
        trials.push(Trial::test(run.name(), move || {
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
        let run = Run {
            path: f.clone(),
            test_structured: false,
            no_opt: false,
            interp: false,
        };

        // TODO: make interp run on just about anything. For right now we don't want to treat
        // bril tests as snapshots
        if f.to_str().unwrap().contains("failing") {
            // uncomment this if you want all bril tests to run
            continue;
        }

        mk_trial(Run {
            interp: true,
            ..run.clone()
        });

        mk_trial(run.clone());
        mk_trial(Run {
            no_opt: true,
            ..run.clone()
        });

        if f.to_str().unwrap().contains("small") && !name.contains("unstructured") {
            mk_trial(Run {
                test_structured: true,
                ..run
            });
        }
    }

    trials
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let tests = generate_tests("tests/**/*.bril");
    libtest_mimic::run(&args, tests).exit();
}
