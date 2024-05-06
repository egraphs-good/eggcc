pub(crate) fn helpers() -> String {
    "
;; saturate all helpers first
(saturate
  (saturate
     (saturate type-helpers) ;; resolve type helpers, finding correct types
     (saturate error-checking) ;; check for errors, relies on type-helpers saturating
      saturating)
  
  (saturate subst) ;; do e-substitution
  apply-subst-unions ;; apply the unions from substitution
  cleanup-subst ;; clean up substitutions that are done

  (saturate boundary-analysis) ;; find boundaries of invariants
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
    type-analysis
    context
    interval-analysis
    memory-helpers
  )
  
    
  (unstable-combined-ruleset optimizations
    loop-simplify
    memory
    loop-unroll
    peepholes
  )

  (unstable-combined-ruleset expensive-optimizations
    optimizations
    ;; TODO why is this expensive? On `adler32.bril` it blows up with 3 iterations
    switch_rewrite
    loop-inv-motion
    loop-strength-reduction
  )
  
  (run-schedule
    {helpers}
    loop-peel
    (repeat 2
      {helpers}
      expensive-optimizations)
    (repeat 4
      {helpers}
      optimizations)
    {helpers})
  "
    )
}
