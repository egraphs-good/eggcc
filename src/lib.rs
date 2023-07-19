use bril2json::parse_abstract_program_from_read;
use bril_rs::Program;
use egglog::EGraph;

use thiserror::Error;

mod conversions;
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
        Program::try_from(parse_abstract_program_from_read(
            program.as_bytes(),
            false,
            false,
            None,
        ))
        .map_err(|err| EggCCError::ConversionError(err.to_string()))
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
        assert!(bril_program.functions.len() == 1);
        assert!(bril_program.functions[0].name == "main");
        assert!(bril_program.imports.is_empty());

        let converted = self.func_to_expr(&bril_program.functions[0]);

        let egglog_code = self.make_optimizer_for(&converted.to_string());

        let mut egraph = EGraph::default();
        egraph
            .parse_and_run_program(&egglog_code)
            .map_err(EggCCError::EggLog)?
            .into_iter()
            .for_each(|output| log::info!("{}", output));
        let extract_report = egraph
            .extract_expr(converted, 0)
            .map_err(EggCCError::EggLog)?;

        let bril_func = self.expr_to_func(extract_report.expr);

        let output_program = Program {
            functions: vec![bril_func],
            imports: vec![],
        };

        Ok(output_program)
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
        
        (datatype FunctionBody
          (End)
          (Print Expr FunctionBody))

        (datatype Function
          ;; name and body
          (Func String FunctionBody))

        (rewrite (add ty (Int ty a) (Int ty b)) (Int ty (+ a b)))

        {program}
        {schedule}
        "
        )
    }
}
