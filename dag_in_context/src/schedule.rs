use crate::EggccConfig;

#[derive(Debug)]
pub enum CompilerPass {
    // Run the given egglog schedule, then extract
    Schedule(String),
    // Run inlining and the given egglog schedule, then extract
    InlineWithSchedule(String),
}

impl CompilerPass {
    pub fn egglog_schedule(&self) -> &str {
        match self {
            CompilerPass::Schedule(s) => s,
            CompilerPass::InlineWithSchedule(s) => s,
        }
    }
}

pub(crate) fn types_and_indexing() -> String {
    "
    (saturate
        (saturate 
            (saturate type-helpers)
            type-analysis)
        (saturate 
            (saturate type-helpers)
            always-run)
        error-checking)"
        .to_string()
}

pub(crate) fn helpers() -> String {
    let types_and_indexing = types_and_indexing();
    format!(
        "
    ;; optimization rules use always-run helpers like SubTuple
    ;; These should be resolved before substitution
    {types_and_indexing}

    ;; substitution depends on type helpers and ExprIsResolved
    ;; run substitution multiple times for nested substitution
    (saturate
        (saturate 
            (saturate type-helpers)
            type-analysis)

        ;; check which eclasses are resolved
        (saturate is-resolved)
        (saturate term-subst)
        ;; do substutition for one round, subsuming as we go
        (saturate subst)
        ;; apply the equalities found
        apply-subst-unions

        ;; add context
        (saturate context)
        (saturate drop)
        apply-drop-unions
        cleanup-drop
    )

    ;; similar to substitution, run term-related things
    (saturate
     terms
     (saturate
       terms-helpers
       (saturate terms-helpers-helpers)))

    ;; run type checking and indexing again
    ;; for newly created terms from substitution
    {types_and_indexing}

    ;; canonicalization and analysis
    (saturate canon)
    (saturate interval-analysis)
    (saturate mem-simple)

    ;; cicm index
    (saturate cicm-index)

    ;; TODO right now we don't run memory-helpers, we run mem-simple instead

    ;; finally, subsume now that helpers are done
    subsume-after-helpers

    ;; do a boundary analysis for loop invariant code motion
    boundary-analysis-prep
    ;; set which expression to hoist (see evil hack in loop_invariant.egg)
    boundary-analysis

    loop-iters-analysis
"
    )
}

fn cheap_optimizations() -> Vec<String> {
    ["interval-rewrite"]
        .iter()
        .map(|opt| opt.to_string())
        .collect()
}

fn optimizations() -> Vec<String> {
    ["push-in"]
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

pub fn mk_sequential_schedule() -> Vec<CompilerPass> {
    let helpers = helpers();

    let mut res = vec![CompilerPass::Schedule(format!(
        "
(run-schedule
   (saturate
      {helpers}
      passthrough
      state-edge-passthrough))"
    ))];
    res.push(CompilerPass::Schedule(format!(
        "
(run-schedule
  (repeat 3
    {helpers}
    (saturate ivt-analysis)
    loop-inversion)
  
  {helpers})"
    )));
    res.push(CompilerPass::Schedule(format!(
        "
(run-schedule
  (repeat 2
      {helpers}
      swap-if)
  {helpers}
  rec-to-loop
  {helpers})"
    )));
    // Inlining has to be run separately because it does not
    // maintain weak linearity!
    // It doesn't compose with other optimizations.
    res.push(CompilerPass::InlineWithSchedule(format!(
        "
(run-schedule {helpers})"
    )));
    res.extend(optimizations().iter().map(|optimization| {
        CompilerPass::Schedule(format!(
            "
(run-schedule
   {helpers}
   {optimization}
   {helpers})
"
        ))
    }));
    res
}

pub fn parallel_schedule(config: &EggccConfig) -> Vec<CompilerPass> {
    let helpers = helpers();

    vec![
        CompilerPass::Schedule(format!(
            "
(run-schedule
   (saturate
      {helpers}
      passthrough
      state-edge-passthrough)
    (repeat 2
      {helpers}
      swap-if)
    {helpers}
    rec-to-loop
    {helpers})"
        )),
        CompilerPass::Schedule(format!(
            "
(run-schedule
    (repeat 3
      {helpers}
      (saturate ivt-analysis)
      loop-inversion)

    {helpers})"
        )),
        // Inlining has to be run separately because it does not
        // maintain weak linearity!
        // It doesn't compose with other optimizations.
        CompilerPass::InlineWithSchedule(format!(
            "
    (run-schedule {helpers})"
        )),
        CompilerPass::Schedule(format!(
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
    ;; non-weakly-linear optimizations once
        {}

    (repeat 4
        {helpers}
        cheap-optimizations
    )

    (saturate
      {helpers}
      passthrough
      state-edge-passthrough)
    add-to-debug-expr
)
",
            if config.non_weakly_linear {
                "non-weakly-linear"
            } else {
                ""
            }
        )),
    ]
}
