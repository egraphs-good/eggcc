pub(crate) mod body_contains;
pub(crate) mod conditional_invariant_code_motion;
pub(crate) mod deep_copy;
pub(crate) mod function_inlining;
pub(crate) mod id_analysis;
pub mod interpreter;
pub(crate) mod ir;
pub(crate) mod is_valid;
pub(crate) mod ivt;
pub(crate) mod loop_invariant;
pub(crate) mod purity_analysis;
pub(crate) mod simple;
pub(crate) mod subst;
pub(crate) mod switch_rewrites;
pub(crate) mod util;

pub type Result = std::result::Result<(), egglog::Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Parallel,
    Sequential,
}

#[derive(Clone, Debug, PartialEq)]
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
    Function(Id, Box<Expr>),
    Call(Id, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(i64),
    Boolean(bool),
    Unit,
    Tuple(Vec<Value>),
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Num,
    Boolean,
    Unit,
    Tuple(Vec<Type>),
}

pub enum TypeError {
    ExpectedType(Expr, Type, Type),
    ExpectedTupleType(Expr, Type),
    ExpectedLoopOutputType(Expr, Type),
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
            &conditional_invariant_code_motion::rules().join("\n"),
            &loop_invariant::rules().join("\n\n"),
        ]
        .join("\n"),
        include_str!("schedule.egg"),
    );

    println!("{}", program);

    egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        })
}
