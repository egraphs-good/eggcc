pub(crate) fn run_optimizations(optimization_set_name: String) -> String {
    format!(
        "
  (repeat 3
    ;; saturate helpers first
    (saturate
      (saturate saturating-helpers)
      saturating)
    
    ;; run substitution in a phased way, avoiding
    ;; saturation issues when substitution observes its
    ;; own equalities.
    (saturate subst)
    (saturate apply-subst-unions)
    (saturate cleanup-subst)

    (saturate
      (saturate saturating-helpers)
      saturating)
    ;; TODO enable this ruleset again after fixing for regions
    ;;conditional-invariant-code-motion
    {optimization_set_name})"
    )
}

pub(crate) fn mk_schedule() -> String {
    let first_three_iterations = run_optimizations("expensive-optimizations".to_string());
    let second_three_iterations = run_optimizations("optimizations".to_string());
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
    context-prop
    interval-analysis)
  
    
  (unstable-combined-ruleset optimizations
    switch_rewrite
    loop-simplify)
  
  (unstable-combined-ruleset expensive-optimizations
    optimizations
    loop-unroll)
  
  (run-schedule
    {first_three_iterations}
    {second_three_iterations})
  "
    )
}
