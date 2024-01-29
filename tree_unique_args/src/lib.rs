pub(crate) mod arg_used_analysis;
pub mod ast;
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
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Get(Box<Expr>, usize),
    /// Concat is a convenient built-in way
    /// to put two tuples together.
    /// It's not strictly necessary, but
    /// doing it by constructing a new big tuple is tedius and slow.
    Concat(Box<Expr>, Box<Expr>),
    Print(Box<Expr>),
    Read(Box<Expr>),
    Write(Box<Expr>, Box<Expr>),
    All(Id, Order, Vec<Expr>),
    Switch(Box<Expr>, Vec<Expr>),
    Loop(Id, Box<Expr>, Box<Expr>),
    Let(Id, Box<Expr>, Box<Expr>),
    Arg(Id),
    Function(Id, Box<Expr>),
    /// A list of functions, with the first
    /// being the main function.
    Program(Vec<Expr>),
    Call(Id, Box<Expr>),
}

impl Expr {
    /// Runs `func` on every child of this expression.
    pub fn for_each_child(&mut self, mut func: impl FnMut(&mut Expr)) {
        match self {
            Expr::Num(_) | Expr::Boolean(_) | Expr::Arg(_) => {}
            Expr::Add(a, b)
            | Expr::Sub(a, b)
            | Expr::Mul(a, b)
            | Expr::LessThan(a, b)
            | Expr::And(a, b)
            | Expr::Or(a, b)
            | Expr::Concat(a, b)
            | Expr::Write(a, b) => {
                func(a);
                func(b);
            }
            Expr::Not(a) | Expr::Print(a) | Expr::Read(a) => {
                func(a);
            }
            Expr::Get(a, _) | Expr::Function(_, a) | Expr::Call(_, a) => {
                func(a);
            }
            Expr::All(_, _, children) => {
                for child in children {
                    func(child);
                }
            }
            Expr::Switch(input, children) => {
                func(input);
                for child in children {
                    func(child);
                }
            }
            Expr::Loop(_, pred, output) | Expr::Let(_, pred, output) => {
                func(pred);
                func(output);
            }
            Expr::Program(functions) => {
                for function in functions {
                    func(function);
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(i64),
    Boolean(bool),
    Tuple(Vec<Value>),
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Num,
    Boolean,
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
            &arg_used_analysis::arg_used_analysis_rules().join("\n"),
            &subst::subst_rules().join("\n"),
            &deep_copy::deep_copy_rules().join("\n"),
            include_str!("sugar.egg"),
            &util::rules().join("\n"),
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

    println!("{}", program);

    egglog::EGraph::default()
        .parse_and_run_program(&program)
        .map(|lines| {
            for line in lines {
                println!("{}", line);
            }
        })
}
