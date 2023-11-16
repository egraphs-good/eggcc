pub(crate) fn passthrough_optimize_rules() -> String {
    "
;; #######################  PURE CASES ####################

;; If a gamma passes along an argument in both branches,
;; extract the input instead.
;; can union with the inputs.
;; BUT only if the theta is pure!
(rule ((= lhs (Project index loop))
       (= loop (Theta pred inputs outputs))
       (= (VecOperand-get outputs index) (Arg index))
       (= passed-through (VecOperand-get inputs index))
       (Body-is-pure loop)
      )
      ((union lhs passed-through)
       ;; also subsume the project
       (delete (Project index loop))))

;; If a gamma with two cases passes along an argument in both branches,
;; union project with input
;; BUT only if the gamma is pure!
(rule ((= lhs (Project index gamma))
        (= gamma (Gamma pred inputs outputs))
        (= outputs (VVO outputs-inner))
        (= 2 (vec-length outputs-inner))
        (= outputs0 (VecVecOperandCtx-get outputs 0))
        (= outputs1 (VecVecOperandCtx-get outputs 1))
        (= (VecOperandCtx-get outputs0 index) (Arg index))
        (= (VecOperandCtx-get outputs1 index) (Arg index))
        (= passed-through (VecOperand-get inputs index)))
      ((union lhs passed-through)
        ;; also subsume the project
        (delete (Project index gamma))))


;; If a gamma passes 1 along the then branch and
;; 0 along the false branch, union project with predicate
;; BUT only if the gamma is pure!
(rule ((= lhs (Project index loop))
       (= loop (Gamma pred inputs outputs))
       (= outputs (VVO outputs-inner))
       (= 2 (vec-length outputs-inner))
       (= outputs0 (VecVecOperandCtx-get outputs 0))
       (= outputs1 (VecVecOperandCtx-get outputs 1))
       (= (VecOperandCtx-get outputs0 index) (Node (PureOp (Const (BoolT) (const) (Bool false)))))
       (= (VecOperandCtx-get outputs1 index) (Node (PureOp (Const (BoolT) (const) (Bool true))))))
      ((union lhs pred)
       ;; also subsume the project
       (delete (Project index loop))))


;; #######################  IMPRURE CASES ####################

;; If a theta passes along argument,
;; can extract the input instead.
(rule ((= lhs (Project index loop))
        (= loop (Theta pred inputs outputs))
        (= (VecOperand-get outputs index) (Arg index))
        (= passedthrough (ExtractedOperand (VecOperand-get inputs index)))
      )
      ((set (ExtractedOperand lhs) passedthrough)))

;; If a gamma with two cases passes along an argument in both branches,
;; can extract the input instead.
(rule ((= lhs (Project index loop))
       (= loop (Gamma pred inputs outputs))
       (= outputs (VVO outputs-inner))
       (= 2 (vec-length outputs-inner))
       (= outputs0 (VecVecOperandCtx-get outputs 0))
       (= outputs1 (VecVecOperandCtx-get outputs 1))
       (= (VecOperandCtx-get outputs0 index) (Arg index))
       (= (VecOperandCtx-get outputs1 index) (Arg index))
       (= passedthrough (ExtractedOperand (VecOperand-get inputs index))))
      ((set (ExtractedOperand lhs) passedthrough)))
        "
    .to_string()
}
