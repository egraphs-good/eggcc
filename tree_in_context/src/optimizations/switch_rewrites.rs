#[cfg(test)]
use crate::egglog_test;

#[test]
fn switch_rewrite_three_quarters_and() -> crate::Result {
    use crate::ast::*;

    let build = tif(and(tfalse(), ttrue()), int(1), int(2));

    let check = tif(tfalse(), tif(ttrue(), int(1), int(2)), int(2));

    egglog_test(
        &format!("{build}"),
        &format!("(check (= {build} {check}))"),
        vec![
            build.to_program(emptyt(), intt()),
            check.to_program(emptyt(), intt()),
        ],
        val_empty(),
        val_int(2),
        vec![],
    )
}

#[test]
fn switch_rewrite_three_quarters_or() -> crate::Result {
    use crate::ast::*;

    let build = tif(or(tfalse(), ttrue()), int(1), int(2));

    let check = tif(tfalse(), int(1), tif(ttrue(), int(1), int(2)));

    egglog_test(
        &format!("{build}"),
        &format!("(check (= {build} {check}))"),
        vec![
            build.to_program(emptyt(), intt()),
            check.to_program(emptyt(), intt()),
        ],
        val_empty(),
        val_int(1),
        vec![],
    )
}

// #[test]
// fn switch_rewrite_purity() -> crate::Result {
//     let build = "
// (let switch-id (Id (i64-fresh!)))
// (let let-id (Id (i64-fresh!)))
// (let impure (Let let-id (All switch-id (Parallel) (Nil)) (All let-id (Sequential) (Pair (Boolean let-id true) (Print (Num let-id 1))))))
// (let switch (Switch (And (Boolean switch-id false) (Get impure 0))
//                     (Pair (Num switch-id 1) (Num switch-id 2))))
// (ExprIsValid switch)
//     ";
//     let check = "
// (fail (check (= switch (Switch (Boolean switch-id false)
//                                (Pair (Num switch-id 1)
//                                      (Switch (Get impure 0)
//                                              (Pair (Num switch-id 1) (Num switch-id 2))))))))
//     ";
//     crate::run_test(build, check)?;

//     let build = "
// (let switch-id (Id (i64-fresh!)))
// (let let-id (Id (i64-fresh!)))
// (let pure   (Let let-id (All switch-id (Parallel) (Nil)) (All let-id (Sequential) (Cons (Boolean let-id true) (Nil)))))
// (let switch (Switch (And (Boolean switch-id false) (Get pure 0))
//                     (Pair (Num switch-id 1) (Num switch-id 2))))
// (ExprIsValid switch)
//     ";
//     let check = "
// (check (= switch (Switch (Boolean switch-id false)
//                          (Pair (Num switch-id 1)
//                                (Switch (Get pure 0)
//                                        (Pair (Num switch-id 1) (Num switch-id 2)))))))
//     ";
//     crate::run_test(build, check)
// }
