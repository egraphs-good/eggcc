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

pub(crate) fn helpers() -> String {
    "
    ;; first, run substitution and drop to saturation
    ;; these depend on type analysis, always-run, and context
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

        (saturate term-subst)
        (saturate subst)
        apply-subst-unions
        cleanup-subst)

    ;; now run canonicalization helpers, interval analysis
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

    ;; finally, subsume now that helpers are done
    subsume-after-helpers

    ;; do a boundary analysis for loop invariant code motion
    (saturate boundary-analysis)
    (saturate terms)
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
        "select_opt",
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
  (repeat 2
    {helpers}
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

pub fn parallel_schedule() -> Vec<CompilerPass> {
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
      loop-inversion)

    {helpers})"
        )),
        CompilerPass::InlineWithSchedule(format!(
            "
;; HACK: when INLINE appears in this string
;; we perform inlining in this pass
(run-schedule
    (saturate
      {helpers}
      passthrough)
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
        )),
    ]
}
