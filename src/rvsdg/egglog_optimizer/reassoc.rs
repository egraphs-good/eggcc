//! This file is intended to move constants to the left
//! for addition and multiplication, promoting constant folding

// TODO: we should prune eclasses with constants to just contain the constant
// otherwise, these rules will reassociate constants many times
pub(crate) fn reassoc_rules() -> String {
    let mut res = vec![];
    for op in ["badd", "bmul"] {
        // Move constants to the left using commutativity
        res.push(format!(
            "
     (rule
      ((= num (Node (PureOp (Const (IntT) (const) (Num n1)))))
       (= lhs ({op} (IntT) other num)))
      ((union lhs ({op} (IntT) num other))))"
        ))
    }

    // Make all terms have the form (+ a (+ b (+ ...)))
    // By moving nesting to the right.
    res.push(
        "
      (rule
        ((= lhs (badd (IntT)
                      (Node (PureOp (badd (IntT) a b)))
                      c)))
        ((union lhs
                (badd (IntT)
                      a
                      (Node (PureOp (badd (IntT) b c)))))))
      "
        .to_string(),
    );

    // Move constants up the tree
    res.push(
        "
        (rule
          ((= lhs (badd (IntT)
                        a
                        (Node (PureOp (badd (IntT) b c)))))
           (= b (Node (PureOp (Const (IntT) (const) (Num n1)))))            
          )
          ((union lhs
             (badd (IntT)
                   b
                   (Node (PureOp (badd (IntT) a c))))))               
        )
      "
        .to_string(),
    );

    // Constant fold two adjacent constants
    res.push(
        "
      (rule
        ((= lhs (badd (IntT)
                      a
                      (Node (PureOp (badd (IntT) b c)))))
         (= a (Node (PureOp (Const (IntT) (const) (Num n1)))))
         (= b (Node (PureOp (Const (IntT) (const) (Num n2))))))

        ((union lhs
          (badd (IntT)
            (Node (PureOp (Const (IntT) (const) (Num (+ n1 n2)))))
            c))))      
      "
        .to_string(),
    );

    res.join("\n")
}
