pub(crate) fn helpers() -> String {
    "
    (saturate
        (saturate
          type-analysis
          (saturate type-helpers))
        error-checking
        always-run
        canon
        interval-analysis
        always-switch-rewrite
        loop-iters-analysis
        ; memory-helpers TODO run memory helpers for memory optimizations
    )

    
    (saturate
        (saturate
          type-analysis
          (saturate type-helpers)
          error-checking
          always-run
          context)
        
        (saturate drop)
        apply-drop-unions
        cleanup-drop

        (saturate subst)
        apply-subst-unions
        cleanup-subst)

    ;; be careful to finish dropping and substituting before subsuming things!
    ;; otherwise substitution or dropat may not finish, violating the weak linearity invariant
    subsume-after-helpers

    (saturate boundary-analysis)
"
    .to_string()
}

fn cheap_optimizations() -> Vec<String> {
    // TODO enable loop peeling
    // currently causes saturation issues, probably by creating dead loops that are allowed to have any value

    ["loop-simplify", "memory", "peepholes"]
        .iter()
        .map(|opt| opt.to_string())
        .collect()
}

fn optimizations() -> Vec<String> {
    [
        "loop-unroll",
        "switch_rewrite",
        "loop-inv-motion",
        "loop-strength-reduction",
    ]
    .iter()
    .map(|opt| opt.to_string())
    .chain(cheap_optimizations())
    .collect()
}

pub fn rulesets() -> String {
    let all_optimizations = optimizations().join("\n");
    let cheap_optimizations = cheap_optimizations().join("\n");
    format!(
        "
(unstable-combined-ruleset cheap-optimizations
    {cheap_optimizations}
)

(unstable-combined-ruleset all-optimizations
    {all_optimizations}
)
    "
    )
}

pub fn mk_sequential_schedule() -> Vec<String> {
    let helpers = helpers();

    let mut res = vec![format!(
        "
(run-schedule
   (saturate
      {helpers}
      passthrough
      state-edge-passthrough))"
    )];
    res.extend(optimizations().iter().map(|optimization| {
        format!(
            "
(run-schedule
   {helpers}
   {optimization}
   {helpers})
"
        )
    }));
    res
}

/// Parallel schedule must return a single string,
/// a schedule that runs optimizations over the egraph.
pub fn parallel_schedule() -> Vec<String> {
    let helpers = helpers();

    vec![format!(
        "
(run-schedule
    (saturate
      {helpers}
      passthrough
      state-edge-passthrough)

    (repeat 2
        {helpers}
        all-optimizations
    )

    (repeat 4
        {helpers}
        cheap-optimizations
    )

    {helpers}
)
"
    )]
}
