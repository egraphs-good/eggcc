use std::collections::HashSet;

use eggcc::util::{Run, RunMode, TestProgram};
use insta::assert_snapshot;
use libtest_mimic::Trial;

/// Generate tests for all configurations of a given file
// slow_test means the test is too slow to run the interpreter on, so use benchmarking mode
fn generate_tests(glob: &str, slow_test: bool) -> Vec<Trial> {
    let mut trials = vec![];

    let mut mk_trial = |run: Run, snapshot: bool| {
        let snapshot_configurations: HashSet<RunMode> = [RunMode::Optimize].into_iter().collect();
        let test_name = run.name() + if snapshot { "_snapshot" } else { "" };

        trials.push(Trial::test(test_name, move || {
            let result = match run.run() {
                Err(error) => {
                    panic!("{}", error);
                }
                Ok(res) => res,
            };
            if run.interp.should_interp()
                && result.original_interpreted.as_ref().unwrap()
                    != result.result_interpreted.as_ref().unwrap()
            {
                panic!(
                    "Interpreted result does not match expected:\nExpected: {}\nGot: {}",
                    result.original_interpreted.unwrap(),
                    result.result_interpreted.unwrap()
                );
            }
            // only assert a snapshot if we are in the "small" folder
            if snapshot && snapshot_configurations.contains(&run.test_type) {
                for visualization in result.visualizations {
                    assert_snapshot!(run.name() + &visualization.name, visualization.result);
                }
            }

            Ok(())
        }))
    };

    for entry in glob::glob(glob).unwrap() {
        let f = entry.unwrap();

        let snapshot = f.to_str().unwrap().contains("small");

        let configurations = if slow_test {
            // in benchmark mode, run a special test pipeline that only runs
            // a few modes, and shares intermediate results
            vec![Run::test_benchmark_config(TestProgram::BrilFile(f.clone()))]
        } else {
            Run::all_configurations_for(TestProgram::BrilFile(f))
        };

        for run in configurations {
            mk_trial(run, snapshot);
        }
    }

    trials
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let mut tests = generate_tests("tests/passing/**/*.bril", false);
    tests.extend(generate_tests("tests/slow/**/*.bril", true));
    // also generate tests for benchmarks
    tests.extend(generate_tests("benchmarks/passing/**/*.bril", true));

    libtest_mimic::run(&args, tests).exit();
}
