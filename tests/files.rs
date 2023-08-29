use std::path::PathBuf;

use eggcc::*;
use insta::assert_snapshot;
use libtest_mimic::Trial;

#[derive(Clone, Copy)]
enum TestType {
    StructuredConversion,
    RvsdgConversion,
    NaiiveOptimization,
}

#[derive(Clone)]
struct Run {
    path: PathBuf,
    test_type: TestType,
    // Don't perform any optimizations, just do round-trip
    no_opt: bool,
    // Take a snapshot of the result
    snapshot: bool,
}

impl Run {
    fn name(&self) -> String {
        let mut name = self.path.file_stem().unwrap().to_str().unwrap().to_string();
        match self.test_type {
            TestType::StructuredConversion => name = format!("{}_structured", name),
            TestType::RvsdgConversion => name = format!("{}_rvsdg_conversion", name),
            TestType::NaiiveOptimization => name = format!("{}_naiive", name),
        }
        if self.no_opt {
            name = format!("{}_no_opt", name);
        }
        name
    }

    fn run(&self) {
        let program_read = std::fs::read_to_string(self.path.clone()).unwrap();
        match self.test_type {
            TestType::StructuredConversion => {
                let structured = Optimizer::parse_to_structured(&program_read).unwrap();
                if self.snapshot {
                    assert_snapshot!(self.name(), format!("{}", structured));
                }
            }
            TestType::RvsdgConversion => {
                todo!()
            }
            TestType::NaiiveOptimization => {
                let parsed = Optimizer::parse_bril(&program_read).unwrap();

                let mut optimizer = Optimizer::default();
                if self.no_opt {
                    optimizer.num_iters = 0;
                }
                let res = optimizer.optimize(&parsed).unwrap();

                let args = Optimizer::parse_bril_args(&program_read);
                assert_eq!(
                    Optimizer::interp(&program_read, args.clone(), None),
                    Optimizer::interp(&format!("{}", res), args, None)
                );

                if self.snapshot {
                    assert_snapshot!(self.name(), format!("{}", res));
                }
            }
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
            test_type: TestType::NaiiveOptimization,
            no_opt: false,
            snapshot: f.to_str().unwrap().contains("small"),
        };

        // TODO optimizer doesn't support these yet
        let banned = ["queens_func", "unstructured", "implicit_return"];
        if banned.iter().any(|b| name.contains(b)) || f.to_str().unwrap().contains("failing") {
            continue;
        }

        mk_trial(run.clone());
        mk_trial(Run {
            no_opt: true,
            ..run.clone()
        });

        mk_trial(Run {
            test_type: TestType::StructuredConversion,
            ..run
        });
    }

    trials
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let tests = generate_tests("tests/**/*.bril");
    libtest_mimic::run(&args, tests).exit();
}
