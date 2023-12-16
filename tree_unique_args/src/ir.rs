pub(crate) enum Sort {
    Expr,
    ListExpr,
    Order,
    I64,
    Bool,
}

impl Sort {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Sort::Expr => "Expr",
            Sort::ListExpr => "ListExpr",
            Sort::Order => "Order",
            Sort::I64 => "i64",
            Sort::Bool => "bool",
        }
    }
}

pub(crate) enum Constructor {
    Num,
    Boolean,
    UnitExpr,
    Add,
    Sub,
    Mul,
    LessThan,
    And,
    Or,
    Not,
    Get,
    Print,
    Read,
    Write,
    All,
    Switch,
    Loop,
    Body,
    Arg,
    Call,
    Cons,
    Nil,
}

impl Constructor {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Constructor::Num => "Num",
            Constructor::Boolean => "Boolean",
            Constructor::UnitExpr => "UnitExpr",
            Constructor::Add => "Add",
            Constructor::Sub => "Sub",
            Constructor::Mul => "Mul",
            Constructor::LessThan => "LessThan",
            Constructor::And => "And",
            Constructor::Or => "Or",
            Constructor::Not => "Not",
            Constructor::Get => "Get",
            Constructor::Print => "Print",
            Constructor::Read => "Read",
            Constructor::Write => "Write",
            Constructor::All => "All",
            Constructor::Switch => "Switch",
            Constructor::Loop => "Loop",
            Constructor::Body => "Body",
            Constructor::Arg => "Arg",
            Constructor::Call => "Call",
            Constructor::Cons => "Cons",
            Constructor::Nil => "Nil",
        }
    }

    pub(crate) fn param_sorts(&self) -> Vec<Sort> {
        match self {
            Constructor::Num => vec![Sort::I64],
            Constructor::Boolean => vec![Sort::Bool],
            Constructor::UnitExpr => vec![],
            Constructor::Add => vec![Sort::Expr, Sort::Expr],
            Constructor::Sub => vec![Sort::Expr, Sort::Expr],
            Constructor::Mul => vec![Sort::Expr, Sort::Expr],
            Constructor::LessThan => vec![Sort::Expr, Sort::Expr],
            Constructor::And => vec![Sort::Expr, Sort::Expr],
            Constructor::Or => vec![Sort::Expr, Sort::Expr],
            Constructor::Not => vec![Sort::Expr],
            Constructor::Get => vec![Sort::Expr, Sort::I64],
            Constructor::Print => vec![Sort::Expr],
            Constructor::Read => vec![Sort::Expr],
            Constructor::Write => vec![Sort::Expr, Sort::Expr],
            Constructor::All => vec![Sort::Order, Sort::ListExpr],
            Constructor::Switch => vec![Sort::Expr, Sort::ListExpr],
            Constructor::Loop => vec![Sort::I64, Sort::Expr, Sort::Expr],
            Constructor::Body => vec![Sort::I64, Sort::Expr, Sort::Expr],
            Constructor::Arg => vec![Sort::I64],
            Constructor::Call => vec![Sort::I64, Sort::Expr],
            Constructor::Cons => vec![Sort::Expr, Sort::ListExpr],
            Constructor::Nil => vec![],
        }
    }

    pub(crate) fn sort(&self) -> Sort {
        match self {
            Constructor::Num => Sort::Expr,
            Constructor::Boolean => Sort::Expr,
            Constructor::UnitExpr => Sort::Expr,
            Constructor::Add => Sort::Expr,
            Constructor::Sub => Sort::Expr,
            Constructor::Mul => Sort::Expr,
            Constructor::LessThan => Sort::Expr,
            Constructor::And => Sort::Expr,
            Constructor::Or => Sort::Expr,
            Constructor::Not => Sort::Expr,
            Constructor::Get => Sort::Expr,
            Constructor::Print => Sort::Expr,
            Constructor::Read => Sort::Expr,
            Constructor::Write => Sort::Expr,
            Constructor::All => Sort::Expr,
            Constructor::Switch => Sort::Expr,
            Constructor::Loop => Sort::Expr,
            Constructor::Body => Sort::Expr,
            Constructor::Arg => Sort::Expr,
            Constructor::Call => Sort::Expr,
            Constructor::Cons => Sort::ListExpr,
            Constructor::Nil => Sort::ListExpr,
        }
    }

    pub(crate) fn num_params(&self) -> usize {
        self.param_sorts().len()
    }
}

pub(crate) const CONSTRUCTORS: [Constructor; 22] = [
    Constructor::Num,
    Constructor::Boolean,
    Constructor::UnitExpr,
    Constructor::Add,
    Constructor::Sub,
    Constructor::Mul,
    Constructor::LessThan,
    Constructor::And,
    Constructor::Or,
    Constructor::Not,
    Constructor::Get,
    Constructor::Print,
    Constructor::Read,
    Constructor::Write,
    Constructor::All,
    Constructor::Switch,
    Constructor::Loop,
    Constructor::Body,
    Constructor::Arg,
    Constructor::Call,
    Constructor::Cons,
    Constructor::Nil,
];
