use super::BRIL_OPS;

/// These extraction rules are not (yet) used to perform actual extraction.
/// Rather, they are used to perform greedy optimizations like passthrough_optimize.
/// The optimizations are performed *during* extraction- they find an opportunity to optimize,
/// and perform the optimization, adding to the Extracted function.
/// Optimizations that use extraction are greedy because they only apply if they lower the cost.
pub(crate) fn extraction_rules() -> String {
    let ruleset = "extraction";
    let vec_ruleset = "extraction-vec";
    let mut res = vec![
        "(sort TermAndCost)".to_string(),
        "(function Smaller (TermAndCost TermAndCost) TermAndCost)".to_string(),
        "(ruleset extraction)".to_string(),
        "(ruleset extraction-vec)".to_string(),
    ];

    for ty in ["Expr", "Operand", "Body", "VecOperand", "VecVecOperand"] {
        res.push(format!(
            "
;; manual, bottom-up extraction of terms using this function
(function Extracted{ty} ({ty}) TermAndCost
            :merge (Smaller old new))
;; Store a term and its cost for this type
(function {ty}AndCost ({ty} i64) TermAndCost)

;; Perform smaller using the next two rules
(rule ((= lhs (Smaller ({ty}AndCost t1 cost1)
                       ({ty}AndCost t2 cost2)))
       (<= cost1 cost2))
      ((union lhs ({ty}AndCost t1 cost1)))
       :ruleset {ruleset})
  
(rule ((= lhs (Smaller ({ty}AndCost t1 cost1)
                       ({ty}AndCost t2 cost2)))
       (> cost1 cost2))
      ((union lhs ({ty}AndCost t2 cost2)))
       :ruleset {ruleset})
"
        ));
    }

    // Compute smallest Expr bottom-up
    for bril_op in BRIL_OPS {
        let op = bril_op.op;
        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
(rule ((= lhs ({op} ty a b))
       (= (OperandAndCost expr1 cost1) (ExtractedOperand a))
       (= (OperandAndCost expr2 cost2) (ExtractedOperand b)))
      ((set (ExtractedExpr lhs)
            (ExprAndCost ({op} ty expr1 expr2)
                         (+ 1 (+ cost1 cost2)))))
        :ruleset {ruleset})
"
            )),
            [Some(_), None] => res.push(format!(
                "
(rule ((= lhs ({op} ty a))
       (= (OperandAndCost expr1 cost1) (ExtractedOperand a)))
      ((set (ExtractedExpr lhs)
            (ExprAndCost ({op} ty expr1)
                         (+ 1 cost1))))
      :ruleset {ruleset})"
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

;; TODO fix this HACK
;; this is how we get an empty vector of vectors in egglog because of
;; typechecking bug in egglog https://github.com/egraphs-good/egglog/issues/113
(let empty-vvo 
  (vec-pop (vec-of (VO (vec-of)))))
"
    ));

    // Rules to extract a vecoperand
    for (vectype, ctor, eltype, empty_vec) in [
        ("VecOperand", "VO", "Operand", "(vec-of)"),
        ("VecVecOperand", "VVO", "VecOperand", "empty-vvo"),
    ] {
        res.push(format!(
            "
(function Extracted{vectype}Helper ({vectype} i64) TermAndCost :merge (Smaller old new))

;; base case: extract nothing
(rule
   (({ctor} vec))
   ((set (Extracted{vectype}Helper ({ctor} vec) 0)
         ({vectype}AndCost ({ctor} {empty_vec}) 0)))
    :ruleset {vec_ruleset})

;; extract one more thing
(rule
   ((= ({vectype}AndCost ({ctor} current) current-cost)
       (Extracted{vectype}Helper ({ctor} vec) index))
    (< index (vec-length vec))
    (= (Extracted{eltype} ({vectype}-get ({ctor} vec) index)) ({eltype}AndCost expr expr-cost)))
   ((set (Extracted{vectype}Helper ({ctor} vec) (+ index 1))
         ({vectype}AndCost
             ({ctor} (vec-push current expr))
             (+ current-cost expr-cost))))
    :ruleset {vec_ruleset})

            
;; finished extracting, create result
(rule
  ((= result
      (Extracted{vectype}Helper ({ctor} vec) index))
   ;; at the end
   (= index (vec-length vec)))
  ((set (Extracted{vectype} ({ctor} vec))
        result))
  :ruleset {vec_ruleset})
      "
        ))
    }

    // TODO implement Call

    res.push(format!(
        "
;; Constant gets cost of 1
(rule
  ((= lhs (Const ty ops lit)))
  ((set (ExtractedExpr lhs) (ExprAndCost lhs 1)))
  :ruleset {ruleset})

;; arg gets cost of 1
(rule
  ((= lhs (Arg index)))
  ((set (ExtractedOperand lhs) (OperandAndCost lhs 1)))
  :ruleset {ruleset})


;; PureOp doesn't add cost
(rule
  ((= lhs (PureOp expr))
   (= (ExprAndCost expr-extracted expr-cost)
      (ExtractedExpr expr)))
   ((set (ExtractedBody lhs) (BodyAndCost (PureOp expr-extracted) expr-cost)))
    :ruleset {ruleset})

;; Nor does Node
(rule
  ((= lhs (Node body))
   (= (BodyAndCost body-extracted body-cost)
      (ExtractedBody body)))
   ((set (ExtractedOperand lhs) (OperandAndCost (Node body-extracted) body-cost)))
    :ruleset {ruleset})

;; Theta gets a cost of 1 for now
(rule
  ((= lhs (Theta pred inputs outputs))
   (= (OperandAndCost pred-extracted pred-cost)
      (ExtractedOperand pred))
   (= (VecOperandAndCost inputs-extracted inputs-cost)
      (ExtractedVecOperand inputs))
   (= (VecOperandAndCost outputs-extracted outputs-cost)
      (ExtractedVecOperand outputs)))
   ((set (ExtractedBody lhs)
         (BodyAndCost
            (Theta pred-extracted inputs-extracted outputs-extracted)
            (+ 1 (+ pred-cost (+ inputs-cost outputs-cost))))))
    :ruleset {ruleset})

;; Gamma gets a cost of 1 for now
(rule
  ((= lhs (Gamma pred inputs outputs))
   (= (OperandAndCost pred-extracted pred-cost)
      (ExtractedOperand pred))
   (= (VecOperandAndCost inputs-extracted inputs-cost)
      (ExtractedVecOperand inputs))
   (= (VecVecOperandAndCost outputs-extracted outputs-cost)
      (ExtractedVecVecOperand outputs)))
  ((set (ExtractedBody lhs)
        (BodyAndCost
          (Gamma pred-extracted inputs-extracted outputs-extracted)
          (+ 1 (+ pred-cost (+ inputs-cost outputs-cost))))))
    :ruleset {ruleset})


;; Project is also free
(rule ((= lhs (Project index body))
       (= (BodyAndCost body-extracted body-cost)
          (ExtractedBody body)))
      ((set (ExtractedOperand lhs)
            (OperandAndCost (Project index body-extracted) body-cost)))
      :ruleset {ruleset})



;; ######################################################
;; The following rules allow optimizations to be applied
;; Optimizations may add to the Extracted table, meaning that 
;; if (= (Extracted body) (BodyAndCost body-extracted body-cost)), `body-extracted` may not be equal
;; to `body`.
;; Once we reach a new context, such as a theta, we can union `body` and `body-extracted`, allowing
;; the optimization to be reflected back into egglog's equivalence relation.
;; Notice that these rules are in the normal optimization ruleset.

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
    ));

    res.join("\n").to_string()
}
