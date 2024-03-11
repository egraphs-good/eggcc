impl TreeProgram {
    pub fn add_context(&self) -> TreeProgram {
        TreeProgram {
            functions: self.functions.iter().map(|f| f.add_context()).collect(),
            entry: self.entry.add_context(),
        }
    }
}

impl RcExpr {
  pub(crate) fn func_add_context(&self) -> RcExpr {
    
  }
}
