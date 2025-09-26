use bril2json::parse_abstract_program_from_read;
use bril_rs::Program;

use cfg::{program_to_cfg, SimpleCfgProgram};
use conversions::check_for_uninitialized_vars;
use dag_in_context::interpreter::{interpret_dag_prog, Value};
use dag_in_context::schema::Constant;
use ordered_float::OrderedFloat;
use rvsdg::{RvsdgError, RvsdgProgram};
use std::path::PathBuf;

use util::Interpretable;

use thiserror::Error;

pub mod canonicalize_names;
pub(crate) mod cfg;
mod conversions;
pub(crate) mod rvsdg;
pub mod util;

#[cfg(test)]
pub(crate) mod test_util;

#[derive(Debug, Error)]
pub enum EggCCError {
    #[error("Egglog error: {0}")]
    EggLog(egglog::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Unstructured control flow detected")]
    UnstructuredControlFlow,
    #[error("Rvsdg error: {0}")]
    RvsdgError(RvsdgError),
    #[error("Uninitialized variable {0} used in function {1}")]
    UninitializedVariable(String, String),
}

pub struct Optimizer {
    pub num_iters: usize,
    pub var_counter: usize,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self {
            num_iters: 3,
            var_counter: 0,
        }
    }
}

impl Optimizer {
    pub fn parse_bril_args(program: &str) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(first_line) = program.split('\n').next() {
            if first_line.contains("# ARGS:") {
                for arg in first_line["# ARGS: ".len()..]
                    .split(' ')
                    .map(|s| s.to_string())
                {
                    args.push(arg);
                }
            } else if first_line.contains("// ARGS:") {
                for arg in first_line["// ARGS: ".len()..]
                    .split(' ')
                    .map(|s| s.to_string())
                {
                    args.push(arg);
                }
            }
        }
        args
    }

    /// Produces a vector of values, one per string argument to the program
    fn parse_arguments(args: Vec<String>) -> Vec<Value> {
        args.into_iter()
            .map(|arg| {
                if let Ok(int) = arg.parse::<i64>() {
                    Value::Const(Constant::Int(int))
                } else if let Ok(f) = arg.parse::<OrderedFloat<f64>>() {
                    Value::Const(Constant::Float(f))
                } else if arg == "true" {
                    Value::Const(Constant::Bool(true))
                } else if arg == "false" {
                    Value::Const(Constant::Bool(false))
                } else {
                    panic!("Invalid argument to bril program: {}", arg);
                }
            })
            .collect()
    }

    /// Interpret a program in an `Interpretable` IR.
    /// Returns the printed output of the program and optionally the cycles taken to run the program.
    /// The program should not return a value.
    pub fn interp(
        program: &Interpretable,
        args: Vec<String>,
        profile_out: Option<PathBuf>,
    ) -> (String, Option<u64>) {
        match program {
            Interpretable::Bril(program) => (Self::interp_bril(program, args, profile_out), None),
            Interpretable::TreeProgram(program) => {
                let mut parsed = Self::parse_arguments(args);
                // add the state value to the end
                parsed.push(Value::StateV);
                let (val, mut printed) = interpret_dag_prog(program, &Value::Tuple(parsed));
                assert_eq!(val, Value::Tuple(vec![Value::StateV]));
                // add new line to the end of each line in printed
                for line in printed.iter_mut() {
                    line.push('\n');
                }
                (printed.join(""), None)
            }
            Interpretable::CycleMeasuringExecutable { executable } => {
                let output = std::process::Command::new(
                    std::path::Path::new(executable).canonicalize().unwrap(),
                )
                .args(args)
                .output()
                .unwrap();
                let output_str = String::from_utf8(output.stdout).unwrap();
                let output_err = String::from_utf8(output.stderr).unwrap();
                let error_code = output.status.code().unwrap();
                if error_code != 0 {
                    panic!("Error code: {}", error_code);
                }
                (output_str, Some(output_err.trim().parse().unwrap()))
            }
            Interpretable::Executable { executable } => {
                let output = std::process::Command::new(
                    std::path::Path::new(executable).canonicalize().unwrap(),
                )
                .args(args)
                .output()
                .unwrap()
                .stdout;

                (String::from_utf8(output).unwrap(), None)
            }
        }
    }

    /// run the rust interpreter on the program
    /// without any optimizations
    pub fn interp_bril(
        program: &Program,
        args: Vec<String>,
        profile_out: Option<PathBuf>,
    ) -> String {
        let mut program_out = Vec::new();

        match profile_out {
            Some(path) => {
                let profile_file = std::fs::File::create(path).unwrap();

                brilirs::run_input(
                    std::io::BufReader::new(program.to_string().as_bytes()),
                    std::io::BufWriter::new(&mut program_out),
                    &args,
                    true,
                    profile_file,
                    false,
                    true,
                    None,
                )
                .expect("brili interp error")
            }
            None => {
                brilirs::run_input(
                    std::io::BufReader::new(program.to_string().as_bytes()),
                    std::io::BufWriter::new(&mut program_out),
                    &args,
                    false,
                    std::io::stderr(),
                    false,
                    true,
                    None,
                )
                .unwrap();
            }
        }

        String::from_utf8(program_out).unwrap()
    }

    pub fn parse_bril(program: &str) -> Result<Program, EggCCError> {
        let abstract_prog =
            parse_abstract_program_from_read(program.as_bytes(), false, false, None);

        /*
        Commented out code converts to SSA format, which
        we are currently not using anywhere.

        let serialized = serde_json::to_string(&abstract_prog).unwrap();
        let ssa_output = run_command_with_stdin(
            std::process::Command::new("python3").arg("bril/examples/to_ssa.py"),
            serialized,
        );
        let ssa_prog: AbstractProgram = serde_json::from_str(&ssa_output).unwrap();

        let ssa_res = Program::try_from(ssa_prog)
            .map_err(|err| EggCCError::ConversionError(err.to_string()))?;

        let dead_code_optimized = run_command_with_stdin(
            std::process::Command::new("python3").arg("bril/examples/tdce.py"),
            serde_json::to_string(&ssa_res).unwrap(),
        );

        let res: Program = serde_json::from_str(&dead_code_optimized)
            .map_err(|err| EggCCError::ConversionError(err.to_string()))?;
        */

        let prog = Program::try_from(abstract_prog)
            .map_err(|err| EggCCError::ConversionError(err.to_string()))?;

        check_for_uninitialized_vars(&prog)?;

        Ok(prog)
    }

    pub fn program_to_cfg(program: &Program) -> SimpleCfgProgram {
        program_to_cfg(program)
    }

    pub fn program_to_rvsdg(program: &Program) -> Result<RvsdgProgram, EggCCError> {
        eprintln!("Converting program to Rvsdg...");
        let cfg = Self::program_to_cfg(program);
        rvsdg::cfg_to_rvsdg(&cfg)
    }

    pub fn fresh_var(&mut self) -> String {
        let res = format!("v{}_", self.var_counter);
        self.var_counter += 1;
        res
    }

    pub fn with_num_iters(mut self, num_iters: usize) -> Self {
        self.num_iters = num_iters;
        self
    }
}
