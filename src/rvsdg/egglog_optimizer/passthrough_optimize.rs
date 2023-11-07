pub(crate) fn passthrough_optimize_rules() -> String {
    let ruleset = "fast-analyses";
    let mut res = vec![];

    res.push(format!(
        "

;; If a theta passes along argument,
;; can extract the input instead.
(rule ((= lhs (Project index loop))
        (= loop (Theta pred (VO inputs) (VO outputs)))
        (= (vec-get outputs index) (Arg index))
        (= passedthrough (ExtractedOperand (vec-get inputs index)))
      )
      ((set (ExtractedOperand lhs) passedthrough))
      :ruleset {ruleset})

;; If a gamma passes along an argument in both branches,
;; extract the input instead.
(rule ((= lhs (Project index loop))
       (= loop (Gamma pred inputs outputs))
       (= outputs (VVO outputs-inner))
       (= 2 (vec-length outputs-inner))
       (= outputs0 (VecVecOperand-get outputs 0))
       (= outputs1 (VecVecOperand-get outputs 1))
       (= (VecOperand-get outputs0 index) (Arg index))
       (= (VecOperand-get outputs1 index) (Arg index))
       (= passedthrough (ExtractedOperand (VecOperand-get inputs index))))
      ((set (ExtractedOperand lhs) passedthrough))
      :ruleset {ruleset})


;; if we reach a new context, union
(rule ((= theta (Theta pred inputs outputs))
       (= (BodyAndCost extracted cost)
          (ExtractedBody theta)))
      ((union theta extracted))
      :ruleset {ruleset})
(rule ((= gamma (Gamma pred inputs outputs))
       (= (BodyAndCost extracted cost)
          (ExtractedBody gamma)))
      ((union gamma extracted))
      :ruleset {ruleset})


;; if we reach the function at the top level, union
(rule ((= func (Func name intypes outtypes body))
       (= (VecOperandAndCost extracted cost)
          (ExtractedVecOperand body)))
      ((union func
              (Func name intypes outtypes extracted)))
      :ruleset {ruleset})
        "
    ));

    res.join("\n").to_string()
}
