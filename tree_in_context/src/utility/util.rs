#[cfg(test)]
use crate::{egglog_test, interpreter::Value};

#[test]
fn test_list_util() -> crate::Result {
    let build = "
		(let list (Cons (Const (Int 0))
                  (Cons (Const (Int 1))
                  (Cons (Const (Int 2))
                  (Cons (Const (Int 3))
                  (Cons (Const (Int 4)) (Nil)))))))
		(let expr (Switch (Const (Int 1)) list))
	";
    let check = "
		(check (= (ListExpr-ith list 1) (Const (Int 1))))
        (check (= (ListExpr-ith list 4) (Const (Int 4))))
        (check (= (ListExpr-length list) 5))
	";
    egglog_test(
        build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn append_test() -> crate::Result {
    let build = "
        (let appended
            (Append
                (Cons (Const (Int 0)) (Cons (Const (Int 1)) (Nil)))
                (Const (Int 2))))
    ";

    let check = "
        (check (
            =
            (Cons (Const (Int 0)) (Cons (Const (Int 1)) (Cons (Const (Int 2)) (Nil))))
            appended
        ))
    ";
    egglog_test(
        build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn test_tuple_ith() -> crate::Result {
    let build = "
    (let tup (Concat par 
                  (Concat sequ (Single (Const (Int 0))) (Single (Const (Int 1))))
                  (Concat sequ (Single (Const (Int 2))) (Single (Const (Int 3))))))
    
    ;; with print
    (let tup2 (Concat par
                (Concat par 
                    (Uop (Print) (Const (Int 0))) 
                    (Concat par (Single (Const (Int 1))) 
                                (Concat par (Uop (Print) (Const (Int 0))) 
                                            (Single (Const (Int 2))))))
                (Concat par (Single (Const (Int 3))) (Uop (Print) (Const (Int 0))))))
    ";

    let check = "
    (check (= (tuple-ith tup 0) (Const (Int 0))))
    (check (= (tuple-ith tup 1) (Const (Int 1))))
    (check (= (tuple-ith tup 2) (Const (Int 2))))
    (check (= (tuple-ith tup 3) (Const (Int 3))))
    (check (= 4 (tuple-length tup)))
    (fail (check (tuple-ith tup 4)))

    (check (= (tuple-ith tup2 0) (Const (Int 1))))
    (check (= (tuple-ith tup2 1) (Const (Int 2))))
    (check (= (tuple-ith tup2 2) (Const (Int 3))))
    (check (= 3 (tuple-length tup2)))
    (fail (check (tuple-ith tup2 3)))
    ";
    egglog_test(
        build,
        check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
