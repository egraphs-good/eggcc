#[test]
fn test_list_util() -> Result<(), egglog::Error> {
    let build = &*"
        (let id (Id 1))
        (let list (Cons (Num id 0) (Cons (Num id 1) (Cons (Num id 2) (Cons (Num id 3) (Cons (Num id 4) (Nil)))))))
        (let t (All (Sequential) list))
    ".to_string();
    let check = &*"
        (check (= (ListExpr-ith list 1) (Num id 1)))
        (check (= (ListExpr-ith list 4) (Num id 4)))
        (check (= (ListExpr-length list) 5))
        
    "
    .to_string();
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
