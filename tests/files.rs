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
            let mut args = Vec::new();

            if let Some(first_line) = program_read.split('\n').next() {
                if first_line.contains("# ARGS:") {
                    for arg in first_line["# ARGS: ".len()..]
                        .split(' ')
                        .map(|s| s.to_string())
                    {
                        args.push(arg);
                    }
                }
            }

            // interp with no optimizations
            let mut base_line_out = Vec::new();
            brilirs::run_input(
                std::io::BufReader::new(program_read.as_bytes()),
                std::io::BufWriter::new(&mut base_line_out),
                &args,
                false,
                std::io::stderr(),
                false,
                true,
                None,
            )
            .unwrap();

            // TODO: comment next line out and uncomment the rest when we support all of bril!
            let res = program_read;
            // let parsed = Optimizer::parse_bril(&program_read).unwrap();
            // let mut optimizer = Optimizer::default();
            // let res = optimizer.optimize(&parsed).unwrap();

            let mut optimized_out = Vec::new();
            brilirs::run_input(
                std::io::BufReader::new(res.to_string().as_bytes()),
                std::io::BufWriter::new(&mut optimized_out),
                &args,
                false,
                std::io::stderr(),
                false,
                true,
                None,
            )
            .unwrap();

            assert_eq!(
                String::from_utf8(base_line_out).unwrap(),
                String::from_utf8(optimized_out).unwrap()
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
        if f.to_str().unwrap().contains("brils") {
            mk_trial(Run {
                interp: true,
                ..run.clone()
            });
            continue;
        }

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

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let tests = generate_tests("tests/**/*.bril");
    libtest_mimic::run(&args, tests).exit();
}
