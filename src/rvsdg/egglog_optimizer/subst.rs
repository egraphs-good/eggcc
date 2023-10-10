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
  (function Subst{btype}All (Body {btype}) {btype} :unextractable)

  ;; helper that keeps track of how many arguments
  ;; we have substituted so far
  ;;       (vec of arguments, progress through that vec, and the expression to substitute into
  (function Subst{btype}AllHelper (Body i64 {btype}) {btype}  :unextractable)

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

pub(crate) fn subst_rules() -> String {
    let mut res = vec![include_str!("subst.egg").to_string()];

    for btype in &["Expr", "Operand", "Body", "VecBody"] {
        res.push(subst_all_rule(btype.to_string()));
    }

    res.join("\n")
}
