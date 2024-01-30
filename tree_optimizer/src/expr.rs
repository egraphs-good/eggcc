use bril_rs::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Parallel,
    Sequential,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Id {
    Unique(i64),
    Shared,
}

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
    /// A pred and a list of branches
    Switch(Box<Expr>, Vec<Expr>),
    /// Should only be a child of `Switch`
    /// Represents a single branch of a switch, giving
    /// it a unique id
    Branch(Id, Box<Expr>),
    Loop(Id, Box<Expr>, Box<Expr>),
    Let(Id, Box<Expr>, Box<Expr>),
    Arg(Id),
    Function(Id, String, TreeType, TreeType, Box<Expr>),
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
            Expr::Get(a, _) | Expr::Function(_, _, _, _, a) | Expr::Call(_, a) => {
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
            Expr::Branch(_id, child) => {
                func(child);
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

#[derive(Clone, PartialEq, Debug)]
pub enum TreeType {
    Unit,
    Bril(Type),
    Tuple(Vec<TreeType>),
}

pub enum TypeError {
    ExpectedType(Expr, TreeType, TreeType),
    ExpectedTupleType(Expr, TreeType),
    ExpectedLoopOutputType(Expr, TreeType),
    NoArg(Expr),
}
