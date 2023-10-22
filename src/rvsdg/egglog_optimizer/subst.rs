use super::BRIL_OPS;

// TODO handle subst from arbitrary Body
// instead of just a `VO`
fn subst_all_rule(btype: String) -> String {
    format!(
        "
  ;;############## Subst{btype}All ##############
  ;; each operand in the vector corresponds to an argument
  ;; and is substituted as that argument
  ;; For example, (SubstExprAll (vec-of (Const 1) (Const 2))
  ;;                            (Add (Arg 0) (Arg 1)))
  ;; => (Add (Const 1) (Const 2))
  (function Subst{btype}All (VecOperand {btype}) {btype} :unextractable)

  ;; helper that keeps track of how many arguments
  ;; we have substituted so far
  ;;       (vec of arguments, progress through that vec, and the expression to substitute into
  (function Subst{btype}AllHelper (VecOperand i64 {btype}) {btype}  :unextractable)

  (rewrite (Subst{btype}All arg-vec expr)
           (Subst{btype}AllHelper arg-vec 0 expr)
           :ruleset subst)

  (rule
    ((= helper (Subst{btype}AllHelper (VO arg-vec) progress expr))
      (< progress (vec-length arg-vec)))
    ((union helper
            (Subst{btype}AllHelper
              (VO arg-vec)
              ;; increment progress
              (+ progress 1)
              ;; substitute the current argument into the expression
              (Subst{btype} expr progress (vec-get arg-vec progress)))))
      :ruleset subst)

  ;; base case: we are done substituting
  (rule
    ((= helper (Subst{btype}AllHelper (VO arg-vec) progress expr))
      (= progress (vec-length arg-vec)))
    ((union helper expr))
    :ruleset subst)
  "
    )
}

// rtjoa: We could parameterize on `above`'s type (currently, always Body) to
// support substituting under a Function as well
fn subst_beneath_rules() -> Vec<String> {
    let mut res = vec!["
        (relation can-subst-Expr-beneath (Body Expr Expr))
        (relation can-subst-Operand-beneath (Body Operand Operand))
        (relation can-subst-Body-beneath (Body Body Body))
        (relation can-subst-VecVecOperand-beneath (Body VecVecOperand VecVecOperand))
        (relation can-subst-VecOperand-beneath (Body VecOperand VecOperand))

        ;; Base case 'do the substitution' rules
        (rule ((can-subst-Operand-beneath above from to)
               (= above     (Theta from inputs outputs)))
              ((union above (Theta to   inputs outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= above     (Theta pred inputs from)))
              ((union above (Theta pred inputs to)))
              :ruleset subst)
        (rule ((can-subst-Operand-beneath above pred-from pred-to)
               (can-subst-VecOperand-beneath above outputs-from outputs-to)
               (= above     (Theta pred-from inputs outputs-from)))
              ((union above (Theta pred-from inputs outputs-to)))
              :ruleset subst)
        (rule ((can-subst-VecVecOperand-beneath above from to)
               (= above     (Gamma pred inputs from)))
              ((union above (Gamma pred inputs to)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= above     (OperandGroup from)))
              ((union above (OperandGroup to)))
              :ruleset subst)

        ;; Learn can-subst-Operand-beneath
        (rule ((can-subst-Body-beneath above from to)
               (= new-from (Node from)))
              ((can-subst-Operand-beneath above new-from (Node to)))
              :ruleset subst)
        (rule ((can-subst-Body-beneath above from to)
               (= new-from (Project i from)))
              ((can-subst-Operand-beneath above new-from (Project i to)))
              :ruleset subst)

        ;; Learn can-subst-body-beneath
        (rule ((can-subst-Expr-beneath above from to)
               (= new-from (PureOp from)))
              ((can-subst-Body-beneath above new-from (PureOp to)))
              :ruleset subst)
        ;; Propagates up same context (Gamma: pred & inputs, Theta: inputs)
        ;; rtjoa: Is it sound to propagate up outputs if we renumber args?
        (rule ((can-subst-Operand-beneath above from to)
               (= new-from (Gamma from inputs outputs)))
              ((can-subst-Body-beneath above new-from (Gamma to inputs outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (Gamma pred from outputs)))
              ((can-subst-Body-beneath above new-from (Gamma pred to outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (Theta pred from outputs)))
              ((can-subst-Body-beneath above new-from (Theta pred to outputs)))
              :ruleset subst)
        (rule ((can-subst-VecOperand-beneath above from to)
               (= new-from (OperandGroup from)))
              ((can-subst-Body-beneath above new-from (OperandGroup to)))
              :ruleset subst)
        "
    .into()];

    // Learn can-subst-expr-beneath
    res.push(
        "
      (rule ((can-subst-VecOperand-beneath above from to)
              (= new-from (Call ty f from n-outs)))
             ((can-subst-Expr-beneath above new-from (Call ty f to n-outs)))
            :ruleset subst)"
            .into(),
    );
    for bril_op in BRIL_OPS {
        let op = bril_op.op;

        match bril_op.input_types.as_ref() {
            [Some(_), Some(_)] => res.push(format!(
                "
              (rule ((can-subst-Operand-beneath above from to)
                      (= new-from ({op} type from e2)))
                     ((can-subst-Expr-beneath above new-from ({op} type to e2)))
                    :ruleset subst)
              (rule ((can-subst-Operand-beneath above from to)
                      (= new-from ({op} type e1 from)))
                     ((can-subst-Expr-beneath above new-from ({op} type e1 to)))
                    :ruleset subst)
                     ",
            )),
            _ => unimplemented!(),
        };
    }

    // Learn can-subst-{VecOperand, VecVecOperand}-beneath
    for (vectype, ctor, eltype) in &[
        ("VecOperand", "VO", "Operand"),
        ("VecVecOperand", "VVO", "VecOperand"),
    ] {
        // Reify vec-get to be able to be able to match on vec contents
        res.push(format!(
            "
          (function {vectype}-get ({vectype} i64) {eltype})
          (rule (({ctor} x) (> (vec-length x) 0))
                ((union ({vectype}-get ({ctor} x) 0) (vec-get x 0)))
                :ruleset subst)
          (rule (({vectype}-get ({ctor} x) j)
                 (= i (+ j 1)) (< i (vec-length x)))
                ((union ({vectype}-get ({ctor} x) i) (vec-get x i)))
                :ruleset subst)"
        ));
        // Propagate info bottom-up
        res.push(format!(
            "
          (rule ((can-subst-{eltype}-beneath above from to)
                 (= from ({vectype}-get ({ctor} vec) i)))
                ((can-subst-{vectype}-beneath
                    above
                    ({ctor} vec)
                    ({ctor} (vec-set vec i to))))
                :ruleset subst)"
        ));
    }

    res
}

pub(crate) fn subst_rules() -> String {
    let mut res = vec![include_str!("subst.egg").to_string()];

    for btype in &["Expr", "Operand", "Body", "VecOperand", "VecVecOperand"] {
        res.push(subst_all_rule(btype.to_string()));
    }

    res.extend(subst_beneath_rules());
    res.join("\n")
}
