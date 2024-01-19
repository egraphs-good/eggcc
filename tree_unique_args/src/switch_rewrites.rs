pub(crate) fn egglog() -> String {
    let equiv_when_b_pure = [
        (
            "(Switch (And a b) (Cons A (Cons B (Nil))))",
            "(Switch a (Pair (Switch b (Pair A B)) B))",
        ),
        (
            "(Switch (Or a b) (Cons A (Cons B (Nil))))",
            "(Switch a (Pair A (Switch b (Pair A B))))",
        ),
    ];
    let rules = equiv_when_b_pure
        .map(|(e1, e2)| {
            format!(
                "(rewrite {e1} {e2} :when ((ExprIsPure b) (ExprShouldBeValid {e1}))
                                    :ruleset switch-rewrites)"
            )
        })
        .join("\n");
    format!("(ruleset switch-rewrites)\n{rules}")
}

#[test]
fn switch_rewrite_and() -> crate::Result {
    let build = "
(let id (Id (i64-fresh!)))
(let switch (Switch (And (Boolean id false) (Boolean id true))
                    (Pair (Num id 1) (Num id 2))))
(ExprShouldBeValid switch)
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
(ExprShouldBeValid switch)
    ";
    let check = "
(check (= switch (Switch (Boolean id false)
                         (Pair (Num id 1)
                               (Switch (Boolean id true)
                                       (Pair (Num id 1) (Num id 2)))))))
    ";
    crate::run_test(build, check)
}

#[test]
fn switch_rewrite_purity() -> crate::Result {
    let build = "
(let switch-id (Id (i64-fresh!)))
(let let-id (Id (i64-fresh!)))
(let impure (Let let-id (UnitExpr let-id) (All (Sequential) (Pair (Boolean let-id true) (Print (Num let-id 1))))))
(let switch (Switch (And (Boolean switch-id false) (Get impure 0))
                    (Pair (Num switch-id 1) (Num switch-id 2))))
(ExprShouldBeValid switch)
    ";
    let check = "
(fail (check (= switch (Switch (Boolean switch-id false)
                               (Pair (Switch (Get impure 0)
                                             (Pair (Num switch-id 1) (Num switch-id 2)))
                                     (Num switch-id 2))))))
    ";
    crate::run_test(build, check)?;

    let build = "
(let switch-id (Id (i64-fresh!)))
(let let-id (Id (i64-fresh!)))
(let impure (Let let-id (UnitExpr let-id) (All (Sequential) (Cons (Boolean let-id true) (Nil)))))
(let switch (Switch (And (Boolean switch-id false) (Get impure 0))
                    (Pair (Num switch-id 1) (Num switch-id 2))))
(ExprShouldBeValid switch)
    ";
    let check = "
(check (= switch (Switch (Boolean switch-id false)
                               (Pair (Switch (Get impure 0)
                                             (Pair (Num switch-id 1) (Num switch-id 2)))
                                     (Num switch-id 2)))))
    ";
    crate::run_test(build, check)
}
