use bril2json::parse_abstract_program_from_read;
use bril_rs::Program;

use cfg::structured::StructuredProgram;
use cfg::to_structured::cfg_to_structured;
use cfg::{program_to_cfg, CfgProgram};
use egglog::ast::Expr;
use egglog::EGraph;
use rvsdg::{RvsdgError, RvsdgProgram};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;

use thiserror::Error;

pub(crate) mod cfg;
mod conversions;
pub(crate) mod peg;
pub(crate) mod rvsdg;
pub mod util;

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

#[allow(dead_code)]
fn run_command_with_stdin(command: &mut std::process::Command, input: String) -> String {
    let mut piped = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = piped.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    });

    std::str::from_utf8(
        &piped
            .wait_with_output()
            .expect("Failed to read stdout")
            .stdout,
    )
    .unwrap()
    .to_string()
}
#[derive(Clone, Copy)]
pub enum RunType {
    StructuredConversion,
    RvsdgConversion,
    NaiiveOptimization,
}

impl Display for RunType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RunType::StructuredConversion => write!(f, "structured"),
            RunType::RvsdgConversion => write!(f, "rvsdg"),
            RunType::NaiiveOptimization => write!(f, "naiive"),
        }
    }
}

impl RunType {
    pub fn produces_bril(&self) -> bool {
        match self {
            RunType::StructuredConversion => false,
            RunType::RvsdgConversion => false,
            RunType::NaiiveOptimization => true,
        }
    }
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
    pub fn interp(program: &str, args: Vec<String>, profile_out: Option<PathBuf>) -> String {
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

    pub fn parse_and_optimize(&mut self, program: &str) -> Result<Program, EggCCError> {
        let parsed = Self::parse_bril(program)?;
        let res = self.optimize(&parsed)?;
        Ok(res)
    }

    pub fn parse_bril(program: &str) -> Result<Program, EggCCError> {
        let abstract_prog =
            parse_abstract_program_from_read(program.as_bytes(), false, false, None);

        // TODO dumb encoding does not support phi nodes yet
        /*
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

        // HACK: Check for uninitialized variables by looking for `__undefined`
        // variables in the program.
        Optimizer::check_for_uninitialized_vars(&prog)?;

        Ok(prog)
    }

    pub fn program_to_cfg(program: &Program) -> CfgProgram {
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

    pub fn fresh_var(&mut self) -> String {
        let res = format!("v{}_", self.var_counter);
        self.var_counter += 1;
        res
    }

    pub fn with_num_iters(mut self, num_iters: usize) -> Self {
        self.num_iters = num_iters;
        self
    }

    pub fn structured_to_optimizer(&mut self, structured: &StructuredProgram) -> String {
        let egg_fns: HashMap<String, Expr> = structured
            .functions
            .iter()
            .map(|f| (f.name.clone(), self.func_to_expr(f)))
            .collect();
        let egg_str = egg_fns
            .values()
            .map(Optimizer::pretty_print_expr)
            .collect::<Vec<String>>()
            .join("\n");

        self.make_optimizer_for(&egg_str)
    }

    pub fn optimized_structured(
        &mut self,
        bril_program: &Program,
    ) -> Result<StructuredProgram, EggCCError> {
        let structured = Self::program_to_structured(bril_program)?;

        let egglog_code = self.structured_to_optimizer(&structured);

        let mut egraph = EGraph::default();
        egraph
            .parse_and_run_program(&egglog_code)
            .map_err(EggCCError::EggLog)?
            .into_iter()
            .for_each(|output| log::info!("{}", output));

        let egg_fns: HashMap<String, Expr> = structured
            .functions
            .iter()
            .map(|f| (f.name.clone(), self.func_to_expr(f)))
            .collect();

        let mut keys = egg_fns.keys().collect::<Vec<&String>>();
        keys.sort();

        let mut termdag = Default::default();
        let mut result = vec![];
        for name in keys {
            let expr = egg_fns.get(name).unwrap();
            let (sort, value) = egraph
                .eval_expr(expr, None, true)
                .map_err(EggCCError::EggLog)?;
            let (_cost, term) = egraph.extract(value, &mut termdag, &sort);
            let structured_func = self.term_to_structured_func(&termdag, &term);

            result.push(structured_func);
        }
        Ok(StructuredProgram { functions: result })
    }

    pub fn optimize(&mut self, bril_program: &Program) -> Result<Program, EggCCError> {
        Ok(self.optimized_structured(bril_program)?.to_program())
    }

    pub fn make_optimizer_for(&mut self, program: &str) -> String {
        //let schedule = "(run 3)";
        let schedule = format!("(run {})", self.num_iters);
        format!(
            "
        (datatype Type
          (IntT)
          (BoolT)
          (FloatT)
          (CharT)
          (PointerT Type))

        (datatype Expr
          (Int Type i64)
          (True Type)
          (False Type)
          (Char Type String)
          (Float Type f64)
          (Var String)
          ;; two arguments and two labels
          (phi Type Expr Expr String String)
          (add Type Expr Expr)
          (sub Type Expr Expr)
          (mul Type Expr Expr)
          (div Type Expr Expr)
          (lt Type Expr Expr)
          (ptradd Type Expr Expr)
          (load Type Expr)

        )

        (datatype RetVal
          (ReturnValue String)
          (Void))

        (datatype Code
          (Assign String Expr)
          (store Expr Expr)
          (free Expr)
          (alloc Type String Expr)
          (Print Expr))

        (datatype CodeList
          (CodeCons Code CodeList)
          (CodeNil))

        (datatype BasicBlock
          (BlockNamed String CodeList))

        (datatype StructuredBlock
            (Block StructuredBlock)
            (Basic BasicBlock)
            (Ite String StructuredBlock StructuredBlock)
            (Loop StructuredBlock)
            (Sequence StructuredBlock StructuredBlock)
            (Break i64)
            (Return RetVal))

        (datatype Argument
            (Arg String Type))


        (datatype ArgList
            (ArgCons Argument ArgList)
            (ArgNil))

        (datatype Function
          ;; name, arguments, and body
          (Func String ArgList StructuredBlock))

        (rewrite (add ty (Int ty a) (Int ty b)) (Int ty (+ a b)))
        (rewrite (sub ty (Int ty a) (Int ty b)) (Int ty (- a b)))

        {program}
        {schedule}
        "
        )
    }
}
