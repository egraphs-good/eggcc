pub use crate::*;

#[test]
fn function_inlining() -> Result {
    let build = "
        (let func0-id (id (i64-fresh!)))
        (let func0 (Func func0-id \"main\" (EVec (vec-of 
            (Call \"inc\" (EVec (vec-of (Num func0-id 1))))
        ))))

        (let func1-id (id (i64-fresh!)))
        (let func1 (Func func1-id \"inc\" (EVec (vec-of 
            (badd (Arg func1-id 0) (Num func1-id 1))
        ))))
    ";

    let check = "
        (check
        (=
            func0
            (Func func0-id \"main\" (EVec (vec-of 
                (EVec (vec-of
                    (badd (Num func0-id 1) (Num func0-id 1))
                ))
            )))
        )
        )
    ";
    run_test(build, check)
}
