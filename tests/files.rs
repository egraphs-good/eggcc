use eggcc::util::Run;
use insta::assert_snapshot;
use libtest_mimic::Trial;

fn generate_tests(glob: &str) -> Vec<Trial> {
    let mut trials = vec![];
    let mut mk_trial = |run: Run| {
        trials.push(Trial::test(run.name(), move || {
            let result = run.run();

            if let Some(interpreted) = result.result_interpreted {
                assert_eq!(result.original_interpreted, interpreted);
            } else {
                assert_snapshot!(run.name(), result.visualization);
            }
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

        // TODO optimizer doesn't support these yet
        let banned = ["queens_func", "unstructured", "implicit_return", "fib"];
        if banned.iter().any(|b| name.contains(b)) || f.to_str().unwrap().contains("failing") {
            continue;
        }

        for run in Run::all_configurations_for(f) {
            mk_trial(run);
        }
    }

    trials
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let tests = generate_tests("tests/**/*.bril");
    libtest_mimic::run(&args, tests).exit();
}
