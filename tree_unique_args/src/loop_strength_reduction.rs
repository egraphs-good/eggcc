pub(crate) fn egglog() -> String {
    "(ruleset loop-strength-reduction)".to_string()
}

#[test]
fn loop_strength_reduction() -> Result<(), egglog::Error> {
    let build = "
    ;; a = 0
    ;; c = 3
    ;; for  i = 0 to n:
    ;;     a = i * c
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
                (Parallel)
                input-list
            )
        )
        (let pred
            (LessThan (Get (Arg loop-id) 1) (Num loop-id 4))
        )
        (let output-list
            (Cons (Mul (Get (Arg loop-id) 1) (Get (Arg loop-id) 2)) ; i * c
            (Cons (Add (Num loop-id 1) (Get (Arg loop-id) 1)) ; i += 1
            (Cons (Get (Arg loop-id) 2) (Nil))))
        )
        (let loop
            (
                Loop
                loop-id
                inputs
                (All
                    (Sequential)
                    (Pair
                        pred
                        (All
                            (Parallel)
                            output-list
                        )
                    )
                )
            )
        )

        ;; TODO: find invariant
        ;; TODO: how to match on something in a Cons? Use a Get with an unknown i?
        ;; And does this require the Gets to be expanded for it to work? (like with Evec-get)
        (rule
            (
                (= any-loop (Loop asd asdf asdfh))
            )
            
            ((let loop-incr-input
                (Num outer-id 0)
            )
            (let loop-incr-output
                (Add (Num loop-id 1) (Get (Arg loop-id) 1))
            )
            (let c-input
                    (Num outer-id 3)
            )
            (let c-output
                    (Get (Arg loop-id) 2)
            )
            (let old-exp
                (Mul (Get (Arg loop-id) 1) (Get (Arg loop-id) 2))
            )

        ; Each time we need to update d by the product of the multiplied constant and the loop increment
        (let addend (Mul c-output loop-incr-output))

        ; n is index of our new, temporary variable d
        ; NOTE: it's hard to add to the end of a Cons list. Is there a better way than adding at the end?
        ; could we shift over all the Arg #s? To add at the front?
        (let n 3)

        ; Initial value of d is i * c
        ; TODO: need to get idx of i in the *inputs*
        (let d-init (Mul c-input loop-incr-input))

        ; Value of d in loop
        ; the id here will get replaced in the DeepCopy
        (let d-out (Add (Get (Arg loop-id) n) addend))

        ; Construct optimized loop
        (let new-id (Id (i64-fresh!)))

        (let new-inputs (All (Parallel) (Append input-list d-init)))
        (let new-outputs (All (Parallel) (Append output-list d-out)))

        (let new-loop
            (NewLoop
                new-id
                new-inputs
                (All
                    (Sequential)
                    (Pair
                        pred
                        new-outputs))))

        ; We can just union within this new loop now
        (union (DeepCopyExpr old-exp new-id) (Get (Arg new-id) n))

        ; NOTE: I think the difference here is we don't need ExprToEvec or anything
        ; because there is only one output always?
        (union loop new-loop)

    ) :ruleset loop-strength-reduction)
    ";

    let check = "
        (check (
            =
            loop
            (Loop
                whatever-id
                (All
                    (Parallel)
                    (Cons (Num outer-id 0) ; a
                    (Cons (Num outer-id 0) ; i
                    (Cons (Num outer-id 3) ; c
                    (Cons
                        (Mul (Num outer-id 3) (Num outer-id 0))
                        (Nil))))))
                (All
                    (Sequential)
                    (Cons
                        (LessThan (Get (Arg whatever-id) 1) (Num whatever-id 4))
                        (Cons (All
                            (Parallel)
                            (Cons (Get (Arg whatever-id) 3) ; i * c => d
                            (Cons (Add (Num whatever-id 1) (Get (Arg whatever-id) 1)) ; i += 1
                            (Cons (Get (Arg whatever-id) 2)
                            (Cons 
                                (Add (Get (Arg whatever-id) 3) (Mul (Get (Arg whatever-id) 2)
                                    (Add (Num whatever-id 1) (Get (Arg whatever-id) 1))
                                  ))
                            (Nil)))))) (Nil))
                            ))
            )
    ))
    ";

    crate::run_test(build, check)
}
