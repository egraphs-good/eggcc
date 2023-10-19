use bril2json::parse_abstract_program_from_read;
use bril_rs::Program;

use cfg::structured::StructuredProgram;
use cfg::to_structured::cfg_to_structured;
use cfg::{program_to_cfg, SimpleCfgProgram};
use conversions::check_for_uninitialized_vars;
use rvsdg::{RvsdgError, RvsdgProgram};
use std::path::PathBuf;

use thiserror::Error;

pub(crate) mod cfg;
mod conversions;
pub(crate) mod peg;
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
            }
        }
        args
    }

    /// run the rust interpreter on the program
    /// without any optimizations
    pub fn interp(program: &Program, args: Vec<String>, profile_out: Option<PathBuf>) -> String {
        let mut optimized_out = Vec::new();

        match profile_out {
            Some(path) => {
                let profile_file = std::fs::File::create(path).unwrap();

                brilirs::run_input(
                    std::io::BufReader::new(program.to_string().as_bytes()),
                    std::io::BufWriter::new(&mut optimized_out),
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
                    std::io::BufWriter::new(&mut optimized_out),
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

        String::from_utf8(optimized_out).unwrap()
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
        let cfg = Self::program_to_cfg(program);
        rvsdg::cfg_to_rvsdg(&cfg)
    }

    pub fn program_to_structured(program: &Program) -> Result<StructuredProgram, EggCCError> {
        let cfg = Self::program_to_cfg(program);
        cfg_to_structured(&cfg)
    }

    pub fn parse_to_structured(program: &str) -> Result<StructuredProgram, EggCCError> {
        let parsed = Self::parse_bril(program)?;
        Self::program_to_structured(&parsed)
    }

    pub fn rvsdg_optimize(program: &Program) -> Result<Program, EggCCError> {
        let rvsdg = Self::program_to_rvsdg(program)?;
        let optimized = rvsdg.optimize()?;
        let cfg = optimized.to_cfg();
        let program = cfg.to_bril();

        Ok(program)
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
