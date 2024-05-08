use std::collections::HashSet;

use eggcc::util::{InterpMode, Run, RunType, TestProgram};
use insta::assert_snapshot;
use libtest_mimic::Trial;

/// Generate tests for all configurations of a given file
/// If `just_brilift` is true, only generate tests that
/// run the full pipeline with brilift
fn generate_tests(glob: &str, benchmark_mode: bool) -> Vec<Trial> {
    let mut trials = vec![];

    let mut mk_trial = |run: Run, snapshot: bool| {
        let snapshot_configurations: HashSet<RunType> = [RunType::Optimize].into_iter().collect();

        trials.push(Trial::test(run.name(), move || {
            let result = match run.run() {
                Err(error) => {
                    panic!("{}", error);
                }
                Ok(res) => res,
            };
            if run.test_type == RunType::CompileBrilift || run.test_type == RunType::CompileBrilLLVM
            {
                let executable = run
                    .output_path
                    .clone()
                    .unwrap_or_else(|| format!("/tmp/{}", run.name()));
                std::process::Command::new("rm")
                    .args(vec![executable.clone(), executable + "-args"])
                    .status()
                    .unwrap();
            }

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

        // in benchmark mode, run fewer tests and use binaries to interpret
        let configurations = if benchmark_mode {
            // use InterpFast, which runs brilift
            // with optimization but without egglog optimization
            vec![
                Run::compile_brilift_config(
                    TestProgram::BrilFile(f.clone()),
                    true,
                    true,
                    InterpMode::InterpFast,
                ),
                Run::compile_llvm_config(
                    TestProgram::BrilFile(f.clone()),
                    true,
                    true,
                    InterpMode::InterpFast,
                ),
                Run::compile_llvm_config(
                    TestProgram::BrilFile(f),
                    false,
                    true,
                    InterpMode::InterpFast,
                ),
            ]
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
    // also generate tests for benchmarks
    tests.extend(generate_tests("benchmarks/passing/**/*.bril", true));

    libtest_mimic::run(&args, tests).exit();
}
