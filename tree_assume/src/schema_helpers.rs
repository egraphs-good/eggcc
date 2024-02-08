use std::fmt::{Display, Formatter};

use crate::schema::{Constant, Expr, RcExpr, TreeProgram, Type};

/// Display for Constant implements a
/// rust-readable representation using
/// the sugar in `ast.rs`.
impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let (term, termdag) = self.to_egglog();
        write!(f, "{}", termdag.to_string(&term))
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

    pub fn to_program(self: &RcExpr, input_ty: Type, output_ty: Type) -> TreeProgram {
        match self.as_ref() {
            Expr::Function(..) => TreeProgram {
                entry: self.clone(),
                functions: vec![],
            },
            _ => TreeProgram {
                entry: RcExpr::new(Expr::Function(
                    "main".to_string(),
                    input_ty,
                    output_ty,
                    self.clone(),
                )),
                functions: vec![],
            },
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
