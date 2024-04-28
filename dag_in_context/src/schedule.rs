pub(crate) fn helpers() -> String {
    "
;; saturate all helpers first
(saturate
  (saturate saturating)
  (saturate error-checking) ;; check for errors, relies on type-helpers saturating
  (saturate subst) ;; do e-substitution
  apply-subst-unions ;; apply the unions from substitution
  cleanup-subst ;; clean up substitutions that are done
)
"
    .to_string()
}

pub fn mk_schedule() -> String {
    let helpers = helpers();
    format!(
        "
  (unstable-combined-ruleset saturating
    always-run
    canon
    type-helpers
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
