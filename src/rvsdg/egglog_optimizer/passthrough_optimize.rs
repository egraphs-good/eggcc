pub(crate) fn passthrough_optimize_rules() -> String {
    let ruleset = "fast-analyses";
    let mut res = vec![];

    res.push(format!(
        "

;; Optimization! If a theta passes along argument,
;; can extract the input instead.
(rule ((= lhs (Project index loop))
        (= loop (Theta pred (VO inputs) (VO outputs)))
        (= (vec-get outputs index) (Arg index))
        (= passedthrough (ExtractedOperand (vec-get inputs index)))
      )
      ((set (ExtractedOperand lhs) passedthrough))
      :ruleset {ruleset})


;; if we reach the function at the top level, union
(rule ((= func (Func name intypes outtypes body))
       (= (VecOperandAndCost extracted cost)
          (ExtractedVecOperand body)))
      ((union func
              (Func name intypes outtypes extracted)))
      :ruleset {ruleset}
      )
        "
    ));

    res.join("\n").to_string()
}
