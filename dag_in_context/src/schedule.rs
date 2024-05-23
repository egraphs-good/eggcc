pub(crate) fn helpers() -> String {
    // 0 -> 101
    // 1 -> 101
    // 2 -> 119
    // 3 -> 139
    // 4 -> 183
    // 5 -> 299
    // 6 -> 631
    "
(repeat 2

    (saturate type-helpers)
    (saturate error-checking)
    passthrough

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
    canon
    type-analysis
    context
    interval-analysis
    memory-helpers
    always-switch-rewrite
    loop-iters-analysis
)


(unstable-combined-ruleset optimizations
    loop-simplify
    memory
    loop-unroll
    peepholes
)

(unstable-combined-ruleset expensive-optimizations
    optimizations
    switch_rewrite
    ;loop-inv-motion
    loop-strength-reduction
    loop-peel
)

(run-schedule {helpers})

; TODO: add the optimizations back

(run-schedule (saturate debug-deletes))
"
    )
}
