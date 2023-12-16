#[allow(dead_code)]

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
    Add,
    Print,
    Loop,
    Cons,
    Nil,
}

impl Constructor {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Constructor::Num => "Num",
            Constructor::Add => "Add",
            Constructor::Print => "Print",
            Constructor::Loop => "Loop",
            Constructor::Cons => "Cons",
            Constructor::Nil => "Nil",
        }
    }

    pub(crate) fn param_sorts(&self) -> Vec<Sort> {
        match self {
            Constructor::Num => vec![Sort::I64],
            Constructor::Add => vec![Sort::Expr, Sort::Expr],
            Constructor::Print => vec![Sort::Expr],
            Constructor::Loop => vec![Sort::I64, Sort::Expr, Sort::Expr],
            Constructor::Cons => vec![Sort::Expr, Sort::ListExpr],
            Constructor::Nil => vec![],
        }
    }

    pub(crate) fn sort(&self) -> Sort {
        match self {
            Constructor::Num => Sort::Expr,
            Constructor::Add => Sort::Expr,
            Constructor::Print => Sort::Expr,
            Constructor::Loop => Sort::Expr,
            Constructor::Cons => Sort::ListExpr,
            Constructor::Nil => Sort::ListExpr,
        }
    }

    pub(crate) fn num_params(&self) -> usize {
        self.param_sorts().len()
    }
}

pub(crate) const CONSTRUCTORS: [Constructor; 6] = [
    Constructor::Num,
    Constructor::Add,
    Constructor::Print,
    Constructor::Loop,
    Constructor::Cons,
    Constructor::Nil,
];
