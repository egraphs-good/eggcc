use crate::{EggCCError, Optimizer};
use bril_rs::Program;
use clap::ValueEnum;
use std::fmt::Debug;
use std::{
    ffi::OsStr,
    fmt::{Display, Formatter},
    io,
    path::PathBuf,
};
use tree_in_context::schema::TreeProgram;

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
        let result = match result {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Error running {:?}: {}", run.test_type, err);
                continue;
            }
        };
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

/// Different ways to run eggcc- the default is TreeConversion,
/// but these others are useful for testing and debugging.
#[derive(Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Debug)]
pub enum RunType {
    /// Do nothing to the input bril program besides parse it.
    /// Output the original program.
    Nothing,
    /// Convert the input bril program to an RVSDG and output it as an SVG.
    RvsdgConversion,
    /// Convert the input bril program to a tree-encoded expression.
    TreeConversion,
    /// Convert the input bril program to tree-encoded expression and optimize it with egglog.
    TreeOptimize,
    /// Convert to RVSDG and back to Bril again,
    /// outputting the bril program.
    RvsdgRoundTrip,
    /// Convert the original program to a CFG and output it as one SVG per function.
    ToCfg,
    /// Convert the original program to a CFG and back to Bril again.
    CfgRoundTrip,
    /// Removes unecessary direct
    /// jumps from the input program by
    /// converting it to a CFG, calling
    /// optimize_jumps, then converting it back to bril.
    OptimizeDirectJumps,
    /// Convert the original program to a RVSDG and then to a CFG, outputting one SVG per function.
    RvsdgToCfg,
}

impl Display for RunType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

impl RunType {
    /// Returns true if the run type produces an IR
    /// that can be interpreted.
    pub fn produces_interpretable(&self) -> bool {
        match self {
            RunType::Nothing => true,
            RunType::RvsdgConversion => false,
            RunType::RvsdgRoundTrip => true,
            RunType::ToCfg => true,
            RunType::CfgRoundTrip => true,
            RunType::OptimizeDirectJumps => true,
            RunType::RvsdgToCfg => true,
            RunType::TreeConversion => true,
            RunType::TreeOptimize => true,
        }
    }

    pub fn supports_memory(&self) -> bool {
        match self {
            RunType::RvsdgRoundTrip | RunType::RvsdgConversion | RunType::Nothing => true,
            RunType::ToCfg
            | RunType::CfgRoundTrip
            | RunType::OptimizeDirectJumps
            | RunType::RvsdgToCfg
            | RunType::TreeConversion
            | RunType::TreeOptimize => false,
        }
    }
}

#[derive(Clone)]
pub struct ProgWithArguments {
    program: Program,
    name: String,
    has_mem: bool,
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
                // check if the path has a folder called mem in it
                let has_mem = path.iter().any(|x| x == "mem");
                let program_read = std::fs::read_to_string(path.clone()).unwrap();
                let args = Optimizer::parse_bril_args(&program_read);
                let program = Optimizer::parse_bril(&program_read).unwrap();
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();

                ProgWithArguments {
                    program,
                    name,
                    args,
                    has_mem,
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

/// an enum of IRs that can be interpreted
pub enum Interpretable {
    Bril(Program),
    TreeProgram(TreeProgram),
}

/// Some sort of visualization of the result, with a name
/// and a file extension.
/// For CFGs, the name is the name of the function and the vizalization
/// is a SVG.
#[derive(Clone, Debug)]
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
    pub original_interpreted: Option<String>,
}

impl Run {
    pub fn all_configurations_for(test: TestProgram) -> Vec<Run> {
        let prog = test.read_program();
        let mut res = vec![];
        for test_type in [
            RunType::RvsdgConversion,
            RunType::RvsdgRoundTrip,
            RunType::CfgRoundTrip,
            RunType::OptimizeDirectJumps,
            RunType::RvsdgToCfg,
            RunType::TreeConversion,
            RunType::TreeOptimize,
        ] {
            if prog.has_mem && !test_type.supports_memory() {
                continue;
            }
            let default = Run {
                test_type,
                interp: false,
                prog_with_args: prog.clone(),
                profile_out: None,
            };
            res.push(default.clone());
            if test_type.produces_interpretable() {
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

    pub fn run(&self) -> Result<RunOutput, EggCCError> {
        let original_interpreted = if self.interp {
            Some(Optimizer::interp_bril(
                &self.prog_with_args.program,
                self.prog_with_args.args.clone(),
                None,
            ))
        } else {
            None
        };

        let (visualizations, interpretable_out) = match self.test_type {
            RunType::Nothing => (
                vec![],
                Some(Interpretable::Bril(self.prog_with_args.program.clone())),
            ),
            RunType::RvsdgConversion => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
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
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let cfg = rvsdg.to_cfg();
                let bril = cfg.to_bril();
                (
                    vec![Visualization {
                        result: bril.to_string(),
                        file_extension: ".bril".to_string(),
                        name: "".to_string(),
                    }],
                    Some(Interpretable::Bril(bril)),
                )
            }
            RunType::TreeConversion => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let tree = rvsdg.to_tree_encoding();
                (
                    vec![Visualization {
                        result: tree.pretty(),
                        file_extension: ".egg".to_string(),
                        name: "".to_string(),
                    }],
                    Some(Interpretable::TreeProgram(tree)),
                )
            }
            RunType::TreeOptimize => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let tree = rvsdg.to_tree_encoding();
                let optimized = tree_in_context::optimize(&tree).map_err(EggCCError::EggLog)?;
                (
                    vec![Visualization {
                        result: optimized.pretty(),
                        file_extension: ".egg".to_string(),
                        name: "".to_string(),
                    }],
                    Some(Interpretable::TreeProgram(optimized)),
                )
            }
            RunType::RvsdgToCfg => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let cfg = rvsdg.to_cfg();
                (cfg.visualizations(), None)
            }
            RunType::ToCfg => {
                let cfg = Optimizer::program_to_cfg(&self.prog_with_args.program);
                (cfg.visualizations(), None)
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
                    Some(Interpretable::Bril(bril)),
                )
            }
            RunType::OptimizeDirectJumps => {
                let cfg = Optimizer::program_to_cfg(&self.prog_with_args.program);
                let optimized = cfg.optimize_jumps();
                let bril = optimized.to_bril();
                (
                    vec![Visualization {
                        result: bril.to_string(),
                        file_extension: ".bril".to_string(),
                        name: "".to_string(),
                    }],
                    Some(Interpretable::Bril(bril)),
                )
            }
        };

        let result_interpreted = if !self.interp {
            None
        } else {
            match interpretable_out {
                Some(program) if self.interp => {
                    assert!(self.test_type.produces_interpretable());
                    Some(Optimizer::interp(
                        &program,
                        self.prog_with_args.args.clone(),
                        self.profile_out.clone(),
                    ))
                }
                _ => None,
            }
        };

        Ok(RunOutput {
            visualizations,
            result_interpreted,
            original_interpreted,
        })
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
