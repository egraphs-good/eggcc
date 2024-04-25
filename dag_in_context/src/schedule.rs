pub(crate) fn helpers() -> String {
    "
;; saturate all helpers first
(saturate
  (saturate saturating-helpers)
  saturating)

;; run substitution in a phased way, avoiding
;; saturation issues when substitution observes its
;; own equalities.
(saturate subst)
(saturate apply-subst-unions)
(saturate cleanup-subst)

;; saturate all helpers again for new substitutions
(saturate
  (saturate saturating-helpers)
  saturating)"
        .to_string()
}

pub(crate) fn mk_schedule() -> String {
    let helpers = helpers();
    format!(
        "
  ;; soundness of typechecking depends on saturating 
  ;; type-helpers first
  (unstable-combined-ruleset saturating-helpers
    type-helpers)
  
  (unstable-combined-ruleset saturating
    always-run
    canon
    error-checking
    type-analysis
    context
    interval-analysis)
  
    
  (unstable-combined-ruleset optimizations
    switch_rewrite
    loop-simplify)
  
  (run-schedule
    {helpers}
    loop-unroll
    (repeat 6
      {helpers}
      optimizations)
    {helpers})
  "
    )
}
