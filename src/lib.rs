use bril2json::parse_abstract_program_from_read;
use bril_rs::AbstractProgram;
use bril_rs::Program;
use cfg::program_to_structured;

use egglog::ast::Expr;
use egglog::EGraph;
use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;

use thiserror::Error;

mod cfg;
mod conversions;
mod util;
use cfg::structured::StructuredProgram;

#[derive(Debug, Error)]
pub enum EggCCError {
    #[error("Egglog error: {0}")]
    EggLog(egglog::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Conversion error: {0}")]
    ConversionError(String),
    #[error("Unstructed control flow detected")]
    UnstructuredControlFlow,
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
        let ssa_prog: AbstractProgram = serde_json::from_str(&ssa_output).unwrap();

        Program::try_from(ssa_prog).map_err(|err| EggCCError::ConversionError(err.to_string()))
    }

    pub fn parse_to_structured(program: &str) -> Result<StructuredProgram, EggCCError> {
        let parsed = Self::parse_bril(program)?;
        Ok(program_to_structured(&parsed))
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
        let structured = program_to_structured(bril_program);

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

        let mut result = vec![];
        for name in keys {
            let expr = egg_fns.get(name).unwrap();
            let rep = egraph
                .extract_expr(expr.clone(), 0)
                .map_err(EggCCError::EggLog)?;
            let structured_func = self.expr_to_structured_func(rep.expr);

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
        (datatype Expr
          (Int String i64)
          (True String)
          (False String)
          (Char String String)
          (Float String f64)
          (Var String)
          (phi String Expr Expr) ;; both expressions should be variables
          (add String Expr Expr)
          (sub String Expr Expr)
          (mul String Expr Expr)
          (div String Expr Expr)
          (lt String Expr Expr))

        (datatype RetVal
          (ReturnValue String)
          (Void))

        (datatype Code
          (Assign String Expr)
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

        (datatype Function
          ;; name and body
          (Func String StructuredBlock))

        (rewrite (add ty (Int ty a) (Int ty b)) (Int ty (+ a b)))
        (rewrite (sub ty (Int ty a) (Int ty b)) (Int ty (- a b)))

        {program}
        {schedule}
        "
        )
    }
}
