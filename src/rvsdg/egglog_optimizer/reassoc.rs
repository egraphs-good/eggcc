//! This file is intended to move constants to the left
//! for addition and multiplication, promoting constant folding

// TODO: we should prune eclasses with constants to just contain the constant
// otherwise, these rules will reassociate constants many times
pub(crate) fn reassoc_rules() -> String {
    let mut res = vec![];
    for op in ["badd", "bmul"] {
        res.push(format!(
            "
     (rule
      ((= num (Node (PureOp (Const (IntT) (const) (Num n1)))))
       (= lhs ({op} (IntT) other num)))
      ((union lhs ({op} (IntT) num other))))"
        ))
    }

    res.join("\n")
}
