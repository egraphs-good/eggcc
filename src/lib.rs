use bril2json::parse_abstract_program_from_read;
use bril_rs::Program;
//use egglog::{ast::parse::ExprParser, ast::Expr, EGraph};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EggCCError {
    #[error("Egglog error: {0}")]
    EggLog(egglog::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

pub struct Optimizer {}

impl Optimizer {
    pub fn optimize(&self, program: &str) -> Result<Program, EggCCError> {
        let bril_program = Program::try_from(parse_abstract_program_from_read(
            program.as_bytes(),
            false,
            false,
            None,
        ))
        .map_err(|err| EggCCError::ConversionError(err.to_string()))?;

        /*let expr_parser = ExprParser::new();
        let egglog_code = self.make_optimizer_for(program);
        let mut egraph = EGraph::default();
        egraph
            .parse_and_run_program(&egglog_code)
            .map_err(EggCCError::EggLog)?
            .into_iter()
            .for_each(|output| log::info!("{}", output));
        let extract_report = egraph
            .extract_expr(
                expr_parser
                    .parse(program)
                    .map_err(|err| EggCCError::Parse(err.to_string()))?,
                0,
            )
            .map_err(EggCCError::EggLog)?;
        Ok(extract_report.expr)*/

        Ok(bril_program)
    }

    /*fn make_optimizer_for(&self, program: &str) -> String {
        format!(
            "
        (datatype Imp
        (Int i64)
        (True)
        (False)
        (Add Imp Imp)
        (Sub Imp Imp)
        (Mul Imp Imp)
        (Div Imp Imp))
        {program}"
        )
    }*/
}
