pub(crate) fn passthrough_optimize_rules() -> String {
    let ruleset = "fast-analyses";
    let mut res = vec![];

    res.push(format!(
        "

;; If a theta passes along argument,
;; can union with the inputs.
;; BUT only if the theta is pure!
(rule ((= lhs (Project index loop))
       (= loop (Theta pred inputs outputs))
       (= (VecOperand-get outputs index) (Arg index))
       (= passed-through (VecOperand-get inputs index))
       (Body-is-pure loop)
      )
      ((union lhs passed-through))
      :ruleset {ruleset})

;; If a gamma passes along an argument in both branches,
;; union project with input
;; BUT only if the gamma is pure!
(rule ((= lhs (Project index loop))
       (= loop (Gamma pred inputs outputs))
       (= outputs (VVO outputs-inner))
       (= 2 (vec-length outputs-inner))
       (= outputs0 (VecVecOperand-get outputs 0))
       (= outputs1 (VecVecOperand-get outputs 1))
       (= (VecOperand-get outputs0 index) (Arg index))
       (= (VecOperand-get outputs1 index) (Arg index))
       (= passed-through (VecOperand-get inputs index)))
      ((union lhs passed-through))
      :ruleset {ruleset})
        "
    ));

    res.join("\n").to_string()
}
