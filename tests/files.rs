use std::collections::HashSet;

use eggcc::util::{Run, RunType, TestProgram};
use insta::assert_snapshot;
use libtest_mimic::Trial;

fn generate_tests(glob: &str) -> Vec<Trial> {
    let mut trials = vec![];

    let mut mk_trial = |run: Run, snapshot: bool| {
        let snapshot_configurations: HashSet<RunType> = [].into_iter().collect();

        trials.push(Trial::test(run.name(), move || {
            let result = match run.run() {
                Err(error) => {
                    panic!("{}", error);
                }
                Ok(res) => res,
            };
            if run.test_type == RunType::CompileBrilift || run.test_type == RunType::CompileBrilLLVM
            {
                let executable = run.output_path.clone().unwrap_or_else(|| run.name());

                let args = if run.test_type == RunType::CompileBrilLLVM {
                    vec![executable]
                } else {
                    vec![executable.clone(), executable + "-args"]
                };

                std::process::Command::new("rm")
                    .args(args)
                    .status()
                    .unwrap();
            }

            if result.result_interpreted.is_some() {
                if result.original_interpreted != result.result_interpreted {
                    panic!(
                        "Interpreted result does not match expected:\nExpected: {}\nGot: {}",
                        result.original_interpreted.unwrap(),
                        result.result_interpreted.unwrap()
                    );
                }
            } else {
                // only assert a snapshot if we are in the "small" folder
                if snapshot && snapshot_configurations.contains(&run.test_type) {
                    for visualization in result.visualizations {
                        assert_snapshot!(run.name() + &visualization.name, visualization.result);
                    }
                }
            }
            Ok(())
        }))
    };

    for entry in glob::glob(glob).unwrap() {
        let f = entry.unwrap();

        if f.iter().any(|folder| folder == "should_fail")
            || f.iter().any(|folder| folder == "failing")
        {
            continue;
        }

        let snapshot = f.to_str().unwrap().contains("small");

        for run in Run::all_configurations_for(TestProgram::BrilFile(f)) {
            mk_trial(run, snapshot);
        }
    }

    trials
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let tests = generate_tests("tests/**/*.bril");
    libtest_mimic::run(&args, tests).exit();
}
