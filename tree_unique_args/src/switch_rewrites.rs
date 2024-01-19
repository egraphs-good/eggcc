pub(crate) fn rules() -> String {
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
    let rules_needing_purity = equiv_when_b_pure
        .map(|(e1, e2)| {
            format!(
                "(rewrite {e1} {e2} :when ((ExprIsPure b) (ExprIsValid {e1}))
                                    :ruleset switch-rewrites)
                 (rewrite {e2} {e1} :when ((ExprIsPure b) (ExprIsValid {e2}))
                                    :ruleset switch-rewrites)"
            )
        })
        .join("\n");
    format!(
        "(ruleset switch-rewrites)
    
        ; Constant condition elimination
        (rewrite (Switch (Boolean id true) (Cons A (Cons B (Nil))))
                 A
                 :when ((ExprIsValid (Switch (Boolean id true) (Cons A (Cons B (Nil))))))
                 :ruleset switch-rewrites)
        (rewrite (Switch (Boolean id false) (Cons A (Cons B (Nil))))
                 B
                 :when ((ExprIsValid (Switch (Boolean id false) (Cons A (Cons B (Nil))))))
                 :ruleset switch-rewrites)

        ; (if E then S1 else S2); S3 ==> if E then S1;S3 else S2;S3
        (rewrite (All ord (Cons (Switch e (Cons S1 (Cons S2 (Nil)))) S3))
                 (Switch e (Cons (All ord (Cons S1 S3)) (Cons (All ord (Cons S2 S3)) (Nil))))
                 :ruleset switch-rewrites)
    
        {rules_needing_purity}"
    )
}

#[test]
fn switch_rewrite_and() -> crate::Result {
    let build = "
(let id (Id (i64-fresh!)))
(let switch (Switch (And (Boolean id false) (Boolean id true))
                    (Pair (Num id 1) (Num id 2))))
(ExprIsValid switch)
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
(ExprIsValid switch)
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
(let impure (Let let-id (UnitExpr switch-id) (All (Sequential) (Pair (Boolean let-id true) (Print (Num let-id 1))))))
(let switch (Switch (And (Boolean switch-id false) (Get impure 0))
                    (Pair (Num switch-id 1) (Num switch-id 2))))
(ExprIsValid switch)
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
(let impure (Let let-id (UnitExpr switch-id) (All (Sequential) (Cons (Boolean let-id true) (Nil)))))
(let switch (Switch (And (Boolean switch-id false) (Get impure 0))
                    (Pair (Num switch-id 1) (Num switch-id 2))))
(ExprIsValid switch)
    ";
    let check = "
(check (= switch (Switch (Boolean switch-id false)
                               (Pair (Switch (Get impure 0)
                                             (Pair (Num switch-id 1) (Num switch-id 2)))
                                     (Num switch-id 2)))))
    ";
    crate::run_test(build, check)
}

#[test]
fn test_constant_condition() -> Result<(), egglog::Error> {
    let build = "
    (let t (Boolean (Id (i64-fresh!)) true))
    (let f (Boolean (Id (i64-fresh!)) false))
    (let a (Num (Id (i64-fresh!)) 3))
    (let b (Num (Id (i64-fresh!)) 4))
    (let switch_t (Switch t (Cons a (Cons b (Nil)))))
    (let switch_f (Switch f (Cons a (Cons b (Nil)))))
    (ExprIsValid switch_t)
    (ExprIsValid switch_f)
  ";
    let check = "
    (check (= switch_t a))
    (check (= switch_f b))
  ";
    crate::run_test(build, check)
}

#[test]
fn switch_pull_in_below() -> Result<(), egglog::Error> {
    let build = "
    (let c (Read (Num (Id (i64-fresh!)) 3)))
    (let s1 (Read (Num (Id (i64-fresh!)) 4)))
    (let s2 (Read (Num (Id (i64-fresh!)) 5)))
    (let s3 (Read (Num (Id (i64-fresh!)) 6)))

    (let switch (Switch c (Cons s1 (Cons s2 (Nil)))))
    (let lhs (All (Sequential) (Cons switch (Cons s3 (Nil)))))
  ";
    let check = "
    (let s1s3 (All (Sequential) (Cons s1 (Cons s3 (Nil)))))
    (let s2s3 (All (Sequential) (Cons s2 (Cons s3 (Nil)))))
    (let expected (Switch c (Cons s1s3 (Cons s2s3 (Nil)))))
    (check (= lhs expected))
  ";
    crate::run_test(build, check)
}
