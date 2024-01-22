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

#[test]
fn switch_rewrite_purity() -> crate::Result {
    let build = "
(let switch-id (Id (i64-fresh!)))
(let let-id (Id (i64-fresh!)))
(let impure (Let let-id (UnitExpr let-id) (All (Sequential) (Pair (Boolean let-id true) (Print (Num let-id 1))))))
(let switch (Switch (And (Boolean switch-id false) (Get impure 0))
                    (Pair (Num switch-id 1) (Num switch-id 2))))
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

#[test]
fn switch_interval() -> Result<(), egglog::Error> {
    let build = "
    (let one   (Num (Id (i64-fresh!)) 1))
    (let two   (Num (Id (i64-fresh!)) 2))
    (let three (Num (Id (i64-fresh!)) 3))
    (let four  (Num (Id (i64-fresh!)) 4))
    (let five  (Num (Id (i64-fresh!)) 5))
    (let cc (LessThan two three))
    (let switch (Switch cc (Cons four (Cons five (Nil)))))
    ";
    let check = "
    (check (= switch four))
    ";
    crate::run_test(build, check)
}