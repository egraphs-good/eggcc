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
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
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
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
    )
}