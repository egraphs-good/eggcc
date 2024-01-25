use thiserror::Error;

pub(crate) mod body_contains;
pub(crate) mod conditional_invariant_code_motion;
pub(crate) mod deep_copy;
pub(crate) mod function_inlining;
pub(crate) mod id_analysis;
pub mod interpreter;
pub(crate) mod ir;
pub(crate) mod is_valid;
pub(crate) mod ivt;
pub(crate) mod purity_analysis;
pub(crate) mod simple;
pub(crate) mod subst;
pub(crate) mod switch_rewrites;
pub(crate) mod type_analysis;
pub(crate) mod util;

pub type Result = std::result::Result<(), Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Egglog(egglog::Error),
    #[error("{0}")]
    Parse(interpreter::ExprParseError),
    #[error("{0}")]
    Type(TypeError),
    #[error("test failed, extracted Exprs were not equal:\n{0:?}\n{1:?}")]
    Assert(
        (Value, interpreter::VirtualMachine),
        (Value, interpreter::VirtualMachine),
    ),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Parallel,
    Sequential,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(i64);

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    Boolean(bool),
    Unit,
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Get(Box<Expr>, usize),
    Print(Box<Expr>),
    Read(Box<Expr>),
    Write(Box<Expr>, Box<Expr>),
    All(Order, Vec<Expr>),
    Switch(Box<Expr>, Vec<Expr>),
    Loop(Id, Box<Expr>, Box<Expr>),
    Let(Id, Box<Expr>, Box<Expr>),
    Arg(Id),
    Call(Id, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(i64),
    Boolean(bool),
    Unit,
    Tuple(Vec<Value>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Num,
    Boolean,
    Unit,
    Tuple(Vec<Type>),
}

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("expected {0:?} to have type {1:?} but it had type {2:?}")]
    ExpectedType(Expr, Type, Type),
    #[error("expected {0:?} to be a tuple, but it had type {1:?}")]
    ExpectedTupleType(Expr, Type),
    #[error("no argument for {0:?}")]
    NoArg(Expr),
}

pub fn run_test(build: &str, check: &str) -> Result {
    let program = format!(
        "{}\n{build}\n{}\n{check}\n",
        [
            include_str!("schema.egg"),
            // analyses
            &is_valid::rules().join("\n"),
            &purity_analysis::purity_analysis_rules().join("\n"),
            &body_contains::rules().join("\n"),
            &subst::subst_rules().join("\n"),
            &deep_copy::deep_copy_rules().join("\n"),
            include_str!("sugar.egg"),
            include_str!("util.egg"),
            &id_analysis::id_analysis_rules().join("\n"),
            // optimizations
            include_str!("simple.egg"),
            include_str!("function_inlining.egg"),
            include_str!("interval_analysis.egg"),
            include_str!("ivt.egg"),
            &switch_rewrites::rules(),
            include_str!("type_analysis.egg"),
            &conditional_invariant_code_motion::rules().join("\n"),
        ]
        .join("\n"),
        include_str!("schedule.egg"),
    );

    let lines = egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map_err(Error::Egglog)?;

    let mut results = Vec::new();
    for line in lines {
        let mut vm = interpreter::VirtualMachine::new();
        let expr = line.parse::<Expr>().map_err(Error::Parse)?;
        interpreter::typecheck(&expr, &None).map_err(Error::Type)?;
        let value = interpreter::interpret(&expr, &None, &mut vm);
        results.push((value, vm));
    }

    if results.len() >= 2 {
        for result in &results[1..] {
            if result != &results[0] {
                return Err(Error::Assert(result.clone(), results[0].clone()));
            }
        }
    }

    Ok(())
}
