// Checks that loop strength reduction works for repeated multiplication by a passthrough arg
#[test]
fn loop_strength_reduction_passthrough_const() -> Result<(), egglog::Error> {
    let build = "
        (let outer-id (Id (i64-fresh!)))
        (let loop-id (Id (i64-fresh!)))
        (let input-list
                (Cons (Num outer-id 0) ; a
                (Cons (Num outer-id 0) ; i
                (Cons (Num outer-id 3) (Nil)))) ; c
        )
        (let inputs
            (
                All
                outer-id
                (Parallel)
                input-list
            )
        )
        (let pred
            (LessThan (Get (Arg loop-id) 1) (Num loop-id 4))
        )
        (let output-list
            (Cons (Mul (Get (Arg loop-id) 2) (Get (Arg loop-id) 1)) ; i * c
            (Cons (Add (Get (Arg loop-id) 1) (Num loop-id 1)) ; i += 1
            (Cons (Get (Arg loop-id) 2) (Nil))))
        )
        (let loop
            (
                Loop
                loop-id
                inputs
                (All
                    loop-id
                    (Sequential)
                    (Pair
                        pred
                        (All
                            loop-id
                            (Parallel)
                            output-list
                        )
                    )
                )
            )
        )
        (ExprIsValid loop)
    ";

    "
    (Let
        (Id 2)
        (All
            (Id 0)
            (Parallel)
            (Pair (Arg (Id 0)) (Num (Id 0) 0)))
            (Let
                (Id 3)
                (Loop whatever-id
                    (All (Id 2) (Parallel) (Cons (Num (Id 2) 0) (Cons (Get (Arg (Id 2)) 1) (Cons (Num (Id 2) 3) (Cons (Mul (Num (Id 2) 3) (Num (Id 2) 0)) (Nil))))))
                    (All whatever-id (Sequential) (Cons (LessThan (Get (Arg whatever-id) 1) (Num whatever-id 4)) (Cons (All whatever-id (Parallel) (Cons (Get (Arg whatever-id) 3) (Cons (Add (Get (Arg whatever-id) 1) (Num whatever-id 1)) (Cons (Get (Arg whatever-id) 2) (Cons (Add (Get (Arg whatever-id) 3) (Mul (Get (Arg whatever-id) 2) (Num whatever-id 1))) (Nil)))))) (Nil)))))
                (All (Id 3) (Parallel) (Cons (Get (Arg (Id 3)) 0) (Cons (Get (Arg (Id 3)) 1) (Cons (Get (Arg (Id 3)) 2) (Nil)))))))
    ";
    let check = "
    (check (=
            loop
            (Loop
                whatever-id
                (All
                    outer-id
                    (Parallel)
                    (Cons (Num outer-id 0) ; a
                    (Cons (Num outer-id 0) ; i
                    (Cons (Num outer-id 3) ; c
                    (Cons
                        (Mul (Num outer-id 3) (Num outer-id 0))
                        (Nil))))))
                (All
                    whatever-id
                    (Sequential)
                    (Cons
                        (LessThan (Get (Arg whatever-id) 1) (Num whatever-id 4))
                        (Cons (All
                            whatever-id
                            (Parallel)
                            (Cons (Get (Arg whatever-id) 3) ; i * c => d
                            (Cons (Add (Get (Arg whatever-id) 1) (Num whatever-id 1)) ; i += 1
                            (Cons (Get (Arg whatever-id) 2)
                            (Cons 
                                (Add (Get (Arg whatever-id) 3) (Mul (Get (Arg whatever-id) 2)
                                    (Num whatever-id 1)
                                  ))
                            (Nil)))))) (Nil)))))))
    ";

    crate::run_test(build, check)
}

// Checks that loop strength reduction works for repeated multiplication by a Number
#[test]
fn loop_strength_reduction_num_const() -> Result<(), egglog::Error> {
    let build = "
        (let outer-id (Id (i64-fresh!)))
        (let loop-id (Id (i64-fresh!)))
        (let input-list
                (Cons (Num outer-id 0) ; a
                (Cons (Num outer-id 0) (Nil))) ; i
        )
        (let inputs
            (
                All
                outer-id
                (Parallel)
                input-list
            )
        )
        (let pred
            (LessThan (Get (Arg loop-id) 1) (Num loop-id 4))
        )
        (let output-list
            (Cons (Mul (Num loop-id 5) (Get (Arg loop-id) 1)) ; i * c
            (Cons (Add (Get (Arg loop-id) 1) (Num loop-id 1)) (Nil))) ; i += 1
        )
        (let loop
            (
                Loop
                loop-id
                inputs
                (All
                    loop-id
                    (Sequential)
                    (Pair
                        pred
                        (All
                            loop-id
                            (Parallel)
                            output-list
                        )
                    )
                )
            )
        )
        (ExprIsValid loop)
    ";

    let check = "
        (check (
            =
            loop
            (Loop
                whatever-id
                (All
                    outer-id
                    (Parallel)
                    (Cons (Num outer-id 0) ; a
                    (Cons (Num outer-id 0) ; i
                    (Cons
                        (Mul (Num outer-id 5) (Num outer-id 0))
                        (Nil)))))
                (All
                    whatever-id
                    (Sequential)
                    (Cons
                        (LessThan (Get (Arg whatever-id) 1) (Num whatever-id 4))
                        (Cons (All
                            whatever-id
                            (Parallel)
                            (Cons (Get (Arg whatever-id) 2) ; i * c => d
                            (Cons (Add (Get (Arg whatever-id) 1) (Num whatever-id 1)) ; i += 1
                            (Cons 
                                (Add (Get (Arg whatever-id) 2) (Mul (Num whatever-id 5)
                                    (Num whatever-id 1)
                                  ))
                            (Nil))))) (Nil))
                            ))
            )
    ))
    ";

    crate::run_test(build, check)
}
