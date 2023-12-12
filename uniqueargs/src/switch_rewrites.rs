pub use crate::*;

#[test]
fn switch_rewrite_and() -> Result {
    let build = "
        (let test-a-id (id (i64-fresh!)))
        (let test-a (Switch
            test-a-id
            (band (Bool global-id false) (Bool global-id true))
            (EVec (vec-of))
            (EVec (vec-of
                (EVec (vec-of (Num test-a-id 2)))
                (EVec (vec-of (Num test-a-id 1)))))))
    ";
    let check = "
        (check (= test-a
            (Switch
                test-b-id
                (Bool global-id false)
                (EVec (vec-of (Bool global-id true)))
                (EVec (vec-of
                    (EVec (vec-of (Num test-b-id 2)))
                    (EVec (vec-of (Project (Switch
                        test-c-id
                        (Arg test-b-id 0)
                        (EVec (vec-of))
                        (EVec (vec-of
                            (EVec (vec-of (Num test-c-id 2)))
                            (EVec (vec-of (Num test-c-id 1)))))
                    ) 0)))))
            )))
    ";
    run_test(build, check)
}

#[test]
fn switch_rewrite_or() -> Result {
    let build = "
        (let test-a-id (id (i64-fresh!)))
        (let test-a (Switch
            test-a-id
            (bor (Bool global-id false) (Bool global-id true))
            (EVec (vec-of))
            (EVec (vec-of
                (EVec (vec-of (Num test-a-id 2)))
                (EVec (vec-of (Num test-a-id 1)))))))
    ";
    let check = "
        (check (= test-a
            (Switch
                test-b-id
                (Bool global-id false)
                (EVec (vec-of (Bool global-id true)))
                (EVec (vec-of
                    (EVec (vec-of (Project (Switch
                        test-c-id
                        (Arg test-b-id 0)
                        (EVec (vec-of))
                        (EVec (vec-of
                            (EVec (vec-of (Num test-c-id 2)))
                            (EVec (vec-of (Num test-c-id 1)))))
                    ) 0)))
                    (EVec (vec-of (Num test-b-id 1)))
                ))
            )))
    ";
    run_test(build, check)
}

#[test]
fn switch_rewrite_demorgan_or() -> Result {
    let build = "(let a (bnot (bor (Arg global-id 0) (Arg global-id 1))))";
    let check = "(check (= a (band (bnot (Arg global-id 0)) (bnot (Arg global-id 1)))))";
    run_test(build, check)
}

#[test]
fn switch_rewrite_demorgan_and() -> Result {
    let build = "(let a (bnot (band (Arg global-id 0) (Arg global-id 1))))";
    let check = "(check (= a (bor (bnot (Arg global-id 0)) (bnot (Arg global-id 1)))))";
    run_test(build, check)
}

#[test]
fn switch_rewrite_double_neg() -> Result {
    let build = "(let a (bnot (bnot (Arg global-id 0))))";
    let check = "(check (= a (Arg global-id 0)))";
    run_test(build, check)
}
