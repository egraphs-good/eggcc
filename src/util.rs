use crate::canonicalize_names::canonicalize_bril;
use crate::rvsdg::from_dag::dag_to_rvsdg;
use crate::{EggCCError, Optimizer};
use bril_rs::Program;
use clap::ValueEnum;
use dag_in_context::dag2svg::tree_to_svg;
use dag_in_context::{build_program, check_roundtrip_egraph};

use dag_in_context::schema::TreeProgram;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::{
    ffi::OsStr,
    fmt::{Display, Formatter},
    io,
    path::PathBuf,
};
use tempfile::tempdir;

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

// Get the eggcc repo root directory. Set by $EGGCC_ROOT, defaults to current
// directory.
fn get_eggcc_root() -> String {
    std::env::var("EGGCC_ROOT").unwrap_or(".".to_string())
}

/// Different ways to run eggcc
#[derive(Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Debug)]
pub enum RunType {
    /// Do nothing to the input bril program besides parse it.
    /// Output the original program.
    Parse,
    /// Convert a Bril file to the JSON representation
    BrilToJson,
    /// Convert the input bril program to the tree encoding, optimize the program
    /// using egglog, and output the resulting bril program.
    /// The default way to run this tool.
    Optimize,
    /// Convert the input bril program to an RVSDG and output it as an SVG.
    RvsdgConversion,
    /// Convert the input bril program to a tree-encoded expression.
    DagConversion,
    /// Convert the input bril program to tree-encoded expression and optimize it with egglog.
    DagOptimize,
    /// Convert the input bril program to a tree-encoded expression and optimize it with egglog,
    /// outputting the resulting RVSDG
    OptimizedRvsdg,
    /// Give the egglog program used to optimize the tree-encoded expression.
    Egglog,
    /// Check that converting the tree program to egglog
    /// and back again results in an identical program.
    CheckExtractIdentical,
    /// Convert to RVSDG and back to Bril again,
    /// outputting the bril program.
    RvsdgRoundTrip,
    /// Convert to RVSDG and back to Bril again
    /// Then convert to an executable using llvm-O0, without doing any optimization.
    RvsdgRoundTripToExecutable,
    /// Convert to Tree Encoding and back to Bril again,
    /// outputting the bril program.
    DagRoundTrip,
    /// Convert the program to a DAG reprensentation then back to an RVSDG.
    DagToRvsdg,
    /// Convert the original program to a CFG and output it as one SVG per function.
    ToCfg,
    /// Convert the original program to a CFG and back to Bril again.
    CfgRoundTrip,
    /// Optimize the original program, then stop on the way back at CFG
    OptimizedCfg,
    /// Removes unecessary direct
    /// jumps from the input program by
    /// converting it to a CFG, calling
    /// optimize_jumps, then converting it back to bril.
    OptimizeDirectJumps,
    /// Convert the original program to a RVSDG and then to a CFG, outputting one SVG per function.
    RvsdgToCfg,
    /// Converts to an executable using brilift (not using eggcc).
    /// Brilift does not support phi nodes right now, so we can't run the optimized program with it.
    Cranelift,
    /// Converts to an executable using brillvm.
    /// `optimize_egglog` and `optimize_bril_llvm` must be set.
    LLVM,
    /// Tests a benchmark by running several different configurations of CompileBrilLLVM
    /// and comparing the results.
    /// The different configurations are with and without egglog optimization, and with and without
    /// llvm optimization.
    TestBenchmark,
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
            RunType::Parse
            | RunType::Optimize
            | RunType::RvsdgRoundTrip
            | RunType::RvsdgRoundTripToExecutable
            | RunType::DagRoundTrip
            | RunType::CfgRoundTrip
            | RunType::OptimizeDirectJumps
            | RunType::DagConversion
            | RunType::DagOptimize
            | RunType::Cranelift
            | RunType::LLVM => true,
            RunType::RvsdgConversion
            | RunType::RvsdgToCfg
            | RunType::Egglog
            | RunType::DagToRvsdg
            | RunType::OptimizedRvsdg
            | RunType::CheckExtractIdentical
            | RunType::ToCfg
            | RunType::OptimizedCfg
            | RunType::TestBenchmark => false,
            RunType::BrilToJson => false,
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
    BrilFile(PathBuf),
    RustFile(PathBuf),
}

impl TestProgram {
    pub fn read_program(self) -> ProgWithArguments {
        match self {
            TestProgram::Prog(prog) => prog,
            TestProgram::BrilFile(path) => {
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
            TestProgram::RustFile(path) => {
                let mut src = String::new();
                let mut file = std::fs::File::open(path.clone()).unwrap();

                file.read_to_string(&mut src).unwrap();
                let syntax = syn::parse_file(&src).unwrap();
                let name = path.display().to_string();
                let program = rs2bril::from_file_to_program(syntax, false, Some(name.clone()));

                ProgWithArguments {
                    program,
                    name,
                    args: vec![],
                }
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum InterpMode {
    /// Interpret the original program and the result
    Interp,
    /// Interpret the original program as a brilift binary and the result
    InterpFast,
    None,
}

impl InterpMode {
    pub fn should_interp(&self) -> bool {
        match self {
            InterpMode::Interp | InterpMode::InterpFast => true,
            InterpMode::None => false,
        }
    }
}

#[derive(Clone)]
pub struct Run {
    pub prog_with_args: ProgWithArguments,
    pub test_type: RunType,
    pub interp: InterpMode,
    pub profile_out: Option<PathBuf>,
    pub llvm_output_dir: Option<String>,
    pub output_path: Option<String>,
    pub optimize_egglog: Option<bool>,
    pub optimize_brilift: Option<bool>,
    pub optimize_bril_llvm: Option<bool>,
}

/// an enum of IRs that can be interpreted
#[derive(Debug)]
pub enum Interpretable {
    Bril(Program),
    TreeProgram(TreeProgram),
    Executable { executable: String },
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
    fn optimize_bril(program: &Program) -> Result<Program, EggCCError> {
        let rvsdg = Optimizer::program_to_rvsdg(program)?;
        let dag = rvsdg.to_dag_encoding(true);
        let optimized = dag_in_context::optimize(&dag).map_err(EggCCError::EggLog)?;
        let rvsdg2 = dag_to_rvsdg(&optimized);
        let cfg = rvsdg2.to_cfg();
        let bril = cfg.to_bril();
        // re-name variables in the bril, hiding our nondeterminism bug ):
        let bril = canonicalize_bril(&bril);

        Ok(bril)
    }

    pub fn compile_brilift_config(
        test: TestProgram,
        optimize_brilift: bool,
        interp: InterpMode,
    ) -> Run {
        Run {
            test_type: RunType::Cranelift,
            interp,
            prog_with_args: test.read_program(),
            profile_out: None,
            output_path: None,
            llvm_output_dir: None,
            optimize_egglog: None,
            optimize_brilift: Some(optimize_brilift),
            optimize_bril_llvm: None,
        }
    }

    pub fn test_benchmark_config(test: TestProgram) -> Run {
        Run {
            test_type: RunType::TestBenchmark,
            // testing does interpretation separately, ignoring this flag
            interp: InterpMode::None,
            prog_with_args: test.read_program(),
            profile_out: None,
            output_path: None,
            llvm_output_dir: None,
            // no need to set optimization flags, since all combinations are tested
            optimize_egglog: None,
            optimize_brilift: None,
            optimize_bril_llvm: None,
        }
    }

    pub fn all_configurations_for(test: TestProgram) -> Vec<Run> {
        let prog = test.clone().read_program();
        let mut res = vec![];
        for test_type in [
            RunType::RvsdgConversion,
            RunType::RvsdgRoundTrip,
            RunType::CfgRoundTrip,
            RunType::OptimizeDirectJumps,
            RunType::RvsdgToCfg,
            RunType::DagConversion,
            RunType::DagOptimize,
            RunType::DagRoundTrip,
            RunType::Optimize,
            RunType::CheckExtractIdentical,
        ] {
            let default = Run {
                test_type,
                interp: InterpMode::None,
                prog_with_args: prog.clone(),
                profile_out: None,
                output_path: None,
                llvm_output_dir: None,
                optimize_egglog: None,
                optimize_brilift: None,
                optimize_bril_llvm: None,
            };
            if test_type.produces_interpretable() {
                let interp = Run {
                    interp: InterpMode::Interp,
                    ..default
                };
                res.push(interp);
            } else {
                res.push(default);
            }
        }

        // run a cranelift baseline
        res.push(Run::compile_brilift_config(
            test.clone(),
            true,
            InterpMode::Interp,
        ));

        #[cfg(feature = "llvm")]
        {
            for optimize_egglog in [true, false] {
                for optimize_brillvm in [true, false] {
                    res.push(Run {
                        test_type: RunType::LLVM,
                        interp: InterpMode::Interp,
                        prog_with_args: prog.clone(),
                        profile_out: None,
                        output_path: None,
                        llvm_output_dir: None,
                        optimize_egglog: Some(optimize_egglog),
                        optimize_brilift: None,
                        optimize_bril_llvm: Some(optimize_brillvm),
                    });
                }
            }
        }

        res
    }

    // give a unique name for this run configuration
    pub fn name(&self) -> String {
        let mut name = format!("{}-{}", self.prog_with_args.name, self.test_type);
        if self.test_type == RunType::Cranelift {
            name += match self.optimize_brilift.unwrap() {
                false => "-O0",
                true => "-O3",
            };
        }
        if self.test_type == RunType::LLVM {
            name += match (
                self.optimize_egglog.unwrap(),
                self.optimize_bril_llvm.unwrap(),
            ) {
                (false, false) => "-O0",
                (true, false) => "-O0-eggcc",
                (false, true) => "-O3",
                (true, true) => "-O3-eggcc",
            };
        }

        name
    }

    pub fn run(&self) -> Result<RunOutput, EggCCError> {
        let original_interpreted = if self.interp == InterpMode::Interp {
            Some(Optimizer::interp_bril(
                &self.prog_with_args.program,
                self.prog_with_args.args.clone(),
                None,
            ))
        } else if self.interp == InterpMode::InterpFast {
            let interpretable = self.run_brilift(self.prog_with_args.program.clone(), true);
            let res = Some(Optimizer::interp(
                interpretable.as_ref().unwrap(),
                self.prog_with_args.args.clone(),
                None,
            ));

            // clean up binary
            if let Interpretable::Executable { executable } = interpretable.unwrap() {
                std::fs::remove_file(executable).unwrap();
            }
            res
        } else {
            None
        };

        let (visualizations, interpretable_out) = match self.test_type {
            RunType::Parse => (
                vec![],
                Some(Interpretable::Bril(self.prog_with_args.program.clone())),
            ),
            RunType::BrilToJson => {
                let json = serde_json::to_string_pretty(&self.prog_with_args.program).unwrap();
                (
                    vec![Visualization {
                        result: json,
                        file_extension: ".json".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
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
            RunType::OptimizedCfg => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let cfg = rvsdg.to_cfg();
                (cfg.visualizations(), None)
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
            RunType::RvsdgRoundTripToExecutable => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let cfg = rvsdg.to_cfg();
                let bril = cfg.to_bril();
                let interpretable = self.run_bril_llvm(bril, false, false)?;
                (vec![], Some(interpretable))
            }
            RunType::DagToRvsdg => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let tree = rvsdg.to_dag_encoding(true);
                let rvsdg2 = dag_to_rvsdg(&tree);
                (
                    vec![Visualization {
                        result: rvsdg2.to_svg(),
                        file_extension: ".svg".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
            RunType::DagRoundTrip => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let tree = rvsdg.to_dag_encoding(true);
                let rvsdg2 = dag_to_rvsdg(&tree);
                let cfg = rvsdg2.to_cfg();
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
            RunType::CheckExtractIdentical => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let tree = rvsdg.to_dag_encoding(true);
                check_roundtrip_egraph(&tree);
                (vec![], None)
            }
            RunType::Optimize => {
                let bril = Run::optimize_bril(&self.prog_with_args.program)?;
                (
                    vec![Visualization {
                        result: bril.to_string(),
                        file_extension: ".bril".to_string(),
                        name: "".to_string(),
                    }],
                    Some(Interpretable::Bril(bril)),
                )
            }
            RunType::DagConversion => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let tree = rvsdg.to_dag_encoding(true);
                (
                    vec![Visualization {
                        result: tree_to_svg(&tree),
                        file_extension: ".svg".to_string(),
                        name: "".to_string(),
                    }],
                    Some(Interpretable::TreeProgram(tree)),
                )
            }
            RunType::DagOptimize => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let tree = rvsdg.to_dag_encoding(true);
                let optimized = dag_in_context::optimize(&tree).map_err(EggCCError::EggLog)?;
                (
                    vec![Visualization {
                        result: tree_to_svg(&optimized),
                        file_extension: ".svg".to_string(),
                        name: "".to_string(),
                    }],
                    Some(Interpretable::TreeProgram(optimized)),
                )
            }
            RunType::OptimizedRvsdg => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let dag = rvsdg.to_dag_encoding(true);
                let optimized = dag_in_context::optimize(&dag).map_err(EggCCError::EggLog)?;
                let rvsdg = dag_to_rvsdg(&optimized);
                (
                    vec![Visualization {
                        result: rvsdg.to_svg(),
                        file_extension: ".svg".to_string(),
                        name: "".to_string(),
                    }],
                    None,
                )
            }
            RunType::Egglog => {
                let rvsdg = Optimizer::program_to_rvsdg(&self.prog_with_args.program)?;
                let dag = rvsdg.to_dag_encoding(true);
                let egglog = build_program(&dag, true);
                (
                    vec![Visualization {
                        result: egglog,
                        file_extension: ".egg".to_string(),
                        name: "".to_string(),
                    }],
                    None,
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
            RunType::Cranelift => {
                let optimize_brilift = self.optimize_brilift.expect(
                    "optimize_brilift is a required flag when running RunMode::CompileBrilift",
                );
                let interpretable =
                    self.run_brilift(self.prog_with_args.program.clone(), optimize_brilift)?;
                (vec![], Some(interpretable))
            }
            RunType::LLVM => {
                let optimize_egglog = self.optimize_egglog.expect(
                    "optimize_egglog is a required flag when running RunMode::CompileBrilLLVM",
                );
                let optimize_brillvm = self.optimize_bril_llvm.expect(
                    "optimize_bril_llvm is a required flag when running RunMode::CompileBrilLLVM",
                );
                let interpretable = self.run_bril_llvm(
                    self.prog_with_args.program.clone(),
                    optimize_egglog,
                    optimize_brillvm,
                )?;
                (vec![], Some(interpretable))
            }
            RunType::TestBenchmark => {
                // optimize_egglog and optimize_brilift should not be set
                assert!(self.optimize_egglog.is_none());
                assert!(self.optimize_brilift.is_none());
                let cranelift_interpretable =
                    self.run_brilift(self.prog_with_args.program.clone(), true)?;

                // cranelift run gets compared to llvm runs
                let interpreted = Optimizer::interp(
                    &cranelift_interpretable,
                    self.prog_with_args.args.clone(),
                    None,
                );

                for optimize_egglog in [true, false] {
                    let resulting_bril = if optimize_egglog {
                        Run::optimize_bril(&self.prog_with_args.program)?
                    } else {
                        self.prog_with_args.program.clone()
                    };

                    for optimize_llvm in [true, false] {
                        let interpretable =
                            self.run_bril_llvm(resulting_bril.clone(), false, optimize_llvm)?;
                        let new_interpreted = Optimizer::interp(
                            &interpretable,
                            self.prog_with_args.args.clone(),
                            None,
                        );
                        if interpreted != new_interpreted {
                            panic!(
                                    "Interpreted outputs differ for {} with optimize_egglog={} and optimize_llvm={}.",
                                    self.name(), optimize_egglog, optimize_llvm
                                );
                        }
                    }
                }

                (vec![], None)
            }
        };

        let result_interpreted = if !(self.interp.should_interp()) {
            None
        } else {
            let Some(interpretable_out) = interpretable_out else {
                panic!(
                    "Interpretable output should be Some if interpret is set for {}.",
                    self.name()
                );
            };
            assert!(self.test_type.produces_interpretable());
            let res = Some(Optimizer::interp(
                &interpretable_out,
                self.prog_with_args.args.clone(),
                self.profile_out.clone(),
            ));

            // clean up binary
            if let Interpretable::Executable { executable } = interpretable_out {
                std::fs::remove_file(executable).unwrap();
            }

            res
        };

        Ok(RunOutput {
            visualizations,
            result_interpreted,
            original_interpreted,
        })
    }

    /// Brillift does not support phi nodes, so we can't
    /// run the optimized program with it.
    /// However, we can get a baseline comparison by running against it.
    fn run_brilift(
        &self,
        program: Program,
        optimize_brilift: bool,
    ) -> Result<Interpretable, EggCCError> {
        // For fast testing modes, we may run brilift to compare against
        // therefore we need a unique name based on this test's name and brilift options
        let unique_name = format!("{}_brilift_{}", self.name(), optimize_brilift);

        // Compile the input bril file
        // options are "none", "speed", and "speed_and_size"
        let opt_level = if optimize_brilift { "speed" } else { "none" };
        let object = format!("/tmp/{}.o", unique_name);
        brilift::compile(&program, None, &object, opt_level, false);

        let library_o = format!("{}/runtime/rt.o", get_eggcc_root());

        let executable = self
            .output_path
            .clone()
            .unwrap_or_else(|| format!("/tmp/{}", unique_name));

        let _ = std::fs::write(
            executable.clone() + "-args",
            self.prog_with_args.args.join(" "),
        );
        let mut cmd = std::process::Command::new("cc");
        cmd.arg(object.clone())
            .arg(library_o.clone())
            .arg("-o")
            .arg(executable.clone());

        #[cfg(target_os = "macos")]
        {
            // Workaround on new macos linkers:
            //
            // On linkers shipped past XCode 15, we see a bug around symbol
            // relocations with an error along the lines of:
            // ld: Assertion failed: (pattern[0].addrMode == addr_other), function addFixupFromRelocations, file Relocations.cpp, line 701.
            //
            // This is either a bug, or a difference in the way symbols are
            // handled, or a bit of both (chatter online differs), but for now,
            // we just retry with the ld_classic flag.
            if !cmd
                .stderr(Stdio::null())
                .status()
                .map(|x| x.success())
                .unwrap_or(false)
            {
                // reset stderr to surface other errors.
                expect_command_success(
                    cmd.stderr(Stdio::inherit()).arg("-Wl,-ld_classic"),
                    "failed to compile brilift with ld_classic flag",
                );
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            expect_command_success(&mut cmd, "failed to compile brilift");
        }

        Ok(Interpretable::Executable { executable })
    }

    fn run_bril_llvm(
        &self,
        input_prog: Program,
        optimize_egglog: bool,
        optimize_llvm: bool,
    ) -> Result<Interpretable, EggCCError> {
        // Make a unique name for this test running bril llvm
        // so we don't have conflicts in /tmp
        let unique_name = format!("{}_{}_{}", self.name(), optimize_egglog, optimize_llvm);

        let program = if optimize_egglog {
            Run::optimize_bril(&input_prog)?
        } else {
            input_prog
        };

        let mut buf = Vec::new();
        serde_json::to_writer_pretty(&mut buf, &program).expect("failed to deserialize");
        let dir = tempdir().expect("couldn't create temp dir");

        let llvm_ir = brillvm::cli::run(&brillvm::cli::Cli {
            file: None,
            runtime: Some(format!("{}/runtime/rt.bc", get_eggcc_root())),
            args: vec![],
            program: Some(String::from_utf8(buf).unwrap()),
            interpreter: false,
        });

        let init_ll_name = format!("{}-init.ll", self.name());
        let file_path = dir.path().join(init_ll_name.clone());
        let mut file = File::create(file_path.clone()).expect("couldn't create temp file");
        file.write_all(llvm_ir.as_bytes())
            .expect("unable to write to temp file");

        let executable = self
            .output_path
            .clone()
            .unwrap_or_else(|| format!("/tmp/{}", unique_name));

        // Copy init file to $output_dir
        if let Some(output_dir) = &self.llvm_output_dir {
            std::fs::create_dir_all(output_dir)
                .unwrap_or_else(|_| panic!("could not create output dir {}", output_dir));
            std::process::Command::new("cp")
                .arg(file_path.clone())
                .arg(output_dir)
                .status()
                .unwrap();
        }
        if optimize_llvm {
            expect_command_success(
                Command::new("clang-18")
                    .arg(file_path.clone())
                    .arg("-O3")
                    .arg("-fno-vectorize")
                    .arg("-fno-slp-vectorize")
                    .arg("-o")
                    .arg(executable.clone()),
                "failed to compile llvm ir",
            );

            if let Some(output_dir) = &self.llvm_output_dir {
                // Emit optimized LLVM IR
                expect_command_success(
                    Command::new("clang-18")
                        .current_dir(output_dir)
                        .arg(file_path.clone())
                        .arg("-O3")
                        .arg("-fno-vectorize")
                        .arg("-fno-slp-vectorize")
                        .arg("-emit-llvm")
                        .arg("-S")
                        .arg("-o")
                        .arg(format!("{}.ll", self.name())),
                    "failed to compile llvm ir and emit llvm ir",
                );
            }
        } else {
            let processed = dir.path().join("postprocessed.ll");
            // HACK: check if opt-18 exists
            // otherwise use opt
            // On Linux, sometimes it's called opt-18, while on mac it seems to be just opt
            // Also, on some machines, just running `opt-18` hangs, so we pass the version flag
            let opt_cmd = if Command::new("opt-18").arg("--version").status().is_ok() {
                "opt-18"
            } else {
                "opt"
            };

            let res = Command::new(opt_cmd)
                .arg("-passes=sroa")
                .arg("-S")
                .arg(file_path.clone())
                .arg("-o")
                .arg(processed.clone())
                .status()
                .unwrap();
            if !res.success() {
                let p1_string = std::fs::read_to_string(file_path.clone()).unwrap();
                panic!("Opt failed on following input:\n{p1_string}");
            }
            expect_command_success(
                Command::new("clang-18")
                    .arg(processed.clone())
                    .arg("-O0")
                    .arg("-o")
                    .arg(executable.clone()),
                "failed to compile llvm ir",
            );
            if let Some(output_dir) = &self.llvm_output_dir {
                expect_command_success(
                    Command::new("clang-18")
                        .current_dir(output_dir)
                        .arg(processed)
                        .arg("-O0")
                        .arg("-emit-llvm")
                        .arg("-S")
                        .arg("-o")
                        .arg(format!("{}.ll", self.name())),
                    "failed to copy unoptimized llvm ir",
                );
            }
        }
        let _ = std::fs::write(
            executable.clone() + "-args",
            self.prog_with_args.args.join(" "),
        );

        Ok(Interpretable::Executable { executable })
    }
}

fn expect_command_success(cmd: &mut std::process::Command, message: &str) {
    let status = cmd.status().unwrap();
    if !status.success() {
        panic!("Command failed: {}", message);
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
