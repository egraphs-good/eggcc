use std::str::FromStr;

use bril_rs::Type;
use strum_macros::{Display, EnumIter};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Order {
    Parallel,
    #[default]
    Sequential,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Parallel => write!(f, "(Parallel)"),
            Order::Sequential => write!(f, "(Sequential)"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Id {
    Unique(i64),
    #[default]
    Shared,
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Unique(id) => write!(f, "(Id {})", id),
            Id::Shared => write!(f, "(Shared)"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, Default, Display)]
pub enum PureBOp {
    #[default]
    Add,
    Sub,
    Mul,
    LessThan,
    And,
    Or,
}

impl FromStr for PureBOp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Add" => Ok(PureBOp::Add),
            "Sub" => Ok(PureBOp::Sub),
            "Mul" => Ok(PureBOp::Mul),
            "LessThan" => Ok(PureBOp::LessThan),
            "And" => Ok(PureBOp::And),
            "Or" => Ok(PureBOp::Or),
            _ => Err(()),
        }
    }
}

impl PureBOp {
    pub fn input_types(&self) -> (Type, Type) {
        match self {
            PureBOp::Add | PureBOp::Sub | PureBOp::Mul | PureBOp::LessThan => {
                (Type::Int, Type::Int)
            }
            PureBOp::And | PureBOp::Or => (Type::Bool, Type::Bool),
        }
    }

    pub fn output_type(&self) -> Type {
        match self {
            PureBOp::Add | PureBOp::Sub | PureBOp::Mul => Type::Int,
            PureBOp::LessThan | PureBOp::And | PureBOp::Or => Type::Bool,
        }
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter, Default, Display)]
pub enum PureUOp {
    #[default]
    Not,
}

impl FromStr for PureUOp {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Not" => Ok(PureUOp::Not),
            _ => Err(()),
        }
    }
}

impl PureUOp {
    pub fn input_type(&self) -> Type {
        match self {
            PureUOp::Not => Type::Bool,
        }
    }

    pub fn output_type(&self) -> Type {
        match self {
            PureUOp::Not => Type::Bool,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Sort {
    Expr,
    ListExpr,
    Order,
    BinPureOp,
    UnaryPureOp,
    IdSort,
    I64,
    Bool,
    Type,
    String,
}

impl Sort {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Sort::Expr => "Expr",
            Sort::ListExpr => "ListExpr",
            Sort::Order => "Order",
            Sort::IdSort => "IdSort",
            Sort::I64 => "i64",
            Sort::String => "String",
            Sort::Bool => "bool",
            Sort::Type => "Type",
            Sort::BinPureOp => "BinPureOp",
            Sort::UnaryPureOp => "UnaryPureOp",
        }
    }
}

// Subset of sorts that refer to expressions
#[derive(Debug, EnumIter, PartialEq)]
pub(crate) enum ESort {
    Expr,
    ListExpr,
}

impl std::fmt::Display for ESort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ESort {
    pub(crate) fn to_sort(&self) -> Sort {
        match self {
            ESort::Expr => Sort::Expr,
            ESort::ListExpr => Sort::ListExpr,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.to_sort().name()
    }
}

#[derive(Clone, Debug, PartialEq, EnumIter)]
pub enum Expr {
    Num(Id, i64),
    Boolean(Id, bool),
    BOp(PureBOp, Box<Expr>, Box<Expr>),
    UOp(PureUOp, Box<Expr>),
    Get(Box<Expr>, usize),
    Print(Box<Expr>),
    Read(Box<Expr>, TreeType),
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
    /// referencing id, function name, and argument
    Call(Id, String, Box<Expr>),
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Num(Id::Shared, 0)
    }
}

impl Expr {
    pub fn is_pure(&self) -> bool {
        use Expr::*;
        match self {
            Num(..) | Boolean(..) | Arg(..) | BOp(..) | UOp(..) | Get(..) | Read(..) | All(..)
            | Switch(..) | Branch(..) | Loop(..) | Let(..) | Function(..) | Program(..)
            | Call(..) => true,
            Print(..) | Write(..) => false,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Expr::Num(..) => "Num",
            Expr::Boolean(..) => "Boolean",
            Expr::BOp(_, _, _) => "BOp",
            Expr::UOp(_, _) => "UOp",
            Expr::Get(_, _) => "Get",
            Expr::Print(_) => "Print",
            Expr::Read(..) => "Read",
            Expr::Write(_, _) => "Write",
            Expr::All(_, _, _) => "All",
            Expr::Switch(_, _) => "Switch",
            Expr::Branch(_, _) => "Branch",
            Expr::Loop(_, _, _) => "Loop",
            Expr::Let(_, _, _) => "Let",
            Expr::Arg(_) => "Arg",
            Expr::Function(_, _, _, _, _) => "Function",
            Expr::Program(_) => "Program",
            Expr::Call(_, _, _) => "Call",
        }
    }

    /// Runs `func` on every child of this expression.
    pub fn for_each_child(&mut self, mut func: impl FnMut(&mut Expr)) {
        match self {
            Expr::Num(..) | Expr::Boolean(..) | Expr::Arg(..) => {}
            Expr::BOp(_, a, b) => {
                func(a);
                func(b);
            }
            Expr::UOp(_, a) => {
                func(a);
            }
            Expr::Write(a, b) => {
                func(a);
                func(b);
            }
            Expr::Print(a) | Expr::Read(a, _) => {
                func(a);
            }
            Expr::Get(a, _) | Expr::Function(_, _, _, _, a) | Expr::Call(_, _, a) => {
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
    Bril(Type),
    Tuple(Vec<TreeType>),
}

impl Default for TreeType {
    fn default() -> Self {
        TreeType::Tuple(vec![])
    }
}

pub enum TypeError {
    ExpectedType(Expr, TreeType, TreeType),
    ExpectedTupleType(Expr, TreeType),
    ExpectedLoopOutputType(Expr, TreeType),
    NoArg(Expr),
}
