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

    subsume-after-helpers

    (saturate boundary-analysis)
)"
    .to_string()
}

fn expensive_optimizations() -> Vec<String> {
    ["loop-unroll"].iter().map(|opt| opt.to_string()).collect()
}

fn optimizations() -> Vec<String> {
    [
        "loop-simplify",
        "memory",
        "peepholes",
        "switch_rewrite",
        "loop-inv-motion",
        "loop-strength-reduction",
        "loop-peel",
    ]
    .iter()
    .map(|opt| opt.to_string())
    .chain(expensive_optimizations().into_iter())
    .collect()
}

const OPTIMIZATIONS: &[&str] = &[];

const SATURATING: &[&str] = &[
    "always-run",
    "passthrough",
    "canon",
    "type-analysis",
    "context",
    "interval-analysis",
    "memory-helpers",
    "always-switch-rewrite",
    "loop-iters-analysis",
];

pub fn rulesets() -> String {
    let all_optimizations = OPTIMIZATIONS.join("\n");
    let saturating_combined = SATURATING.join("\n");
    format!(
        "
(unstable-combined-ruleset saturating
    {saturating_combined}
)

(unstable-combined-ruleset all-optimizations
    {all_optimizations}
)
    "
    )
}

pub fn mk_sequential_schedule() -> Vec<String> {
    let helpers = helpers();
    OPTIMIZATIONS
        .iter()
        .map(|optimization| {
            format!(
                "
(run-schedule
   {helpers}
   {optimization})
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
    (repeat 3
        {helpers}
        all-optimizations
    )

    {helpers}
)
"
    )]
}
