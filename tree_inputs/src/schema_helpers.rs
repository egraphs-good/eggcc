use std::fmt::{Display, Formatter};

use crate::schema::{Constant, Expr, RcExpr, TreeProgram};

/// Display for Constant implements a
/// rust-readable representation using
/// the sugar in `ast.rs`.
impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Constant::Int(i) => write!(f, "{}", i),
            Constant::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl Expr {
    pub fn func_name(&self) -> Option<String> {
        match self {
            Expr::Function(name, _, _, _) => Some(name.clone()),
            _ => None,
        }
    }

    pub fn func_body(&self) -> Option<&RcExpr> {
        match self {
            Expr::Function(_, _, _, body) => Some(body),
            _ => None,
        }
    }
}

impl TreeProgram {
    pub fn get_function(&self, name: &str) -> Option<&RcExpr> {
        if self.entry.func_name() == Some(name.to_string()) {
            return Some(&self.entry);
        }
        self.functions
            .iter()
            .find(|expr| expr.func_name() == Some(name.to_string()))
    }
}
