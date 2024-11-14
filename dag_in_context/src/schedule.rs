// Hard constraints
// These are constraints that will break eggcc if not respected,
// specifically linearity and inequality
// Type helpers need to be run before error checking
// passthrough depends on 
// * substitution needs to be saturated before extraction
//  * all the soft constraints need to be run before substitution

// Soft constraints
// * Type helpers need to saturate right after type analysis to resolve the types
// * always-run depends on type helpers and type analysis saturating
// * Most optimizations depend on always run
// * always-run, type helpers need to be run before error checking

pub(crate) fn helpers() -> String {
    "
(repeat 1
    (saturate
        (saturate 
            (saturate type-helpers)
            type-analysis)
        (saturate 
            (saturate type-helpers)
            always-run))
    error-checking

    (saturate interval-analysis)
    (saturate always-switch-rewrite)
    (saturate
        (saturate memory-always-run)
        (saturate memory-helpers)
        (saturate memory))

    (saturate canon)

    (repeat 2
        state-edge-passthrough
        (repeat 1
            passthrough

            subsume-after-helpers
        )
    )

    (saturate
        (saturate 
            (saturate type-helpers)
            type-analysis)
        (saturate is-resolved)

        (saturate subst)
        apply-subst-unions
        cleanup-subst
        (saturate context)

        (saturate drop)
        apply-drop-unions
        cleanup-drop
    )
)

boundary-analysis
(saturate loop-iters-analysis)
"
    .to_string()
}

pub(crate) fn after_helpers() -> String {
    "
    (saturate 
        (saturate type-helpers)
        always-run-postprocess)
"
    .to_string()
}

fn cheap_optimizations() -> String {
    [
        "loop-simplify",
        "memory",
        "peepholes",
    ]
    .iter()
    .map(|opt| opt.to_string())
    .collect::<Vec<String>>()
    .join("\n")
}

fn optimizations() -> Vec<String> {
    [
        "loop-unroll",
        "switch_rewrite",
        "loop-inv-motion",
        "loop-strength-reduction",
        "loop-peel",
        "loop-inversion",
    ]
    .iter()
    .map(|opt| opt.to_string())
    .chain(std::iter::once(cheap_optimizations()))
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
    let cheap_optimizations = cheap_optimizations();
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
    let after_helpers = after_helpers();
    optimizations()
        .iter()
        .map(|optimization| {
            format!(
                "
(run-schedule
   {helpers}
   {optimization}
   {after_helpers})

(run-schedule {helpers})
(run-schedule
    (saturate
        (saturate 
            (saturate type-helpers)
            type-analysis)
        (saturate is-resolved)

        (saturate subst)
        apply-subst-unions
        cleanup-subst
        (saturate context)

        (saturate drop)
        apply-drop-unions
        cleanup-drop
    )
)
"
            )
        })
        .collect()
}

/// Parallel schedule must return a single string,
/// a schedule that runs optimizations over the egraph.
pub fn parallel_schedule() -> Vec<String> {
    let helpers = helpers();
    let after_helpers = after_helpers();
    let mut schedule = "".to_string();
    let all_optimization_iter = 2;
    let cheap_optimization_iter = 4;
    for _ in 0..all_optimization_iter {
        schedule.push_str(&format!(
            "
            (run-schedule {helpers})
            (run-schedule all-optimizations)
            (run-schedule {after_helpers})
            "
        ));
    }
    for _ in 0..cheap_optimization_iter {
        schedule.push_str(&format!(
            "
            (run-schedule {helpers})
            (run-schedule cheap-optimizations)
            (run-schedule {helpers} {after_helpers})
            "
        ));
    }
    // schedule.push_str(&format!("
    // (run-schedule {helpers})
    // (run-schedule
    //     (saturate
    //         (saturate 
    //             (saturate type-helpers)
    //             type-analysis)
    //         (saturate is-resolved)

    //         (saturate subst)
    //         apply-subst-unions
    //         cleanup-subst
    //         (saturate context)

    //         (saturate drop)
    //         apply-drop-unions
    //         cleanup-drop
    //     )
    // )
    // "));
    vec![schedule]
}
