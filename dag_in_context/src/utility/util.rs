#[cfg(test)]
use crate::{egglog_test, interpreter::Value};

#[test]
fn test_list_util() -> crate::Result {
    use crate::ast::emptyt;
    let emptyt = emptyt();
    let build = format!(
        "
		(let list (Cons (Const (Int 0) {emptyt})
                  (Cons (Const (Int 1) {emptyt})
                  (Cons (Const (Int 2) {emptyt})
                  (Cons (Const (Int 3) {emptyt})
                  (Cons (Const (Int 4) {emptyt}) (Nil)))))))
		(let expr (Switch (Const (Int 1) {emptyt}) list))
	"
    );
    let check = format!(
        "
		(check (= (ListExpr-ith list 1) (Const (Int 1) {emptyt})))
        (check (= (ListExpr-ith list 4) (Const (Int 4) {emptyt})))
        (check (= (ListExpr-length list) 5))
	"
    );
    egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn append_test() -> crate::Result {
    use crate::ast::emptyt;
    let emptyt = emptyt();
    let build = format!(
        "
        (let appended
            (Append
                (Cons (Const (Int 0) {emptyt}) (Cons (Const (Int 1) {emptyt}) (Nil)))
                (Const (Int 2) {emptyt})))
    "
    );

    let check = format!("
        (check (
            =
            (Cons (Const (Int 0) {emptyt}) (Cons (Const (Int 1) {emptyt}) (Cons (Const (Int 2) {emptyt}) (Nil))))
            appended
        ))
    ");
    egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}

#[test]
fn test_tuple_ith() -> crate::Result {
    use crate::ast::emptyt;
    let emptyt = emptyt();
    let build = format!(
        "
    (let tup (Concat par 
                  (Concat par (Single (Const (Int 0) {emptyt})) (Single (Const (Int 1) {emptyt})))
                  (Concat par (Single (Const (Int 2) {emptyt})) (Single (Const (Int 3) {emptyt})))))
    
    ;; with print
    (let tup2 (Concat par
                (Concat par 
                    (Single (Bop (Print) (Const (Int 0) {emptyt}) (Arg (Base (StateT)))))
                    (Concat par (Single (Const (Int 1) {emptyt})) 
                                (Single (Const (Int 2) {emptyt}))))
                (Single (Const (Int 3) {emptyt}))))
    "
    );

    let check = format!(
        "
    (check (= (tuple-ith tup 0) (Const (Int 0) {emptyt})))
    (check (= (tuple-ith tup 1) (Const (Int 1) {emptyt})))
    (check (= (tuple-ith tup 2) (Const (Int 2) {emptyt})))
    (check (= (tuple-ith tup 3) (Const (Int 3) {emptyt})))
    (check (= 4 (tuple-length tup)))
    (fail (check (tuple-ith tup 4)))

    (check (= (tuple-ith tup2 0) (Bop (Print) (Const (Int 0) {emptyt}) (Arg (Base (StateT))))))
    (check (= (tuple-ith tup2 1) (Const (Int 1) {emptyt})))
    (check (= (tuple-ith tup2 2) (Const (Int 2) {emptyt})))
    (check (= (tuple-ith tup2 3) (Const (Int 3) {emptyt})))
    (check (= 4 (tuple-length tup2)))
    (fail (check (tuple-ith tup2 4)))
    "
    );
    egglog_test(
        &build,
        &check,
        vec![],
        Value::Tuple(vec![]),
        Value::Tuple(vec![]),
        vec![],
    )
}
