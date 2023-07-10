use std::io::{stdin, Read};

use egglog::{ast::parse::ExprParser, ast::Expr, EGraph};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EggCCError {
    #[error("Egglog error: {0}")]
    EggLog(egglog::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

struct Optimizer {}

impl Optimizer {
    fn optimize(&self, program: &str) -> Result<Expr, EggCCError> {
        let expr_parser = ExprParser::new();
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
        Ok(extract_report.expr)
    }

    fn make_optimizer_for(&self, program: &str) -> String {
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
    }
}

fn main() {
    let mut input = String::new();
    stdin().lock().read_to_string(&mut input).unwrap();
    let optimizer = Optimizer {};
    match optimizer.optimize(&input) {
        Ok(expr) => println!("{}", expr),
        Err(err) => println!("{}", err),
    }
}
