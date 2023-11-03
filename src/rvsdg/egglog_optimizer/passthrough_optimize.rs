use super::BRIL_OPS;

pub(crate) fn passthrough_optimize_rules() -> String {
    let ruleset = "fast-analyses";
    let mut res = vec![
        "(sort TermAndCost)".to_string(),
        "(function Smaller (TermAndCost TermAndCost) TermAndCost)".to_string(),
    ];

    for ty in ["Expr", "Operand", "Body", "VecOperand", "VecVecOperand"] {
        res.push(format!("(function {ty}AndCost ({ty} i64) TermAndCost)"));
        res.push(format!(
            "(rule ((= lhs (Smaller ({ty}AndCost t1 cost1)
                        ({ty}AndCost t2 cost2)))
        (<= cost1 cost2))
       ((union lhs ({ty}AndCost t1 cost1)))
      :ruleset {ruleset})"
        ));
        res.push(format!(
            "(rule ((= lhs (Smaller ({ty}AndCost t1 cost1)
                        ({ty}AndCost t2 cost2)))
        (> cost1 cost2))
       ((union lhs ({ty}AndCost t2 cost2)))
      :ruleset {ruleset}
      )"
        ));

        res.push(format!(
            "(function Extracted{ty} ({ty}) TermAndCost
           :merge (Smaller old new))"
        ));
    }

    // Compute smallest Expr bottom-up
    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "(rule ((= lhs ({op} ty a b))
        (= (OperandAndCost expr1 cost1) (ExtractedOperand a))
        (= (OperandAndCost expr2 cost2) (ExtractedOperand b)))
       ((set (ExtractedExpr lhs)
             (ExprAndCost ({op} ty expr1 expr2)
                          (+ 1 (+ cost1 cost2)))))
          :ruleset {ruleset})
"
            )),
            _ => unimplemented!(),
        }
    }

    // PRINT is just like above, but without a type
    res.push(format!(
        "
(rule ((= lhs (PRINT a b))
        (= (OperandAndCost expr1 cost1) (ExtractedOperand a))
        (= (OperandAndCost expr2 cost2) (ExtractedOperand b)))
      ((set (ExtractedExpr lhs)
        (ExprAndCost (PRINT expr1 expr2)
                (+ 1 (+ cost1 cost2)))))
      :ruleset {ruleset})
"
    ));

    // TODO implement Call

    // Constants and arguments get a cost of one
    res.push(format!(
        "
(rule
  ((= lhs (Const ty ops lit)))
  ((set (ExtractedExpr lhs) (ExprAndCost lhs 1)))
  :ruleset {ruleset})
    
(rule
  ((= lhs (Arg index)))
  ((set (ExtractedOperand lhs) (OperandAndCost lhs 1)))
  :ruleset {ruleset})"
    ));

    // Optimization! If a theta passes along argument,
    // can extract the input instead.
    res.push(
        "
    (rule ((= lhs (Project index loop))
           (= loop (Theta pred (VO inputs) (VO outputs)))
           (= (vec-get outputs index) (Arg index))
           (= passedthrough (ExtractedOperand (vec-get inputs index)))
          )
          ((set (ExtractedOperand lhs) passedthrough)))
        "
        .to_string(),
    );

    res.join("\n").to_string()
}
