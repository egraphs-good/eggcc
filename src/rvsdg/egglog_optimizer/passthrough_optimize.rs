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
(rule ((= lhs (Project index loop))
        (= loop (Gamma pred inputs outputs))
        (= outputs (VVO outputs-inner))
        (= 2 (vec-length outputs-inner))
        (= outputs0 (VecVecOperand-get outputs 0))
        (= outputs1 (VecVecOperand-get outputs 1))
        (= (VecOperand-get outputs0 index) (Arg index))
        (= (VecOperand-get outputs1 index) (Arg index))
        (= passed-through (VecOperand-get inputs index)))
      ((union lhs passed-through)
        ;; also subsume the project
        (delete (Project index loop))))


;; If a gamma passes 1 along the then branch and
;; 0 along the false branch, union project with predicate
;; BUT only if the gamma is pure!
(rule ((= lhs (Project index loop))
       (= loop (Gamma pred inputs outputs))
       (= outputs (VVO outputs-inner))
       (= 2 (vec-length outputs-inner))
       (= outputs0 (VecVecOperand-get outputs 0))
       (= outputs1 (VecVecOperand-get outputs 1))
       (= (VecOperand-get outputs0 index) (Node (PureOp (Const (BoolT) (const) (Bool false)))))
       (= (VecOperand-get outputs1 index) (Node (PureOp (Const (BoolT) (const) (Bool true))))))
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
       (= outputs0 (VecVecOperand-get outputs 0))
       (= outputs1 (VecVecOperand-get outputs 1))
       (= (VecOperand-get outputs0 index) (Arg index))
       (= (VecOperand-get outputs1 index) (Arg index))
       (= passedthrough (ExtractedOperand (VecOperand-get inputs index))))
      ((set (ExtractedOperand lhs) passedthrough)))

;; if we reach a new context, union
(rule ((= theta (Theta pred inputs outputs))
       (= (BodyAndCost extracted cost)
          (ExtractedBody theta)))
      ((union theta extracted)))
(rule ((= gamma (Gamma pred inputs outputs))
       (= (BodyAndCost extracted cost)
          (ExtractedBody gamma)))
      ((union gamma extracted)))


;; if we reach the function at the top level, union
(rule ((= func (Func name intypes outtypes body))
       (= (VecOperandAndCost extracted cost)
          (ExtractedVecOperand body)))
      ((union func
              (Func name intypes outtypes extracted))))
        "
    .to_string()
}
