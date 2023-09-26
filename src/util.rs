use crate::peg::rvsdg_to_peg;
use crate::Optimizer;
use bril_rs::{Literal, Program};
use std::fmt::Debug;
use std::{
    ffi::OsStr,
    fmt::{Display, Formatter},
    io,
    path::PathBuf,
    str::FromStr,
};

pub(crate) struct ListDisplay<'a, TS>(pub TS, pub &'a str);

impl<'a, TS> Display for ListDisplay<'a, TS>
where
    TS: Clone + IntoIterator,
    TS::Item: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut did_something = false;
        for item in self.0.clone().into_iter() {
            if did_something {
                f.write_str(self.1)?;
            }
            Display::fmt(&item, f)?;
            did_something = true;
        }
        Ok(())
    }
}

/// Parse a string containing a bril program (in text format) into a Program.
///
/// This function is intended for use in tests and in ad-hoc debugging.
#[allow(unused)]
pub(crate) fn parse_from_string(input: &str) -> bril_rs::Program {
    use bril2json::parse_abstract_program_from_read;
    use bril_rs::load_program_from_read;
    let abs_program = parse_abstract_program_from_read(input.as_bytes(), true, false, None);
    let mut buf = Vec::new();
    serde_json::to_writer_pretty(&mut buf, &abs_program).unwrap();
    buf.push(b'\n');
    let json_str = String::from_utf8(buf).unwrap();
    load_program_from_read(json_str.as_bytes())
}

/// Write the visualizations to output files in the output directory.
/// If the directory does not exist, it creates it.
/// If the directory contains any files whose names conflict with the
/// output files, it replaces them.
///
/// Like other utilities related to `DebugVisualizations`, this method is
/// only intended for debugging eggcc.
pub fn visualize(test: TestProgram, output_dir: PathBuf) -> io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    // make the directory if it doesn't exist
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }

    let all_configs = Run::all_configurations_for(test);

    let results = all_configs.iter().map(|run| (run, run.run()));

    for (run, result) in results {
        let mut output_path = output_dir.clone();
        assert!(output_path.is_dir());
        output_path.push(format!(
            "{}{}",
            run.name(),
            result.visualization_file_extension
        ));
        let mut file = File::create(output_path)?;
        if let Some(interpreted) = result.result_interpreted {
            file.write_all(interpreted.as_bytes())?;
        } else {
            file.write_all(result.visualization.as_bytes())?;
        }
    }

    Ok(())
}

/// Invokes some program with the given arguments, piping the given input to the program.
/// Returns an error if the program returns a non-zero exit code.
/// Code adapted from https://github.com/egraphs-good/egg/blob/e7845c5ae34267256b544c8e6b5bc36d91d096d2/src/dot.rs#L127
pub fn run_cmd_line<S1, S2, I>(program: S1, args: I, input: &str) -> std::io::Result<String>
where
    S1: AsRef<OsStr>,
    S2: AsRef<OsStr>,
    I: IntoIterator<Item = S2>,
{
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    write!(stdin, "{}", input)?;

    let output = child.wait_with_output()?;
    match output.status.code() {
        Some(0) => Ok(String::from_utf8(output.stdout).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("utf8 error: {}", e))
        })?),
        Some(e) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("program returned error code {}", e),
        )),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "program was killed by a signal",
        )),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RunType {
    StructuredConversion,
    RvsdgConversion,
    PegConversion,
    NaiiveOptimization,
}

impl Debug for RunType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for RunType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "structured" => Ok(RunType::StructuredConversion),
            "rvsdg" => Ok(RunType::RvsdgConversion),
            "naiive" => Ok(RunType::NaiiveOptimization),
            "peg" => Ok(RunType::PegConversion),
            _ => Err(format!("Unknown run type: {}", s)),
        }
    }
}

impl Display for RunType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RunType::StructuredConversion => write!(f, "structured"),
            RunType::RvsdgConversion => write!(f, "rvsdg"),
            RunType::PegConversion => write!(f, "peg"),
            RunType::NaiiveOptimization => write!(f, "naiive"),
        }
    }
}

impl RunType {
    pub fn produces_bril(&self) -> bool {
        match self {
            RunType::StructuredConversion => false,
            RunType::RvsdgConversion => false,
            RunType::PegConversion => false,
            RunType::NaiiveOptimization => true,
        }
    }
}

#[derive(Clone)]
pub struct ProgWithArguments {
    program: Program,
    name: String,
    args: Vec<String>,
}

#[derive(Clone)]
pub enum TestProgram {
    Prog(ProgWithArguments),
    File(PathBuf),
}

impl TestProgram {
    pub fn read_program(self) -> ProgWithArguments {
        match self {
            TestProgram::Prog(prog) => prog,
            TestProgram::File(path) => {
                let program_read = std::fs::read_to_string(path.clone()).unwrap();
                let args = Optimizer::parse_bril_args(&program_read);
                let program = Optimizer::parse_bril(&program_read).unwrap();
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();

                ProgWithArguments {
                    program,
                    name,
                    args,
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Run {
    pub prog_with_args: ProgWithArguments,
    pub test_type: RunType,
    // Also interpret the resulting program
    pub interp: bool,
}

#[derive(Clone)]
pub struct RunOutput {
    // a visualization of the result
    pub visualization: String,
    // a viable file extension for the visualization
    pub visualization_file_extension: String,
    // if the result was interpreted, the stdout of interpreting it
    pub result_interpreted: Option<String>,
    pub original_interpreted: String,
}

impl Run {
    pub fn all_configurations_for(test: TestProgram) -> Vec<Run> {
        let prog = test.read_program();
        let mut res = vec![];
        for test_type in [
            RunType::StructuredConversion,
            RunType::RvsdgConversion,
            RunType::PegConversion,
        ] {
            let default = Run {
                test_type,
                interp: false,
                prog_with_args: prog.clone(),
            };
            res.push(default.clone());
            if test_type.produces_bril() || test_type == RunType::PegConversion {
                let interp = Run {
                    interp: true,
                    ..default
                };
                res.push(interp);
            }
        }
        res
    }

    // give a unique name for this run configuration
    pub fn name(&self) -> String {
        let mut name = format!("{}-{}", self.prog_with_args.name, self.test_type);
        if self.interp {
            name = format!("{}-interp", name);
        }
        name
    }

    pub fn run(&self) -> RunOutput {
        let original_interpreted = Optimizer::interp(
            &self.prog_with_args.program,
            self.prog_with_args.args.clone(),
            None,
        );
        let mut optimized = None;
        let mut peg = None;
        let (visualization, visualization_file_extension) = match self.test_type {
            RunType::StructuredConversion => {
                let structured =
                    Optimizer::program_to_structured(&self.prog_with_args.program).unwrap();
                (structured.to_string(), ".txt")
            }
            RunType::RvsdgConversion => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                let svg = rvsdg.to_svg();
                (svg, ".svg")
            }
            RunType::NaiiveOptimization => {
                let mut optimizer = Optimizer::default();
                let res = optimizer.optimize(&self.prog_with_args.program).unwrap();
                let vis = (format!("{}", res), ".bril");
                optimized = Some(res);
                vis
            }
            RunType::PegConversion => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                peg = Some(rvsdg_to_peg(&rvsdg));
                let dot = peg.as_ref().unwrap().graph();
                let svg = run_cmd_line("dot", ["-Tsvg"], &dot).unwrap();
                (svg, ".svg")
            }
        };

        let result_interpreted = match optimized {
            Some(program) if self.interp => {
                assert!(self.test_type.produces_bril());
                Some(Optimizer::interp(
                    &program,
                    self.prog_with_args.args.clone(),
                    None,
                ))
            }
            _ if self.test_type == RunType::PegConversion => {
                let args: Vec<Literal> = self
                    .prog_with_args
                    .args
                    .iter()
                    .map(|s| {
                        if let Ok(int) = s.parse::<i64>() {
                            Literal::Int(int)
                        } else {
                            panic!("unsupported argument {s}")
                        }
                    })
                    .collect();
                Some(peg.unwrap().simulate(&args))
            }
            _ => None,
        };

        RunOutput {
            visualization,
            visualization_file_extension: visualization_file_extension.to_string(),
            result_interpreted,
            original_interpreted,
        }
    }
}
