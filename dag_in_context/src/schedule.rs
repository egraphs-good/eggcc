pub(crate) fn helpers() -> String {
    "
;; saturate all helpers first
(saturate
  (saturate saturating-helpers)
  saturating)
"
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
    interval-analysis
    subst
    apply-subst-unions
    cleanup-subst)
  
    
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
