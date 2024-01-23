#[test]
fn function_inlining() -> crate::Result {
    let build = "
        (let outer-id (Id (i64-fresh!)))
        (let func-id (Id (i64-fresh!)))

        (let func0 
            (Function
                func-id
                (Add (Arg func-id) (Num func-id 1))
            )
        )

        (let call
            (Call
                func-id
                (Num outer-id 2)
            )
        )
        (ExprIsValid call)
    ";

    let check = "
        (check 
            (=
                call
                (Let some-id
                    (Num outer-id 2)
                    (Add (Arg some-id) (Num some-id 1))
                )
            )
        
        )
        (extract call)
    ";

    crate::run_test(build, check)
}
