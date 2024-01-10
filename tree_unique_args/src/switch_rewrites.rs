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

#[test]
fn switch_rewrite_and() -> crate::Result {
    let build = "
(let id (Id (i64-fresh!)))
(let switch (Switch (And (Boolean id false) (Boolean id true))
                    (Pair (Num id 1) (Num id 2))))
    ";
    let check = "
(check (= switch (Switch (Boolean id false)
                         (Pair (Switch (Boolean id true)
                                       (Pair (Num id 1) (Num id 2)))
                               (Num id 2)))))
    ";
    crate::run_test(build, check)
}

#[test]
fn switch_rewrite_or() -> crate::Result {
    let build = "
(let id (Id (i64-fresh!)))
(let switch (Switch (Or (Boolean id false) (Boolean id true))
                    (Pair (Num id 1) (Num id 2))))
    ";
    let check = "
(check (= switch (Switch (Boolean id false)
                         (Pair (Num id 1)
                               (Switch (Boolean id true)
                                       (Pair (Num id 1) (Num id 2)))))))
    ";
    crate::run_test(build, check)
}
