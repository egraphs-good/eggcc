#[test]
fn test_list_util() -> Result<(), egglog::Error> {
    let build = &*format!("
        (let id (Id 1))
        (let list (Cons (Num id 0) (Cons (Num id 1) (Cons (Num id 2) (Cons (Num id 3) (Cons (Num id 4) (Nil)))))))
        (let t (All (Sequential) list))
    ");
    let check = &*format!(
        "
        (check (= (ListExpr-ith list 1) (Num id 1)))
        (check (= (ListExpr-ith list 4) (Num id 4)))
        (check (= (ListExpr-length list) 5))
        
    "
    );
    crate::run_test(build, check)
}

#[test]
fn append_test() -> Result<(), egglog::Error> {
    let build = "
        (let id (Id (i64-fresh!)))
        (let appended
            (Append
                (Cons (Num id 0) (Cons (Num id 1) (Nil)))
                (Num id 2)))
    ";

    let check = "
        (check (
            =
            (Cons (Num id 0) (Cons (Num id 1) (Cons (Num id 2) (Nil))))
            appended
        ))
    ";

    crate::run_test(build, check)
}

#[test]
fn get_loop_output_ith_test() -> Result<(), egglog::Error> {
    let build = "
    (let id1 (Id (i64-fresh!)))
    (let id-outer (Id (i64-fresh!)))
    (let loop1
        (Loop id1
            (All (Parallel) (Pair (Arg id-outer) (Num id-outer 0)))
            (All (Sequential) (Pair
                ; pred
                (LessThan (Get (Arg id1) 0) (Get (Arg id1) 1))
                ; output
                (All (Parallel) (Pair
                    (Add (Get (Arg id1) 0) (Num id1 1))
                    (Sub (Get (Arg id1) 1) (Num id1 1))))))))
    (let out0 (Add (Get (Arg id1) 0) (Num id1 1)))
    (let out1 (Sub (Get (Arg id1) 1) (Num id1 1)))
    ";

    let check = "
        (check (
            =
            (get-loop-outputs-ith loop 0)
            out0
        ))
        (check (
            =
            (get-loop-outputs-ith loop 1)
            out1
        ))
    ";

    crate::run_test(build, check)
}
