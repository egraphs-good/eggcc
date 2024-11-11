pub(crate) fn helpers() -> String {
    "
(saturate

    (saturate type-helpers)
    (saturate error-checking)
    state-edge-passthrough

    (saturate
        (saturate type-helpers)
        (saturate error-checking)
        saturating
    )

    (saturate drop)
    apply-drop-unions
    cleanup-drop

    (saturate
        (saturate type-helpers)
        (saturate error-checking)
        saturating
    )

    (saturate subst)
    apply-subst-unions
    cleanup-subst

    (saturate boundary-analysis)
)

;; be careful to finish dropping and substituting before subsuming things!
;; otherwise substitution or dropat may not finish, violating the weak linearity invariant
(saturate subsume-after-helpers)
"
    .to_string()
}

fn cheap_optimizations() -> Vec<String> {
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
        "loop-peel",
    ]
    .iter()
    .map(|opt| opt.to_string())
    .chain(cheap_optimizations())
    .collect()
}

fn saturating_rulesets() -> Vec<String> {
    [
        "always-run",
        "passthrough",
        "canon",
        "type-analysis",
        "context",
        "interval-analysis",
        "memory-helpers",
        "always-switch-rewrite",
        "loop-iters-analysis",
    ]
    .iter()
    .map(|opt| opt.to_string())
    .collect()
}

pub fn rulesets() -> String {
    let all_optimizations = optimizations().join("\n");
    let saturating_combined = saturating_rulesets().join("\n");
    let cheap_optimizations = cheap_optimizations().join("\n");
    format!(
        "
(unstable-combined-ruleset saturating
    {saturating_combined}
)

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
    optimizations()
        .iter()
        .map(|optimization| {
            format!(
                "
(run-schedule
   {helpers}
   {optimization}
   {helpers})
"
            )
        })
        .collect()
}

/// Parallel schedule must return a single string,
/// a schedule that runs optimizations over the egraph.
pub fn parallel_schedule() -> Vec<String> {
    let helpers = helpers();

    vec![format!(
        "
(run-schedule

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
