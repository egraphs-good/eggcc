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

pub fn mk_schedule() -> String {
    let helpers = helpers();
    format!(
        "
(unstable-combined-ruleset saturating
    always-run
    passthrough
    canon
    type-analysis
    context
    interval-analysis
    memory-helpers
    always-switch-rewrite
    loop-iters-analysis
)

(unstable-combined-ruleset cheap-optimizations
    loop-simplify
    memory
    loop-unroll
    peepholes
)

(unstable-combined-ruleset all-optimizations
    cheap-optimizations
    
    switch_rewrite
    loop-inv-motion
    loop-strength-reduction
    loop-peel
)

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
    )
}
