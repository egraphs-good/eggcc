use bril2json::parse_abstract_program_from_read;
use bril_rs::AbstractProgram;
use bril_rs::Program;
use egglog::ast::Expr;
use egglog::EGraph;
use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;

use thiserror::Error;

pub(crate) mod cfg;
mod conversions;
pub(crate) mod rvsdg;
mod util;

#[derive(Debug, Error)]
pub enum EggCCError {
    #[error("Egglog error: {0}")]
    EggLog(egglog::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

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
    pub fn parse_and_optimize(&mut self, program: &str) -> Result<Program, EggCCError> {
        let parsed = Self::parse_bril(program)?;
        eprintln!("Parsed program: {}", parsed);
        let res = self.optimize(&parsed)?;
        Ok(res)
    }

    pub fn parse_bril(program: &str) -> Result<Program, EggCCError> {
        let abstract_prog =
            parse_abstract_program_from_read(program.as_bytes(), false, false, None);
        let serialized = serde_json::to_string(&abstract_prog).unwrap();

        // call SSA conversion
        let ssa_output = run_command_with_stdin(
            std::process::Command::new("python3").arg("bril/examples/to_ssa.py"),
            serialized,
        );
        eprintln!("ssa output: {}", ssa_output);
        let ssa_prog: AbstractProgram = serde_json::from_str(&ssa_output).unwrap();

        Program::try_from(ssa_prog).map_err(|err| EggCCError::ConversionError(err.to_string()))
    }

    pub fn fresh(&mut self) -> String {
        let res = format!("v{}_", self.var_counter);
        self.var_counter += 1;
        res
    }

    pub fn with_num_iters(mut self, num_iters: usize) -> Self {
        self.num_iters = num_iters;
        self
    }

    pub fn optimize(&mut self, bril_program: &Program) -> Result<Program, EggCCError> {
        assert!(!bril_program.functions.is_empty());
        assert!(bril_program.functions.iter().any(|f| { f.name == "main" }));
        assert!(bril_program.imports.is_empty());

        let egg_fns: HashMap<String, Expr> = bril_program
            .functions
            .iter()
            .map(|f| (f.name.clone(), self.func_to_expr(f)))
            .collect();

        let egg_str = egg_fns
            .values()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let egglog_code = self.make_optimizer_for(&egg_str);

        println!("{}", egglog_code);

        let mut egraph = EGraph::default();
        egraph
            .parse_and_run_program(&egglog_code)
            .map_err(EggCCError::EggLog)?
            .into_iter()
            .for_each(|output| log::info!("{}", output));

        // TODO: idk how rust works, so why do I have to clone??? @ryan-berger
        let mut fn_names = egg_fns.keys().cloned().collect::<Vec<String>>();

        // sort the function names for deterministic map iteration
        fn_names.sort();

        fn_names.iter().try_fold(
            Program {
                functions: vec![],
                imports: vec![],
            },
            |mut program, name| {
                let e = &egg_fns[name];
                let rep = egraph
                    .extract_expr(e.clone(), 0)
                    .map_err(EggCCError::EggLog)?;

                program.functions.push(self.expr_to_func(rep.expr));
                Ok(program)
            },
        )
    }

    fn make_optimizer_for(&mut self, program: &str) -> String {
        //let schedule = "(run 3)";
        let schedule = format!("(run {})", self.num_iters);
        format!(
            "
        (datatype Expr
          (Int String i64)
          (True String)
          (False String)
          (Char String String)
          (Float String f64)
          (add String Expr Expr)
          (sub String Expr Expr)
          (mul String Expr Expr)
          (div String Expr Expr))

        (datatype RetVal
            (ReturnValue Expr)
            (Void))

        (datatype FunctionBody
          (End)
          (Ret RetVal FunctionBody)
          (Call String Expr FunctionBody)
          (Print Expr FunctionBody))

        (datatype Function
          ;; name and body
          (Func String FunctionBody))

        (rewrite (add ty (Int ty a) (Int ty b)) (Int ty (+ a b)))
        (rewrite (sub ty (Int ty a) (Int ty b)) (Int ty (- a b)))

        {program}
        {schedule}
        "
        )
    }
}
