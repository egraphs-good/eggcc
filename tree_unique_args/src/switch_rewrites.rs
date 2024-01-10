pub(crate) fn egglog() -> String {
    format!(
        "
; Generated from switch_rewrites.rs
(ruleset switch-rewrites)

(rewrite (Switch (And a b) (Cons A (Cons B (Nil))))
         (Switch a (Pair (Switch b (Pair A B))
                         B))
         :ruleset switch-rewrites)

(rewrite (Switch (Or a b) (Cons A (Cons B (Nil))))
         (Switch a (Pair A
                         (Switch b (Pair A B))))
         :ruleset switch-rewrites)
"
    )
}
