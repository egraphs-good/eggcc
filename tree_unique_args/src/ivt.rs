#![cfg(test)]

use crate::{run_test, Result};

#[test]
fn basic_ivt() -> Result {
    // do {
    //     if (x < 1) {
    //         (print 0);
    //     } else {
    //         (print 1);
    //     }
    // } while (x == 0);
    //
    // =>
    //
    // if (x < 1) {
    //     do {
    //         (print 0);
    //     } while (x == 0);
    // } else {
    //     (print 1)
    // }
    let build = "
        (let outer-id (Id (i64-fresh!)))
        (let loop-id (Id (i64-fresh!)))
        (let pred (LessThan (Arg loop-id) (Num loop-id 1)))
        (let switch (Switch pred
                            (Pair (Print (Num loop-id 0))
                                  (Print (Num loop-id 1)))))
        (let loop (Loop loop-id (Arg outer-id) (All loop-id (Sequential) (Pair pred switch))))
        (ExprIsValid loop)";
    let check = "
        (check (= loop
          (Switch
            (LessThan (Arg outer-id) (Num outer-id 1))
            (Cons
                (Loop new-id (Arg outer-id)
                    (All new-id (Sequential)
                        (Cons (LessThan (Arg new-id) (Num new-id 1))
                        (Cons (Print (Num new-id 0)) (Nil)))))
                (Cons (Print (Num outer-id 1)) (Nil))))))";
    run_test(build, check)
}
