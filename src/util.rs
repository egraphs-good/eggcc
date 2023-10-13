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
        // if there's an interpreted value do that as well
        if let Some(interpreted) = result.result_interpreted {
            let mut output_path = output_dir.clone();
            output_path.push(format!("{}-interp.txt", run.name()));
            let mut file = File::create(output_path)?;
            file.write_all(interpreted.as_bytes())?;
        }

        for visualization in result.visualizations {
            let mut output_path = output_dir.clone();

            assert!(output_path.is_dir());
            output_path.push(format!(
                "{}{}{}",
                run.name(),
                visualization.name,
                visualization.file_extension
            ));
            let mut file = File::create(output_path)?;
            file.write_all(visualization.result.as_bytes())?;
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

/// Different ways to run eggcc- the default is RvsdgOptimize,
/// but these others are useful for testing and debugging.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunType {
    /// Do nothing to the input bril program besides parse it.
    /// Output the original program.
    Nothing,
    /// Convert the input bril program to a structured
    /// format.
    /// Output a human-readable debug version of this structured
    /// program, using while loops and if statements.
    StructuredConversion,
    /// Convert the input bril program to an RVSDG and output it as an SVG.
    RvsdgConversion,
    /// Convert the input bril program to a PEG and output it in graphviz dot format.
    PegConversion,
    /// Convert to RVSDG and back to Bril again,
    /// outputting the bril program.
    RvsdgRoundTrip,
    /// Convert the original program to a CFG and output it as one SVG per function.
    ToCfg,
    /// Convert the original program to a CFG and back to Bril again.
    CfgRoundTrip,
    /// Convert the original program to a RVSDG and then to a CFG, outputting one SVG per function.
    RvsdgToCfg,
    /// Convert the original program to a RVSDG, optimize it, then turn
    /// it back into a Bril program. This is the main way to run eggcc.
    RvsdgOptimize,
    /// Convert the original program to a RVSDG, optimize it, then output
    /// the optimized RVSDG as an SVG.
    OptimizedRvsdg,
    /// Convert the original program to a RVSDG, then output the egglog program
    /// that is used to optimize the RVSDG.
    RvsdgEgglogEncoding,
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
            "nothing" => Ok(RunType::Nothing),
            "structured" => Ok(RunType::StructuredConversion),
            "rvsdg" => Ok(RunType::RvsdgConversion),
            "rvsdg-roundtrip" => Ok(RunType::RvsdgRoundTrip),
            "to-cfg" => Ok(RunType::ToCfg),
            "peg" => Ok(RunType::PegConversion),
            "cfg-roundtrip" => Ok(RunType::CfgRoundTrip),
            "rvsdg-to-cfg" => Ok(RunType::RvsdgToCfg),
            "rvsdg-optimize" => Ok(RunType::RvsdgOptimize),
            "rvsdg-egglog-encoding" => Ok(RunType::RvsdgEgglogEncoding),
            "optimized-rvsdg" => Ok(RunType::OptimizedRvsdg),
            _ => Err(format!("Unknown run type: {}", s)),
        }
    }
}

impl Display for RunType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RunType::Nothing => write!(f, "nothing"),
            RunType::StructuredConversion => write!(f, "structured"),
            RunType::RvsdgConversion => write!(f, "rvsdg"),
            RunType::PegConversion => write!(f, "peg"),
            RunType::RvsdgRoundTrip => write!(f, "rvsdg-roundtrip"),
            RunType::ToCfg => write!(f, "to-cfg"),
            RunType::CfgRoundTrip => write!(f, "cfg-roundtrip"),
            RunType::RvsdgToCfg => write!(f, "rvsdg-to-cfg"),
            RunType::RvsdgOptimize => write!(f, "rvsdg-optimize"),
            RunType::OptimizedRvsdg => write!(f, "optimized-rvsdg"),
            RunType::RvsdgEgglogEncoding => write!(f, "rvsdg-egglog-encoding"),
        }
    }
}

impl RunType {
    pub fn produces_bril(&self) -> bool {
        match self {
            RunType::Nothing => true,
            RunType::StructuredConversion => false,
            RunType::RvsdgConversion => false,
            RunType::PegConversion => false,
            RunType::RvsdgRoundTrip => true,
            RunType::ToCfg => true,
            RunType::CfgRoundTrip => true,
            RunType::RvsdgToCfg => true,
            RunType::RvsdgOptimize => true,
            RunType::RvsdgEgglogEncoding => true,
            RunType::OptimizedRvsdg => false,
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
    pub profile_out: Option<PathBuf>,
}

/// Some sort of visualization of the result, with a name
/// and a file extension.
/// For CFGs, the name is the name of the function and the vizalization
/// is a SVG.
#[derive(Clone)]
pub struct Visualization {
    pub result: String,
    pub file_extension: String,
    pub name: String,
}

#[derive(Clone)]
pub struct RunOutput {
    pub visualizations: Vec<Visualization>,
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
            RunType::RvsdgRoundTrip,
            RunType::PegConversion,
            RunType::CfgRoundTrip,
            RunType::RvsdgOptimize,
            RunType::RvsdgToCfg,
        ] {
            let default = Run {
                test_type,
                interp: false,
                prog_with_args: prog.clone(),
                profile_out: None,
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

        let mut peg = None;
        let (visualizations, bril_out) = match self.test_type {
            RunType::Nothing => (vec![], Some(self.prog_with_args.program.clone())),
            RunType::StructuredConversion => {
                let structured =
                    Optimizer::program_to_structured(&self.prog_with_args.program).unwrap();
                (
                    vec![Visualization {
                        result: structured.to_string(),
                        file_extension: ".txt".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
            RunType::RvsdgConversion => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                let svg = rvsdg.to_svg();
                (
                    vec![Visualization {
                        result: svg,
                        file_extension: ".svg".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
            RunType::RvsdgRoundTrip => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                let cfg = rvsdg.to_cfg();
                let bril = cfg.to_bril();
                (
                    vec![Visualization {
                        result: bril.to_string(),
                        file_extension: ".bril".to_string(),
                        name: "".to_string(),
                    }],
                    Some(bril),
                )
            }
            RunType::RvsdgToCfg => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                let cfg = rvsdg.to_cfg();
                (cfg.visualizations(), None)
            }
            RunType::ToCfg => {
                let cfg = Optimizer::program_to_cfg(&self.prog_with_args.program);
                (cfg.visualizations(), None)
            }
            RunType::PegConversion => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                peg = Some(rvsdg_to_peg(&rvsdg));
                let dot = peg.as_ref().unwrap().graph();
                (
                    vec![Visualization {
                        result: dot,
                        file_extension: ".dot".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
            RunType::CfgRoundTrip => {
                let cfg = Optimizer::program_to_cfg(&self.prog_with_args.program);
                let bril = cfg.to_bril();
                (
                    vec![Visualization {
                        result: bril.to_string(),
                        file_extension: ".bril".to_string(),
                        name: "".to_string(),
                    }],
                    Some(bril),
                )
            }
            RunType::RvsdgOptimize => {
                let optimized = Optimizer::rvsdg_optimize(&self.prog_with_args.program).unwrap();
                (
                    vec![Visualization {
                        result: optimized.to_string(),
                        file_extension: ".bril".to_string(),
                        name: "".to_string(),
                    }],
                    Some(optimized),
                )
            }
            RunType::RvsdgEgglogEncoding => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                let (egglog_code, _) = rvsdg.build_egglog_code();
                (
                    vec![Visualization {
                        result: egglog_code,
                        file_extension: ".egglog".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
            RunType::OptimizedRvsdg => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program).unwrap();
                let optimized = rvsdg.optimize().unwrap();
                let svg = optimized.to_svg();
                (
                    vec![Visualization {
                        result: svg,
                        file_extension: ".svg".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
        };

        let result_interpreted = match bril_out {
            Some(program) if self.interp => {
                assert!(self.test_type.produces_bril());
                Some(Optimizer::interp(
                    &program,
                    self.prog_with_args.args.clone(),
                    self.profile_out.clone(),
                ))
            }
            _ if self.test_type == RunType::PegConversion && self.interp => {
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
            visualizations,
            result_interpreted,
            original_interpreted,
        }
    }
}

pub(crate) struct FreshNameGen {
    next: usize,
}

impl FreshNameGen {
    pub(crate) fn new() -> Self {
        Self { next: 0 }
    }

    pub(crate) fn fresh(&mut self) -> String {
        let name = format!("v{}", self.next);
        self.next += 1;
        name
    }

    pub(crate) fn fresh_usize(&mut self) -> usize {
        let name = self.next;
        self.next += 1;
        name
    }
}
