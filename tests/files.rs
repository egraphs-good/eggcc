use std::path::PathBuf;
use glob::GlobResult;

use eggcc::*;
use insta::assert_snapshot;
use libtest_mimic::Trial;
use brilirs;

#[derive(Clone)]
struct Run {
    path: PathBuf,
    test_structured: bool,
    no_opt: bool,
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
        name
    }

    fn run(&self) {
        let program_read = std::fs::read_to_string(self.path.clone()).unwrap();
        if self.test_structured {
            let structured = Optimizer::parse_to_structured(&program_read).unwrap();
            assert_snapshot!(self.name(), format!("{}", structured));
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
        };
        // TODO optimizer doesn't support these yet
        let banned = [
            "diamond",
            "fib",
            "queens_func",
            "unstructured",
            "implicit_return",
        ];
        if !banned.iter().any(|b| name.contains(b)) {
            mk_trial(run.clone());
            mk_trial(Run {
                no_opt: true,
                ..run.clone()
            });
        }
        if f.to_str().unwrap().contains("small") && !name.contains("unstructured") {
            mk_trial(Run {
                test_structured: true,
                ..run
            });
        }
    }

    trials
}

fn make_interp_test(gr: GlobResult) -> Trial {
    let path = gr.unwrap();
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(String::from)
        .unwrap();

    Trial::test(name.clone(), move || {
        let mut args: Vec<String> = Vec::new();

        let f_string = std::fs::read_to_string(path.clone()).unwrap();

        // read in the first line, parse it if it has turnt's # ARGS: command.
        if let Some(first_line) = f_string.split("\n").next() {
            if first_line.contains("# ARGS:") {
                for arg in first_line["# ARGS: ".len()..]
                    .split(" ")
                    .map(|s| s.to_string()) {
                    args.push(arg);
                }
            }
        }

        let mut out_buf = Vec::new();

        brilirs::run_input(
            std::io::BufReader::new(f_string.as_bytes()),
            std::io::BufWriter::new(&mut out_buf),
            &args,
            false,
            std::io::stderr(),
            false,
            true,
            None
        )?;

        let mut check_path = path.clone();
        check_path.set_extension("out");

        let out = std::fs::read_to_string(check_path.clone()).unwrap();

        assert_eq!(String::from_utf8(out_buf).unwrap(), out);
        Ok(())
    })
}

fn generate_interp_tests(glob: &str) -> Vec<Trial> {
    let trials: Vec<Trial> = glob::glob(glob)
        .unwrap()
        .map(make_interp_test)
        .collect();
    trials
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let mut tests = generate_tests("tests/{small,snapshots}/*.bril");
    tests.append(&mut generate_interp_tests("tests/brils/**/*.bril"));
    libtest_mimic::run(&args, tests).exit();
}
